use std::{collections::HashMap, sync::Arc};

use excalidraw_core::{
    arrowhead::Arrowhead,
    bounds::Bounds,
    element::{ConvertTarget, Element},
    factory::*,
    geometry::Point,
    scene::AppState,
    types::{FontFamily, Theme},
};
use gpui_kit::{base::StyledExt, *};

use crate::{
    chrome,
    design::{Tokens, size},
    i18n::{I18n, Language},
    state::{Document, DragMode, History, ResizeHandle, Tool, screen_to_world, selection_bounds},
};

/// Default size for a single click with a shape tool (Excalidraw-like).
const CLICK_DEFAULT_SIZE: f64 = 100.0;
/// Below this drag distance a press is treated as a click.
const CLICK_THRESHOLD: f64 = 3.0;
/// Vertical gap between the text elements a multi-line paste splits into,
/// matching `LINE_GAP` in upstream `App.clipboard.ts`.
const PASTE_LINE_GAP: f64 = 10.0;

/// The element-level event handler [`Editor::act`] hands back.
///
/// Boxed rather than an `impl Fn` because `gpui-pre` is an edition-2024 crate:
/// its `Context::listener` return type implicitly captures the borrow of the
/// context, which an opaque return type cannot express from here.
pub(crate) type Handler<E> = Box<dyn Fn(&E, &mut Window, &mut App)>;

pub struct Editor {
    pub document: Document,
    pub history: History,
    pub i18n: I18n,
    drawing: bool,
    drag_start: Option<Point>,
    last_point: Option<Point>,
    current_points: Vec<Point>,
    panning: bool,
    pan_start: Option<Point>,
    pan_scroll: Option<Point>,
    drag_mode: Option<DragMode>,
    resize_origin: Option<Bounds>,
    rotate_start_angle: f64,
    editing_text: Option<String>,
    text_edit_started: bool,
    /// The next typed character should replace the placeholder text.
    replace_on_type: bool,
    marquee_start: Option<Point>,
    marquee_current: Option<Point>,
    laser_points: Vec<Point>,
    erasing: bool,
    /// Committed vertices of an in-progress multi-point line/arrow.
    pending_points: Vec<Point>,
    /// Live cursor position in world space (for the rubber-band guide).
    cursor_world: Option<Point>,
    /// Modifiers held during the last pointer motion over the canvas.
    ///
    /// The live drag preview has to honour shift-square *before* the shape is
    /// committed, and the paint pass has no access to the event that produced
    /// the current cursor position, so it is latched here.
    cursor_mods: Modifiers,
    /// Last pointer position in window space (upstream `viewport.lastPosition`).
    ///
    /// Stays at (0, 0) until the first pointer move, which is how upstream
    /// decides there is nowhere to hang a cursor hint.
    cursor_screen: (f64, f64),
    /// Focus anchor for the whole editor.
    ///
    /// GPUI routes keystrokes along the focused node's path and only falls back
    /// to the *dispatch-tree root* — which belongs to the window wrapper, not to
    /// this view — so an unfocused editor never sees a key event at all. The
    /// root element tracks this handle, which both focuses the editor on the
    /// first frame and re-focuses it whenever the user clicks anywhere in it.
    focus_handle: Option<FocusHandle>,
    /// Linear element whose vertex is being dragged (`DragMode::Endpoint`).
    drag_target: Option<String>,
    /// World position where the current drag began.
    drag_start_world: Option<Point>,
    /// Selection-bounds anchor captured at drag start (for grid snapping).
    drag_anchor: Option<Point>,
    /// Total delta already applied during the current drag.
    drag_accum: Point,
    space_pan: bool,
    /// Screen position of an open context menu.
    context_menu: Option<Point>,
    /// Element — or group — the unlock bubble currently points at
    /// (upstream `appState.activeLockedId`).
    active_locked_id: Option<String>,
    /// File menu popover visibility.
    show_menu: bool,
    /// "More tools" popover visibility (the vertical-dots button).
    show_more_tools: bool,
    /// Preferences second-level panel visibility (inside the file menu).
    show_prefs: bool,
    /// Language listbox visibility (the open state of the language select).
    show_lang: bool,
    /// Transient status line message.
    status: Option<String>,
    /// Live OS file drag over the window (upstream `FileDropOverlay`).
    file_drag: Option<FileDragState>,
    /// Transient tooltip beside the cursor (upstream `CursorHint`).
    cursor_hint: Option<CursorHintState>,
    /// Bottom-centre status toast (upstream `Toast`).
    toast: Option<ToastState>,
    /// Live colour-picking gesture (upstream `EyeDropper`).
    eye_dropper: Option<EyeDropperState>,
    /// Monotonic counter behind the transient overlays' auto-hide timers.
    overlay_nonce: u64,
    /// Wall-clock time of the last cursor hint, for the letter-shortcut
    /// cooldown (upstream `CursorHints.lastShownAt`).
    cursor_hint_shown_at: Option<std::time::Instant>,
    /// Command palette visibility, its filter text and the highlighted row.
    show_palette: bool,
    palette_query: String,
    palette_index: usize,
    /// Find on canvas visibility, its needle and the match cursor.
    show_find: bool,
    find_query: String,
    find_matches: Vec<String>,
    find_index: usize,
    /// Help / shortcuts dialog visibility.
    show_help: bool,
    /// Last known viewport size, so a palette action that needs one (Zoom to fit)
    /// does not have to be handed the window at click time.
    viewport: (f64, f64),
    image_sources: HashMap<String, Arc<RenderImage>>,
    pending_image_pick: bool,
    /// Set when Open is chosen, next to the image picker above and for the same
    /// reason: the palette's keyboard path runs through the context-free
    /// `handle_key`, so the request is recorded here and started from `render`,
    /// which has a `Context`.
    pending_scene_open: bool,
    /// The "Save to…" modal, and the file name its field shows.
    ///
    /// `project_name` mirrors upstream's `appState.name`: empty means "not named
    /// yet", and the field falls back to `Untitled-<timestamp>` exactly the way
    /// `App.getName()` does.
    show_save_dialog: bool,
    pending_save_dialog: bool,
    project_name: String,
    /// Set by the modal's button: the platform save dialog needs a `Context`,
    /// which the button's click has, but keeping one flag for both entry points
    /// means there is a single place that starts a save.
    pending_scene_save: bool,
    /// Which row's colour picker popup is open (upstream keeps one `ColorPicker`
    /// per row and its open state beside it).
    color_popup: Option<ColorTarget>,
    /// The ramp step each popup's grid draws, seeded from the colour in use.
    shade_stroke: usize,
    shade_background: usize,
    /// The hex field's buffer, and whether keystrokes are routed into it.
    hex_buffer: String,
    hex_focused: bool,
    /// The font-picker popup and its quick-search needle.
    font_popup: bool,
    font_query: String,
    /// The element-link dialog and its text field.
    show_link_dialog: bool,
    link_input: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            history: History::new(100),
            i18n: I18n::new(Language::detect()),
            drawing: false,
            drag_start: None,
            last_point: None,
            current_points: Vec::new(),
            panning: false,
            pan_start: None,
            pan_scroll: None,
            drag_mode: None,
            resize_origin: None,
            rotate_start_angle: 0.0,
            editing_text: None,
            text_edit_started: false,
            replace_on_type: false,
            marquee_start: None,
            marquee_current: None,
            laser_points: Vec::new(),
            erasing: false,
            pending_points: Vec::new(),
            cursor_world: None,
            cursor_mods: Modifiers::default(),
            cursor_screen: (0.0, 0.0),
            focus_handle: None,
            drag_target: None,
            drag_start_world: None,
            drag_anchor: None,
            drag_accum: Point::zero(),
            space_pan: false,
            context_menu: None,
            active_locked_id: None,
            show_menu: false,
            show_more_tools: false,
            show_prefs: false,
            show_lang: false,
            status: None,
            file_drag: None,
            cursor_hint: None,
            toast: None,
            eye_dropper: None,
            overlay_nonce: 0,
            cursor_hint_shown_at: None,
            show_palette: false,
            palette_query: String::new(),
            palette_index: 0,
            show_find: false,
            find_query: String::new(),
            find_matches: Vec::new(),
            find_index: 0,
            show_help: false,
            viewport: (800.0, 600.0),
            image_sources: HashMap::new(),
            pending_image_pick: false,
            pending_scene_open: false,
            show_save_dialog: false,
            pending_save_dialog: false,
            project_name: String::new(),
            pending_scene_save: false,
            color_popup: None,
            shade_stroke: crate::theme::DEFAULT_STROKE_SHADE,
            shade_background: crate::theme::DEFAULT_BACKGROUND_SHADE,
            hex_buffer: String::new(),
            hex_focused: false,
            font_popup: false,
            font_query: String::new(),
            show_link_dialog: false,
            link_input: String::new(),
        }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.cancel_pending();
        self.active_locked_id = None;
        self.document.tool = tool;
        if tool == Tool::Image {
            self.pending_image_pick = true;
        }
        self.drawing = false;
        self.drag_start = None;
        self.current_points.clear();
        self.drag_mode = None;
        self.marquee_start = None;
        self.laser_points.clear();
        if tool != Tool::Text {
            self.commit_text_edit();
        }
    }

    pub fn set_language(&mut self, language: Language) {
        self.i18n.set_language(language);
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.document.set_theme(theme);
    }

    pub fn add_demo_scene(&mut self) {
        use excalidraw_core::FillStyle;

        let blue = ElementOptions {
            stroke_color: "#1971c2".to_string(),
            ..Default::default()
        };
        self.document
            .scene
            .add(new_rectangle(80.0, 80.0, 200.0, 120.0, &blue));

        let red = ElementOptions {
            stroke_color: "#e03131".to_string(),
            ..Default::default()
        };
        self.document
            .scene
            .add(new_ellipse(360.0, 80.0, 140.0, 140.0, &red));

        let green = ElementOptions {
            stroke_color: "#2f9e44".to_string(),
            fill_style: FillStyle::Hachure,
            background_color: "#b2f2bb".to_string(),
            ..Default::default()
        };
        self.document
            .scene
            .add(new_diamond(560.0, 100.0, 120.0, 120.0, &green));

        let solid = ElementOptions {
            stroke_color: "#6741d9".to_string(),
            fill_style: FillStyle::Solid,
            background_color: "#d0bfff".to_string(),
            ..Default::default()
        };
        let mut rounded = new_rectangle(80.0, 460.0, 220.0, 100.0, &solid);
        if let excalidraw_core::element::Element::Rectangle { base } = &mut rounded {
            base.roundness = Some(excalidraw_core::types::Roundness {
                kind: excalidraw_core::types::RoundnessType::ProportionalRadius,
                value: Some(0.25),
            });
        }
        self.document.scene.add(rounded);

        // A sticky note and an arrow bound to the first rectangle, so the
        // binding machinery is visible in the demo. Fresh notes start empty,
        // but the canned showcase labels its own.
        let (note, mut note_text) =
            new_sticky_note_with_text(80.0, 620.0, 160.0, 140.0, &ElementOptions::default());
        if let excalidraw_core::element::Element::Text(t) = &mut note_text {
            t.text = "Note".to_string();
        }
        self.document.scene.add(note);
        self.document.scene.add(note_text);
        self.document.refresh_bindings();

        let rect_id = self.document.scene.elements[0].id().to_string();
        let opts = ElementOptions {
            stroke_color: "#e8590c".to_string(),
            ..Default::default()
        };
        let arrow = new_arrow(
            vec![Point::new(280.0, 140.0), Point::new(360.0, 150.0)],
            None,
            Some(Arrowhead::Arrow),
            &opts,
        );
        let arrow_id = arrow.id().to_string();
        self.document.scene.add(arrow);
        self.document.select(&arrow_id);
        let from = Point::new(360.0, 150.0);
        self.document
            .attach_linear_binding(&arrow_id, true, Some(&rect_id), from);
        self.document.clear_selection();

        self.document.scene.add(new_text(
            80.0,
            380.0,
            "Excalidraw",
            36.0,
            &ElementOptions::default(),
        ));
    }

    // ----- geometry helpers --------------------------------------------------

    fn screen_point(&self, x: f32, y: f32) -> Point {
        screen_to_world(
            Point::new(x as f64, y as f64),
            self.document.zoom,
            self.document.scroll,
        )
    }

    fn commit_text_edit(&mut self) {
        if let Some(id) = self.editing_text.take() {
            self.document.clear_selection();
            // A label edits through its container: upstream keeps the container
            // selected when the editor closes, not the text living inside it.
            let selected = match self.document.scene.get(&id) {
                Some(excalidraw_core::element::Element::Text(t)) => {
                    t.container_id.clone().unwrap_or(id)
                }
                _ => id,
            };
            self.document.select(&selected);
        }
        self.text_edit_started = false;
        self.replace_on_type = false;
    }

    fn begin_text_edit(&mut self, id: &str, is_new: bool) {
        self.editing_text = Some(id.to_string());
        self.text_edit_started = true;
        self.replace_on_type = is_new;
        self.cursor_world = None;
    }

    pub(crate) fn push_checkpoint(&mut self) {
        self.history.push(&self.document.scene);
    }

    /// Wire a GPUI listener that mutates editor state.
    ///
    /// `Context::listener` runs the handler but never invalidates the view, and
    /// this crate paints a hand-rolled canvas instead of a declarative tree, so
    /// a missing `notify` is invisible until something else happens to dirty
    /// the window. In practice that means a drag paints nothing until a hover
    /// change comes along — the editor feels like it has dropped the input.
    /// Funnelling every handler through here makes forgetting impossible; the
    /// `no_raw_context_listeners` test in `tests/source_hygiene.rs` keeps it
    /// that way.
    pub(crate) fn act<E, F>(cx: &Context<Self>, f: F) -> Handler<E>
    where
        E: ?Sized + 'static,
        F: Fn(&mut Self, &E, &mut Window, &mut Context<Self>) + 'static,
    {
        Box::new(cx.listener(move |this, event, window, cx| {
            f(this, event, window, cx);
            cx.notify();
        }))
    }

    /// Whether a pointer gesture is in flight.
    ///
    /// The canvas owns the whole window, but every island on top of it is
    /// `occlude`d, so a pointer that strays over the toolbar stops producing
    /// canvas events. Move/up listeners are therefore also registered on the
    /// root *with this guard*: they only forward while a gesture is live, so
    /// hovering the chrome never leaks into the canvas.
    pub fn is_interacting(&self) -> bool {
        self.panning
            || self.drawing
            || self.erasing
            || self.drag_mode.is_some()
            || self.marquee_start.is_some()
            || !self.laser_points.is_empty()
            || !self.pending_points.is_empty()
    }

    /// The shape currently being dragged out, as a real element.
    ///
    /// Excalidraw previews the exact geometry it is about to commit, so the
    /// preview is built with the same factory the commit path uses — the two
    /// can never drift apart.
    pub fn drag_preview(&self) -> Option<excalidraw_core::element::Element> {
        if !self.drawing {
            return None;
        }
        let start = self.drag_start?;
        let world = self.cursor_world?;
        let (x, y, w, h) = self.drag_box(start, world, self.cursor_mods.shift);
        let opts = self.current_style_options();
        Some(match self.document.tool {
            Tool::Rectangle => new_rectangle(x, y, w, h, &opts),
            Tool::Diamond => new_diamond(x, y, w, h, &opts),
            Tool::Ellipse => new_ellipse(x, y, w, h, &opts),
            Tool::Frame => new_frame(x, y, w, h, None, &opts),
            Tool::StickyNote => new_sticky_note_with_text(x, y, w, h, &opts).0,
            _ => return None,
        })
    }

    /// Points of an in-progress free-draw stroke, for the live ink preview.
    pub fn freedraw_preview(&self) -> Option<&[Point]> {
        if self.drawing && self.document.tool == Tool::FreeDraw && self.current_points.len() >= 2 {
            Some(&self.current_points)
        } else {
            None
        }
    }

    fn current_style_options(&self) -> ElementOptions {
        ElementOptions {
            stroke_color: self.document.style.stroke_color.clone(),
            background_color: self.document.style.background_color.clone(),
            fill_style: self.document.style.fill_style,
            stroke_width: self.document.style.stroke_width,
            stroke_style: self.document.style.stroke_style,
            roughness: self.document.style.roughness,
            opacity: self.document.style.opacity,
        }
    }

    /// Recompute a text element's bounds from its content (auto-resize).
    fn refresh_text_bounds(&mut self, id: &str) {
        // A container's label is laid out from the container's geometry
        // (`sync_container_text`), not from its own glyph run, so the
        // auto-resize below must not re-pin it to the text's width.
        let container_bound = matches!(
            self.document.scene.get(id),
            Some(excalidraw_core::element::Element::Text(t)) if t.container_id.is_some()
        );
        if container_bound {
            self.document.refresh_bindings();
            return;
        }
        if let Some(excalidraw_core::element::Element::Text(t)) = self.document.scene.get_mut(id) {
            let lines: Vec<&str> = t.text.split('\n').collect();
            let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
            t.base.width = (longest * t.font_size * 0.6).max(t.font_size * 0.6);
            t.base.height = (lines.len() as f64).max(1.0) * t.font_size * t.line_height;
        }
    }

    fn cancel_pending(&mut self) {
        self.pending_points.clear();
        self.cursor_world = None;
        self.context_menu = None;
    }

    fn element_angle(&self) -> f64 {
        let ids: Vec<String> = self.document.selected.iter().cloned().collect();
        for id in ids {
            if let Some(e) = self.document.scene.get(&id) {
                return e.base().angle;
            }
        }
        0.0
    }

    /// Index of a vertex handle under `world`, for a single selected line/arrow.
    fn hit_endpoint(&self, world: Point) -> Option<(String, usize)> {
        let (id, points) = self.document.selected_linear_points()?;
        let threshold = 8.0 / self.document.zoom;
        for (i, p) in points.iter().enumerate() {
            if world.distance(*p) <= threshold {
                return Some((id.clone(), i));
            }
        }
        None
    }

    fn hit_handle(&self, world: Point) -> Option<ResizeHandle> {
        let bounds = selection_bounds(&self.document.scene, &self.document.selected)?;
        let threshold = 8.0 / self.document.zoom;
        let sx = bounds.min_x;
        let sy = bounds.min_y;
        let ex = bounds.max_x;
        let ey = bounds.max_y;
        let cx = (sx + ex) / 2.0;
        let cy = (sy + ey) / 2.0;

        let rotate_origin = Point::new(cx, sy - 24.0 / self.document.zoom);
        if world.distance(rotate_origin) <= threshold {
            return Some(ResizeHandle::Rotate);
        }

        let handles = [
            (ResizeHandle::Nw, Point::new(sx, sy)),
            (ResizeHandle::N, Point::new(cx, sy)),
            (ResizeHandle::Ne, Point::new(ex, sy)),
            (ResizeHandle::E, Point::new(ex, cy)),
            (ResizeHandle::Se, Point::new(ex, ey)),
            (ResizeHandle::S, Point::new(cx, ey)),
            (ResizeHandle::Sw, Point::new(sx, ey)),
            (ResizeHandle::W, Point::new(sx, cy)),
        ];
        for (handle, p) in handles {
            if world.distance(p) <= threshold {
                return Some(handle);
            }
        }
        None
    }

    // ----- structural editing -------------------------------------------------

    /// Duplicate the selection in place (used by alt-drag).
    fn duplicate_in_place(&mut self) {
        let ids: Vec<String> = self.document.selected.iter().cloned().collect();
        let mut new_ids = Vec::new();
        for id in ids {
            let src = self.document.scene.get(&id).cloned();
            if let Some(mut copy) = src {
                let nid = new_id();
                copy.base_mut().id = nid.clone();
                self.document.scene.add(copy);
                new_ids.push(nid);
            }
        }
        if !new_ids.is_empty() {
            self.document.selected.clear();
            self.document.selected.extend(new_ids);
        }
    }

    /// Finish and commit an in-progress multi-point line/arrow.
    fn finish_pending_line(&mut self) {
        if self.pending_points.len() < 2 {
            self.pending_points.clear();
            self.cursor_world = None;
            return;
        }
        let pts = self.pending_points.clone();
        let opts = self.current_style_options();
        let is_line = self.document.tool == Tool::Line;
        self.push_checkpoint();
        let el = if is_line {
            new_line(pts.clone(), &opts)
        } else {
            new_arrow(pts.clone(), None, Some(Arrowhead::Arrow), &opts)
        };
        let id = el.id().to_string();
        self.document.scene.add(el);
        self.document.select(&id);

        let n = pts.len();
        let exclude = vec![id.clone()];
        // Preferences → Arrow binding. Off means arrows stay free-standing.
        if self.document.arrow_binding {
            let th = excalidraw_core::binding::BINDING_THRESHOLD / self.document.zoom;
            let start_target =
                excalidraw_core::binding::find_bindable(&self.document.scene, pts[0], th, &exclude);
            let end_target = excalidraw_core::binding::find_bindable(
                &self.document.scene,
                pts[n - 1],
                th,
                &exclude,
            );
            if let Some(t) = start_target {
                self.document
                    .attach_linear_binding(&id, true, Some(&t), pts[1]);
            }
            if let Some(t) = end_target {
                self.document
                    .attach_linear_binding(&id, false, Some(&t), pts[n - 2]);
            }
        }
        self.pending_points.clear();
        self.cursor_world = None;
    }

    // ----- mouse ------------------------------------------------------------

    pub fn on_canvas_mouse_down(&mut self, x: f32, y: f32, mods: Modifiers, click_count: usize) {
        let world = self.screen_point(x, y);
        self.context_menu = None;
        self.show_menu = false;
        self.show_prefs = false;
        self.show_lang = false;
        self.show_more_tools = false;
        // The hint would otherwise sit under the cursor while the user draws.
        self.cursor_hint = None;

        // A press anywhere other than the element the unlock bubble points at
        // dismisses it (upstream compares the hit against `activeLockedId` on
        // pointer-down for exactly this reason).
        if let Some(active) = self.active_locked_id.clone() {
            let hit_the_target = self
                .document
                .elements_at_including_locked(world, 5.0 / self.document.zoom)
                .iter()
                .any(|element| locked_subject_id(element) == active);
            if !hit_the_target {
                self.active_locked_id = None;
            }
        }

        // Space-drag pans regardless of the active tool.
        if self.space_pan {
            self.panning = true;
            self.pan_start = Some(Point::new(x as f64, y as f64));
            self.pan_scroll = Some(self.document.scroll);
            return;
        }

        // View mode — the canvas is read-only; drags pan instead of editing.
        if self.document.view_mode {
            self.panning = true;
            self.pan_start = Some(Point::new(x as f64, y as f64));
            self.pan_scroll = Some(self.document.scroll);
            return;
        }

        match self.document.tool {
            Tool::Hand => {
                self.panning = true;
                self.pan_start = Some(Point::new(x as f64, y as f64));
                self.pan_scroll = Some(self.document.scroll);
            }
            Tool::Selection => {
                if let Some((id, index)) = self.hit_endpoint(world) {
                    self.push_checkpoint();
                    self.drag_mode = Some(DragMode::Endpoint { index });
                    self.drag_target = Some(id);
                    return;
                }
                if let Some(handle) = self.hit_handle(world) {
                    self.push_checkpoint();
                    self.drag_mode = Some(match handle {
                        ResizeHandle::Rotate => {
                            let bounds =
                                selection_bounds(&self.document.scene, &self.document.selected)
                                    .unwrap_or(Bounds::new(0.0, 0.0, 0.0, 0.0));
                            let center = bounds.center();
                            self.rotate_start_angle = self.element_angle();
                            DragMode::Rotate {
                                center,
                                last: world,
                            }
                        }
                        h => {
                            let bounds =
                                selection_bounds(&self.document.scene, &self.document.selected)
                                    .unwrap_or(Bounds::new(0.0, 0.0, 0.0, 0.0));
                            self.resize_origin = Some(bounds);
                            DragMode::Resize {
                                handle: h,
                                pivot: bounds.center(),
                            }
                        }
                    });
                    return;
                }

                if let Some(id) = self.document.element_at(world, 5.0 / self.document.zoom) {
                    let already = self.document.selected.contains(&id);
                    if mods.shift && already {
                        self.document.selected.remove(&id);
                        return;
                    }
                    if mods.shift {
                        self.document.selected.insert(id.clone());
                    } else if !already {
                        self.document.select(&id);
                    }
                    if let Some(e) = self.document.scene.get(&id) {
                        if e.is_text() && already {
                            self.begin_text_edit(&id, false);
                            return;
                        }
                    }
                    // A container's label opens on the second click. Upstream
                    // resolves the hit container to its bound text before
                    // starting the editor, so a double-click anywhere on a
                    // sticky note edits its text instead of dragging the note.
                    if click_count >= 2 {
                        if let Some(text_id) = self.document.container_text_id(&id) {
                            self.begin_text_edit(&text_id, false);
                            return;
                        }
                    }
                    self.begin_drag(world, mods.alt);
                } else {
                    if !mods.shift {
                        self.document.clear_selection();
                    }
                    self.marquee_start = Some(world);
                    self.marquee_current = Some(world);
                }
            }
            Tool::FreeDraw => {
                self.drawing = true;
                self.current_points = vec![world];
                self.last_point = Some(world);
                self.cursor_world = Some(world);
            }
            Tool::Text => {
                // The text tool is the one place a container label is
                // reachable directly: clicking its glyphs edits the label,
                // while a click on bare container surface starts a new text
                // (upstream's `getTextElementAtPosition` behaves the same way).
                if let Some(id) = self
                    .document
                    .text_target_at(world, 5.0 / self.document.zoom)
                {
                    if let Some(e) = self.document.scene.get(&id) {
                        if e.is_text() {
                            self.document.select(&id);
                            self.begin_text_edit(&id, false);
                            return;
                        }
                    }
                }
                let opts = self.current_style_options();
                self.push_checkpoint();
                let el = new_text(
                    world.x,
                    world.y,
                    "Text",
                    self.document.style.font_size,
                    &opts,
                );
                let id = el.id().to_string();
                self.document.scene.add(el);
                self.document.select(&id);
                self.begin_text_edit(&id, true);
            }
            Tool::Image => {}
            Tool::Eraser => {
                self.push_checkpoint();
                self.erasing = true;
                self.erase_at(world);
            }
            Tool::Laser => {
                self.laser_points = vec![world];
            }
            Tool::Line | Tool::Arrow => {
                // Multi-point authoring: each click appends a vertex; the line
                // is committed by a double-click, by re-clicking the last
                // vertex, or with Enter.
                let same_as_last = self
                    .pending_points
                    .last()
                    .map(|p| p.distance(world) <= CLICK_THRESHOLD)
                    .unwrap_or(false);
                if click_count >= 2 || (same_as_last && self.pending_points.len() >= 2) {
                    self.finish_pending_line();
                    return;
                }
                if self.pending_points.is_empty() {
                    self.push_checkpoint();
                }
                self.pending_points.push(world);
                self.cursor_world = Some(world);
            }
            _ => {
                self.drawing = true;
                self.drag_start = Some(world);
                self.last_point = Some(world);
                self.current_points = vec![world];
                self.cursor_world = Some(world);
            }
        }
    }

    /// Decide whether a completed click points the unlock bubble at something.
    ///
    /// Mirrors upstream's pointer-up rule: if *any* element under the cursor is
    /// already selected the bubble clears (the click belonged to that
    /// selection), otherwise the topmost element decides — a locked one arms the
    /// bubble, an unlocked one leaves it empty.
    fn settle_active_locked(&mut self, world: Point) {
        let hits = self
            .document
            .elements_at_including_locked(world, 5.0 / self.document.zoom);
        let already_selected = hits
            .iter()
            .any(|element| self.document.selected.contains(element.id()));
        self.active_locked_id = if already_selected {
            None
        } else {
            hits.last()
                .filter(|element| element.base().locked)
                .map(|element| locked_subject_id(element))
        };
    }

    /// Unlock whatever the bubble points at, and select it — clicking the
    /// padlock is how a user says "now let me edit this".
    fn on_unlock_locked_target(&mut self) {
        let Some(id) = self.active_locked_id.clone() else {
            return;
        };
        let ids: Vec<String> = self
            .document
            .locked_target(&id)
            .iter()
            .map(|element| element.id().to_string())
            .collect();
        if ids.is_empty() {
            self.active_locked_id = None;
            return;
        }
        self.push_checkpoint();
        self.document.selected = ids.into_iter().collect();
        self.document.set_selected_locked(false);
        self.active_locked_id = None;
    }

    fn begin_drag(&mut self, world: Point, alt: bool) {
        self.push_checkpoint();
        if alt {
            self.duplicate_in_place();
        }
        self.drag_mode = Some(DragMode::Move { last: world });
        self.drag_start_world = Some(world);
        self.drag_anchor = selection_bounds(&self.document.scene, &self.document.selected)
            .map(|b| Point::new(b.min_x, b.min_y));
        self.drag_accum = Point::zero();
    }

    fn erase_at(&mut self, world: Point) {
        let threshold = 8.0 / self.document.zoom;
        if let Some(id) = self.document.element_at(world, threshold) {
            self.document.scene.remove(&id);
            self.document.selected.remove(&id);
            // A container takes its label with it. The label is not hit-tested
            // on its own, so erasing the note would otherwise leave an orphan
            // text floating where the note used to be.
            if let Some(text_id) = self.document.container_text_id(&id) {
                self.document.scene.remove(&text_id);
                self.document.selected.remove(&text_id);
            }
        }
    }

    pub fn on_canvas_mouse_move(&mut self, x: f32, y: f32, mods: Modifiers) {
        self.cursor_mods = mods;
        self.cursor_screen = (x as f64, y as f64);
        if self.panning {
            if let (Some(start), Some(scroll)) = (self.pan_start, self.pan_scroll) {
                self.document.scroll = Point::new(
                    scroll.x + (x as f64 - start.x),
                    scroll.y + (y as f64 - start.y),
                );
            }
            return;
        }
        let world = self.screen_point(x, y);
        self.cursor_world = Some(world);

        if self.erasing {
            self.erase_at(world);
            return;
        }

        if let Some(mode) = self.drag_mode {
            match mode {
                DragMode::Move { .. } => self.apply_move(world, mods),
                DragMode::Resize { handle, pivot } => {
                    self.apply_resize(handle, pivot, world, mods.shift)
                }
                DragMode::Rotate { center, last } => {
                    let prev = (last.y - center.y).atan2(last.x - center.x);
                    let now = (world.y - center.y).atan2(world.x - center.x);
                    let mut delta = now - prev;
                    if mods.shift {
                        let step = 15.0_f64.to_radians();
                        delta = (delta / step).round() * step;
                    }
                    self.document.rotate_selected(delta, center);
                    self.drag_mode = Some(DragMode::Rotate {
                        center,
                        last: world,
                    });
                }
                DragMode::Endpoint { index } => {
                    if let Some(id) = self.drag_target.clone() {
                        self.document
                            .move_linear_endpoint(&id, index, self.document.snap(world));
                    }
                }
            }
            return;
        }

        if let Some(start) = self.marquee_start {
            self.marquee_current = Some(world);
            let b = Bounds::new(
                start.x.min(world.x),
                start.y.min(world.y),
                start.x.max(world.x),
                start.y.max(world.y),
            );
            let ids: Vec<String> = self
                .document
                .scene
                .non_deleted()
                .filter(|e| !e.base().locked)
                // A label is never box-selected: it belongs to its container,
                // which the same marquee already picks up (upstream's
                // `shouldIgnoreElementFromSelection`).
                .filter(|e| !crate::state::is_bound_text(e))
                .filter(|e| {
                    let eb = excalidraw_core::bounds::element_bounds(e);
                    // Preferences "Select on": Wrap (upstream default) requires the
                    // marquee to fully contain the element; Overlap takes
                    // anything it touches.
                    if self.document.select_wrap {
                        b.min_x <= eb.min_x
                            && b.min_y <= eb.min_y
                            && b.max_x >= eb.max_x
                            && b.max_y >= eb.max_y
                    } else {
                        b.intersects(&eb)
                    }
                })
                .map(|e| e.id().to_string())
                .collect();
            self.document.select_many(ids);
            return;
        }

        if self.document.tool == Tool::Laser {
            if !self.laser_points.is_empty() {
                self.laser_points.push(world);
            }
            return;
        }

        if self.drawing {
            self.current_points.push(world);
            self.last_point = Some(world);
        }
    }

    /// Apply a drag delta with optional grid snapping and shift axis-locking.
    fn apply_move(&mut self, world: Point, mods: Modifiers) {
        let (Some(start), Some(anchor)) = (self.drag_start_world, self.drag_anchor) else {
            return;
        };
        let mut raw = Point::new(world.x - start.x, world.y - start.y);
        if mods.shift {
            if raw.x.abs() >= raw.y.abs() {
                raw.y = 0.0;
            } else {
                raw.x = 0.0;
            }
        }
        let target = Point::new(anchor.x + raw.x, anchor.y + raw.y);
        let snapped = self.document.snap(target);
        let delta = Point::new(snapped.x - anchor.x, snapped.y - anchor.y);
        let inc = Point::new(delta.x - self.drag_accum.x, delta.y - self.drag_accum.y);
        if inc.x != 0.0 || inc.y != 0.0 {
            self.document.move_selected(inc.x, inc.y);
        }
        self.drag_accum = delta;
    }

    fn apply_resize(&mut self, handle: ResizeHandle, _pivot: Point, world: Point, shift: bool) {
        let Some(origin) = self.resize_origin else {
            return;
        };
        let (sx, sy) = (origin.min_x, origin.min_y);
        let (ex, ey) = (origin.max_x, origin.max_y);
        let w = origin.width();
        let h = origin.height();

        let (anchor_x, anchor_y) = match handle {
            ResizeHandle::Nw | ResizeHandle::N => (ex, ey),
            ResizeHandle::Ne | ResizeHandle::E => (sx, ey),
            ResizeHandle::Se | ResizeHandle::S => (sx, sy),
            ResizeHandle::Sw | ResizeHandle::W => (ex, sy),
            ResizeHandle::Rotate => return,
        };

        let mut new_w = match handle {
            ResizeHandle::Nw | ResizeHandle::Sw | ResizeHandle::W => (anchor_x - world.x).abs(),
            ResizeHandle::Ne | ResizeHandle::Se | ResizeHandle::E => (world.x - anchor_x).abs(),
            _ => w,
        };
        let mut new_h = match handle {
            ResizeHandle::Nw | ResizeHandle::Ne | ResizeHandle::N => (anchor_y - world.y).abs(),
            ResizeHandle::Sw | ResizeHandle::Se | ResizeHandle::S => (world.y - anchor_y).abs(),
            _ => h,
        };

        if shift && w > 0.0 && h > 0.0 {
            // Uniform scaling proportionally to the dominant axis.
            let ratio = w / h;
            if new_w / w >= new_h / h {
                new_h = new_w / ratio;
            } else {
                new_w = new_h * ratio;
            }
        }

        let scale_x = if w > 0.0 { new_w / w } else { 1.0 };
        let scale_y = if h > 0.0 { new_h / h } else { 1.0 };

        if scale_x > 0.0 && scale_y > 0.0 && (scale_x != 1.0 || scale_y != 1.0) {
            let anchor = Point::new(anchor_x, anchor_y);
            self.document.resize_selected(scale_x, scale_y, anchor);
            // `scale_x`/`scale_y` are absolute: they are measured against the
            // pre-drag origin but applied to the element's current geometry.
            // Re-basing the origin on the geometry we just produced makes the
            // next event's scale incremental. Reusing the stale origin would
            // multiply the element on every move (runaway growth, then freeze).
            self.resize_origin = selection_bounds(&self.document.scene, &self.document.selected);
        }
    }

    pub fn on_canvas_mouse_up(&mut self, x: f32, y: f32, mods: Modifiers) {
        let world = self.screen_point(x, y);
        // Upstream arms the unlock bubble on pointer-up, and only for a press
        // that neither panned, erased, nor dragged a marquee — a real marquee
        // is a selection gesture, never a "let me unlock that" one.
        if !self.panning && !self.erasing && self.document.tool == Tool::Selection {
            let marquee_dragged = self
                .marquee_start
                .map(|start| start.distance(world) > CLICK_THRESHOLD)
                .unwrap_or(false);
            if !marquee_dragged {
                self.settle_active_locked(world);
            }
        }
        if self.panning {
            self.panning = false;
            self.pan_start = None;
            self.pan_scroll = None;
            return;
        }

        if self.erasing {
            self.erasing = false;
            return;
        }

        if self.drag_mode.is_some() {
            self.drag_mode = None;
            self.resize_origin = None;
            self.drag_start_world = None;
            self.drag_anchor = None;
            self.drag_accum = Point::zero();
            self.drag_target = None;
            return;
        }

        if self.marquee_start.is_some() {
            self.marquee_start = None;
            self.marquee_current = None;
            return;
        }

        if self.document.tool == Tool::Laser {
            self.laser_points.clear();
            return;
        }

        // Multi-point line/arrow: a drag from the first click commits two points.
        if matches!(self.document.tool, Tool::Line | Tool::Arrow) {
            if let Some(start) = self.pending_points.first().copied() {
                if self.pending_points.len() == 1 && start.distance(world) > CLICK_THRESHOLD {
                    let end = if mods.shift {
                        excalidraw_core::binding::constrain_angle(start, world, 15.0)
                    } else {
                        self.document.snap(world)
                    };
                    self.pending_points.push(end);
                    self.finish_pending_line();
                }
            }
            return;
        }

        if !self.drawing {
            return;
        }
        self.drawing = false;

        let opts = self.current_style_options();
        self.push_checkpoint();

        let start = self.drag_start;
        match self.document.tool {
            Tool::FreeDraw => {
                let pts = streamline_points(&self.current_points, 0.35);
                self.document.scene.add(new_freedraw(pts, &opts));
            }
            Tool::Rectangle | Tool::Diamond | Tool::Ellipse | Tool::Frame => {
                if let Some(start) = start {
                    let (x, y, w, h) = self.drag_box(start, world, mods.shift);
                    let el = match self.document.tool {
                        Tool::Rectangle => new_rectangle(x, y, w, h, &opts),
                        Tool::Diamond => new_diamond(x, y, w, h, &opts),
                        Tool::Ellipse => new_ellipse(x, y, w, h, &opts),
                        _ => new_frame(x, y, w, h, None, &opts),
                    };
                    self.document.scene.add(el);
                }
            }
            Tool::StickyNote => {
                if let Some(start) = start {
                    let (x, y, w, h) = self.drag_box(start, world, mods.shift);
                    let (note, text) = new_sticky_note_with_text(x, y, w, h, &opts);
                    self.document.scene.add(note);
                    self.document.scene.add(text);
                    self.document.refresh_bindings();
                }
            }
            _ => {}
        }

        self.drag_start = None;
        self.current_points.clear();
        self.last_point = None;

        // Upstream behaviour: once a shape is committed the padlock in the
        // toolbar decides whether the tool stays armed or hands control back to
        // the selection tool. Excalidraw ships with it *off*, so drawing one
        // rectangle leaves you able to select what you just drew.
        if !self.document.tool_locked
            && matches!(
                self.document.tool,
                Tool::Rectangle
                    | Tool::Diamond
                    | Tool::Ellipse
                    | Tool::FreeDraw
                    | Tool::Frame
                    | Tool::StickyNote
            )
        {
            self.set_tool(Tool::Selection);
        }
    }

    /// Compute a normalised drag box, honouring shift-square and snapping, and
    /// falling back to a default size for a bare click.
    fn drag_box(&self, start: Point, world: Point, shift: bool) -> (f64, f64, f64, f64) {
        let mut end = self.document.snap(world);
        if shift {
            end = excalidraw_core::binding::constrain_square(start, end);
        }
        let mut w = (end.x - start.x).abs();
        let mut h = (end.y - start.y).abs();
        if w < CLICK_THRESHOLD && h < CLICK_THRESHOLD {
            w = CLICK_DEFAULT_SIZE;
            h = CLICK_DEFAULT_SIZE;
        }
        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        (x, y, w.max(1.0), h.max(1.0))
    }

    // ----- actions -----------------------------------------------------------

    // The properties island lives in a sibling module, so it reaches these verbs
    // through narrow wrappers rather than poking `document` directly and
    // forgetting the history checkpoint. Each one also carries the guard the
    // bare `document` call lacks (e.g. align needs two elements).

    pub(crate) fn delete_selected(&mut self) {
        self.on_delete();
    }

    pub(crate) fn duplicate_selected(&mut self) {
        self.on_duplicate();
    }

    pub(crate) fn group_selected(&mut self) {
        self.on_group();
    }

    pub(crate) fn align_selected(&mut self, align: excalidraw_core::operations::Align) {
        self.on_align(align);
    }

    pub(crate) fn distribute_selected(&mut self, dir: excalidraw_core::operations::Distribute) {
        self.on_distribute(dir);
    }

    /// Lock or unlock the selection, as the padlock in the panel does.
    pub(crate) fn lock_selected(&mut self, locked: bool) {
        if locked {
            self.on_lock();
        } else {
            self.on_unlock();
        }
    }

    pub fn on_undo(&mut self) {
        if self.history.undo(&mut self.document.scene) {
            self.document.clear_selection();
            self.document.refresh_bindings();
        }
    }

    fn on_redo(&mut self) {
        if self.history.redo(&mut self.document.scene) {
            self.document.clear_selection();
            self.document.refresh_bindings();
        }
    }

    fn on_delete(&mut self) {
        if self.document.selected.is_empty() {
            return;
        }
        self.push_checkpoint();
        self.document.delete_selected();
        self.active_locked_id = None;
    }

    fn on_duplicate(&mut self) {
        if self.document.selected.is_empty() {
            return;
        }
        self.push_checkpoint();
        self.document.duplicate_selected();
    }

    fn on_copy(&mut self) {
        self.document.copy_selected();
        self.status = Some(format!(
            "{} {}",
            self.i18n.t("ctx.copy"),
            self.document.clipboard.len()
        ));
    }

    fn on_cut(&mut self) {
        if self.document.selected.is_empty() {
            return;
        }
        self.push_checkpoint();
        self.document.cut_selected();
    }

    fn on_paste(&mut self) {
        if !self.document.has_clipboard() {
            return;
        }
        self.push_checkpoint();
        self.document.paste(20.0, 20.0);
    }

    pub fn sync_image_sources(&mut self) {
        let mut missing: Vec<(String, String, String)> = Vec::new();
        for (id, file) in &self.document.files {
            if !self.image_sources.contains_key(id) {
                missing.push((id.clone(), file.mime_type.clone(), file.data_url.clone()));
            }
        }
        for (id, mime, url) in missing {
            if let Some(bytes) = crate::bitmap::bytes_from_data_url(&url) {
                if let Some(render) = crate::bitmap::decode_render_image(&bytes, &mime) {
                    self.image_sources.insert(id, render);
                }
            }
        }
        let known: Vec<String> = self.document.files.keys().cloned().collect();
        self.image_sources.retain(|id, _| known.contains(id));
    }

    pub fn register_image_file(&mut self, bytes: &[u8], mime: &str) -> String {
        let id = format!("file-{}", uuid::Uuid::new_v4());
        let mut file = excalidraw_core::scene::BinaryFileData::new(
            id.clone(),
            mime,
            crate::bitmap::data_url(mime, bytes),
        );
        file.created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        self.document.files.insert(id.clone(), file);
        if let Some(render) = crate::bitmap::decode_render_image(bytes, mime) {
            self.image_sources.insert(id.clone(), render);
        }
        id
    }

    pub fn image_dimensions(&self, file_id: &str) -> Option<(f64, f64)> {
        self.image_sources
            .get(file_id)
            .map(|r| crate::bitmap::render_image_size(r))
    }

    fn default_insert_point(&self, width: f64, height: f64) -> Point {
        let (vw, vh) = self.viewport;
        let center = screen_to_world(
            Point::new(vw / 2.0, vh / 2.0),
            self.document.zoom,
            self.document.scroll,
        );
        Point::new(center.x - width / 2.0, center.y - height / 2.0)
    }

    pub fn insert_image_bytes(
        &mut self,
        bytes: &[u8],
        mime: &str,
        at: Option<Point>,
    ) -> Option<String> {
        if bytes.is_empty() {
            return None;
        }
        let file_id = self.register_image_file(bytes, mime);
        let (width, height) = self
            .image_dimensions(&file_id)
            .map(|(w, h)| crate::bitmap::fit_size(w, h))
            .unwrap_or((200.0, 150.0));
        let origin = at.unwrap_or_else(|| self.default_insert_point(width, height));
        let opts = self.current_style_options();
        self.push_checkpoint();
        let mut element = new_image(origin.x, origin.y, width, height, Some(file_id), &opts);
        if let Element::Image(image) = &mut element {
            image.status = excalidraw_core::element::ImageStatus::Saved;
        }
        let id = element.id().to_string();
        self.document.scene.add(element);
        self.document.select(&id);
        self.status = Some(format!(
            "{} {}x{}",
            self.i18n.t("toolbar.image"),
            width.round(),
            height.round()
        ));
        Some(id)
    }

    pub fn paste_image_from_clipboard(&mut self, cx: &App) -> bool {
        let Some(item) = cx.read_from_clipboard() else {
            return false;
        };
        for entry in &item.entries {
            if let ClipboardEntry::Image(image) = entry {
                let mime = crate::bitmap::detect_mime(&image.bytes, None);
                return self.insert_image_bytes(&image.bytes, &mime, None).is_some();
            }
        }
        false
    }

    /// Paste plain text from the system clipboard.
    ///
    /// Upstream's paste takes two shapes, and both are needed here.
    ///
    /// While a text element is being edited, upstream is a real `<textarea>`,
    /// so the browser pastes at the caret and the text simply joins the
    /// element. This editor has no caret — it appends — so the pasted text
    /// joins at the end, which is where typing already puts it.
    ///
    /// On the canvas, pasted text becomes *one text element per line*, stacked
    /// downward from the pointer, because upstream refuses to put a multi-line
    /// blob into a single auto-resizing element (`App.clipboard.ts`,
    /// `addTextFromPaste`). A blank line becomes a paragraph gap rather than an
    /// empty element — and only when the line above it had text, so a run of
    /// blank lines collapses into one gap.
    ///
    /// Returns whether anything was pasted, so a caller can fall through to the
    /// next clipboard flavour when the clipboard holds no text.
    pub fn paste_text_from_clipboard(&mut self, cx: &App) -> bool {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return false;
        };
        if text.is_empty() {
            return false;
        }
        // A Windows clipboard hands over CRLF; the text model only knows LF.
        let text = text.replace("\r\n", "\n").replace('\r', "\n");

        if let Some(id) = self.editing_text.clone() {
            if let Some(excalidraw_core::element::Element::Text(t)) =
                self.document.scene.get_mut(&id)
            {
                if self.replace_on_type {
                    t.text.clear();
                }
                t.text.push_str(&text);
            }
            self.replace_on_type = false;
            self.refresh_text_bounds(&id);
            return true;
        }

        self.paste_text_as_elements(&text);
        true
    }

    /// Split pasted text into the text elements upstream would create.
    fn paste_text_as_elements(&mut self, text: &str) {
        // Upstream pastes at the pointer's last position. The canvas records
        // every move into `cursor_world`, so that is the pointer; with no move
        // yet it falls back to the centre of the view, like image drops do.
        let origin = self.cursor_world.unwrap_or_else(|| {
            let (vw, vh) = self.viewport;
            screen_to_world(
                Point::new(vw / 2.0, vh / 2.0),
                self.document.zoom,
                self.document.scroll,
            )
        });
        let options = self.current_style_options();
        let font_size = self.document.style.font_size;
        let mut ids = Vec::new();
        let mut y = origin.y;
        let mut has_text_above = false;
        for line in text.split('\n') {
            let line = line.trim();
            if line.is_empty() {
                if has_text_above {
                    y += font_size * 1.25 + PASTE_LINE_GAP;
                    has_text_above = false;
                }
                continue;
            }
            let element = new_text(0.0, 0.0, line, font_size, &options);
            let id = element.id().to_string();
            self.document.scene.add(element);
            // Recompute through the same path typing uses, so a pasted element
            // measures exactly like one that was typed.
            self.refresh_text_bounds(&id);
            let Some(bounds) = self.document.scene.get(&id).map(|e| {
                let base = e.base();
                (base.width, base.height)
            }) else {
                continue;
            };
            let (w, h) = bounds;
            // Upstream centres each pasted line on the paste column.
            if let Some(element) = self.document.scene.get_mut(&id) {
                let base = element.base_mut();
                base.x = origin.x - w / 2.0;
                base.y = y - h / 2.0;
            }
            ids.push(id);
            y += h + PASTE_LINE_GAP;
            has_text_above = true;
        }
        if !ids.is_empty() {
            self.document.select_many(ids);
        }
    }

    pub fn insert_image_paths(&mut self, paths: &[std::path::PathBuf]) -> usize {
        let mut inserted = 0;
        for path in paths {
            if path.is_dir() {
                continue;
            }
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let mime = crate::bitmap::detect_mime(&bytes, path.to_str());
            let offset = 24.0 * inserted as f64;
            let at = {
                let (vw, vh) = self.viewport;
                let center = screen_to_world(
                    Point::new(vw / 2.0 + offset, vh / 2.0 + offset),
                    self.document.zoom,
                    self.document.scroll,
                );
                Some(center)
            };
            if self.insert_image_bytes(&bytes, &mime, at).is_some() {
                inserted += 1;
            }
        }
        inserted
    }

    fn open_image_picker(&mut self, cx: &mut Context<Self>) {
        let prompt = SharedString::from(self.i18n.t("toolbar.image"));
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(prompt),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
                let _ = this.update(cx, |editor, _| {
                    editor.insert_image_paths(&paths);
                });
            }
            let _ = this.update(cx, |editor, _| {
                if editor.document.tool == Tool::Image {
                    editor.set_tool(Tool::Selection);
                }
            });
        })
        .detach();
    }

    fn on_zoom_in(&mut self) {
        self.document.zoom = (self.document.zoom * 1.2).min(8.0);
    }

    fn on_zoom_out(&mut self) {
        self.document.zoom = (self.document.zoom / 1.2).max(0.05);
    }

    fn on_reset_zoom(&mut self) {
        self.document.zoom = 1.0;
        self.document.scroll = Point::zero();
    }

    fn on_toggle_theme(&mut self) {
        let next = match self.document.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        self.document.theme = next;
    }

    fn on_toggle_grid(&mut self) {
        self.document.show_grid = !self.document.show_grid;
    }

    fn on_toggle_library(&mut self) {
        self.document.show_library = !self.document.show_library;
    }

    fn on_group(&mut self) {
        if self.document.selected.len() < 2 {
            return;
        }
        self.push_checkpoint();
        self.document.group_selected();
    }

    fn on_ungroup(&mut self) {
        if self.document.selected.is_empty() {
            return;
        }
        self.push_checkpoint();
        self.document.ungroup_selected();
    }

    fn on_lock(&mut self) {
        self.push_checkpoint();
        self.document.set_selected_locked(true);
        self.active_locked_id = None;
    }

    fn on_unlock(&mut self) {
        self.push_checkpoint();
        self.document.set_selected_locked(false);
        self.active_locked_id = None;
    }

    /// Re-type the selected element, as the popup above it does.
    ///
    /// Re-picking the type it already has is a no-op upstream, so it must not
    /// leave an undo step behind either.
    fn on_convert_target(&mut self, target: ConvertTarget) {
        let changes = self
            .document
            .lone_selected_element()
            .map(|e| {
                e.convert_target() != Some(target) && e.convertible_targets().contains(&target)
            })
            .unwrap_or(false);
        if !changes {
            return;
        }
        self.push_checkpoint();
        self.document.convert_selected_to(target);
    }

    fn on_align(&mut self, align: excalidraw_core::operations::Align) {
        if self.document.selected.len() < 2 {
            return;
        }
        self.push_checkpoint();
        self.document.align_selected(align);
    }

    fn on_distribute(&mut self, dir: excalidraw_core::operations::Distribute) {
        if self.document.selected.len() < 3 {
            return;
        }
        self.push_checkpoint();
        self.document.distribute_selected(dir);
    }

    pub fn selected_count(&self) -> usize {
        self.document.selected.len()
    }

    /// Whether the hamburger dropdown is open.
    ///
    /// Exposed so integration tests can assert the chrome's popovers really
    /// toggle; the field itself stays private.
    pub fn is_menu_open(&self) -> bool {
        self.show_menu
    }

    /// Open or close the hamburger menu, closing any other popover with it —
    /// two dropdowns overlapping is never something a user asked for.
    pub fn set_menu_open(&mut self, open: bool) {
        self.show_menu = open;
        if open {
            self.show_more_tools = false;
        } else {
            self.show_prefs = false;
            self.show_lang = false;
        }
        // The canvas-background picker hangs off a swatch inside this menu, so it
        // cannot outlive it — a stale open state would make the next click on
        // that swatch close instead of open the picker.
        if self.color_popup == Some(ColorTarget::CanvasBackground) {
            self.close_color_popup();
        }
    }

    /// Whether the language select in the hamburger menu is expanded.
    pub fn is_language_open(&self) -> bool {
        self.show_lang
    }

    /// The tallest the hamburger menu may grow before it scrolls.
    ///
    /// `.dropdown-menu-container` caps itself at
    /// `calc(100svh - var(--editor-container-padding) * 2 - 2.25rem)`, i.e. the
    /// window less the 16px inset top and bottom and the 36px hamburger chip it
    /// hangs under. Without the cap the menu runs off a short window and the
    /// sections past the fold — the language select among them — cannot be
    /// reached at all.
    fn menu_max_height(&self) -> f32 {
        (self.viewport.1 as f32 - size::CONTAINER_PADDING * 2.0 - size::BUTTON_LG).max(0.0)
    }

    /// Open or close the "more tools" popover, closing the hamburger with it.
    pub fn set_more_tools_open(&mut self, open: bool) {
        self.show_more_tools = open;
        if open {
            self.show_menu = false;
        }
    }

    /// Whether the "More tools" popover is open.
    pub fn is_more_tools_open(&self) -> bool {
        self.show_more_tools
    }

    // ----- overlays: command palette, find, help -----------------------------

    pub fn is_palette_open(&self) -> bool {
        self.show_palette
    }

    pub fn is_find_open(&self) -> bool {
        self.show_find
    }

    pub fn is_help_open(&self) -> bool {
        self.show_help
    }

    pub fn palette_query(&self) -> &str {
        &self.palette_query
    }

    pub fn palette_index(&self) -> usize {
        self.palette_index
    }

    pub fn set_palette_query(&mut self, query: impl Into<String>) {
        self.palette_query = query.into();
        self.palette_index = 0;
    }

    /// Open command palette, resetting the filter so it always starts from the full list.
    pub fn open_palette(&mut self) {
        self.show_palette = true;
        self.palette_query.clear();
        self.palette_index = 0;
        self.show_menu = false;
        self.show_more_tools = false;
    }

    pub fn toggle_palette(&mut self) {
        if self.show_palette {
            self.show_palette = false;
        } else {
            self.open_palette();
        }
    }

    /// The commands matching the current filter, as (id, i18n label key,
    /// shortcut, icon) tuples. Exposed so tests can assert the filter and the
    /// palette's translation coverage without reaching into the view.
    pub fn palette_results(
        &self,
    ) -> Vec<(
        &'static str,
        &'static str,
        Option<&'static str>,
        &'static str,
    )> {
        let needle = self.palette_query.to_lowercase();
        PALETTE_COMMANDS
            .iter()
            .filter(|c| self.palette_matches(c, &needle))
            .map(|c| (c.id, c.label, c.shortcut, c.icon))
            .collect()
    }

    /// Case-insensitive match against the *translated* label, so the palette is
    /// searchable in the user's own language — typing the localised "Undo" finds it.
    fn palette_matches(&self, command: &PaletteCommand, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        self.i18n.t(command.label).to_lowercase().contains(needle) || command.id.contains(needle)
    }

    /// Every command's label key, across the whole table. Used by the i18n test.
    pub fn palette_labels() -> Vec<&'static str> {
        PALETTE_COMMANDS.iter().map(|c| c.label).collect()
    }

    /// Every key the Help dialog resolves — its section headings and the label
    /// beside each shortcut. Table-driven, so no literal lookup pins them.
    pub fn help_labels() -> Vec<&'static str> {
        let mut keys = Vec::new();
        for &(section, rows) in HELP_SHORTCUTS {
            keys.push(section);
            for &(label, _) in rows {
                keys.push(label);
            }
        }
        keys
    }

    fn run_palette_selection(&mut self) {
        let needle = self.palette_query.to_lowercase();
        let picked = PALETTE_COMMANDS
            .iter()
            .filter(|c| self.palette_matches(c, &needle))
            .nth(self.palette_index)
            .map(|c| c.action);
        self.show_palette = false;
        if let Some(action) = picked {
            action.run(self);
        }
    }

    pub fn open_find(&mut self) {
        self.show_find = true;
        self.show_menu = false;
        self.show_more_tools = false;
        self.recompute_find();
    }

    pub fn find_query(&self) -> &str {
        &self.find_query
    }

    pub fn find_match_count(&self) -> usize {
        self.find_matches.len()
    }

    pub fn find_index(&self) -> usize {
        self.find_index
    }

    pub fn set_find_query(&mut self, query: impl Into<String>) {
        self.find_query = query.into();
        self.recompute_find();
    }

    fn recompute_find(&mut self) {
        let needle = self.find_query.trim().to_lowercase();
        self.find_matches = if needle.is_empty() {
            Vec::new()
        } else {
            self.document
                .scene
                .non_deleted()
                .filter(|e| {
                    element_search_text(e).is_some_and(|s| s.to_lowercase().contains(&needle))
                })
                .map(|e| e.id().to_string())
                .collect()
        };
        self.find_index = 0;
        self.focus_find_match();
    }

    fn step_find(&mut self, forward: bool) {
        if self.find_matches.is_empty() {
            return;
        }
        let len = self.find_matches.len();
        self.find_index = if forward {
            (self.find_index + 1) % len
        } else {
            (self.find_index + len - 1) % len
        };
        self.focus_find_match();
    }

    /// Select the current match and pan it into view. Selecting is also what
    /// the canvas paints as the highlight, so a match is visible without
    /// threading a second overlay through the paint pass.
    fn focus_find_match(&mut self) {
        let Some(id) = self.find_matches.get(self.find_index).cloned() else {
            return;
        };
        self.document.select(&id);
        let Some(element) = self.document.scene.get(&id) else {
            return;
        };
        let b = excalidraw_core::bounds::element_bounds(element);
        let (vw, vh) = self.viewport;
        self.document.scroll = Point::new(
            vw / 2.0 - (b.min_x + b.width() / 2.0) * self.document.zoom,
            vh / 2.0 - (b.min_y + b.height() / 2.0) * self.document.zoom,
        );
    }

    pub fn open_help(&mut self) {
        self.show_help = true;
        self.show_menu = false;
        self.show_more_tools = false;
    }

    /// Verification hook: boots the editor straight into the hamburger menu
    /// with the Preferences submenu expanded, so the screenshot loop can capture
    /// that state without input injection. Not part of the public API surface.
    #[doc(hidden)]
    pub fn debug_open_prefs(&mut self) {
        self.set_menu_open(true);
        self.show_prefs = true;
    }

    /// Number of vertices committed so far for an in-progress line/arrow.
    pub fn pending_point_count(&self) -> usize {
        self.pending_points.len()
    }

    /// Whether an Open is waiting for the next frame to start its file picker.
    pub fn scene_open_pending(&self) -> bool {
        self.pending_scene_open
    }

    /// Whether the "Save to…" modal is up.
    pub fn save_dialog_open(&self) -> bool {
        self.show_save_dialog
    }

    /// Whether a save is waiting for the next frame to start its file picker.
    pub fn scene_save_pending(&self) -> bool {
        self.pending_scene_save
    }

    /// Whether a "Save to…" is waiting for the next frame to raise the modal.
    pub fn save_dialog_pending(&self) -> bool {
        self.pending_save_dialog
    }

    /// What the modal's file-name field shows, i.e. `App.getName()`.
    pub fn project_name_value(&self) -> String {
        if self.project_name.is_empty() {
            self.untitled_name()
        } else {
            self.project_name.clone()
        }
    }

    /// The status line's current text, if any.
    pub fn status_text(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Id of the text element currently being edited, if any.
    pub fn editing_text_id(&self) -> Option<&str> {
        self.editing_text.as_deref()
    }

    pub fn has_text_selected(&self) -> bool {
        self.document
            .selected_elements()
            .iter()
            .any(|e| e.is_text())
    }

    // ----- file menu ---------------------------------------------------------

    /// The colour an export paints behind the artwork.
    ///
    /// Same rule as the live canvas: an explicitly picked tint is exported
    /// as-is; the default white falls through to the theme's surface.
    pub fn export_background(&self) -> &str {
        if self.document.view_background_color == "#ffffff" {
            match self.document.theme {
                Theme::Light => "#ffffff",
                Theme::Dark => "#121212",
            }
        } else {
            &self.document.view_background_color
        }
    }

    pub fn export_svg_string(&self) -> String {
        crate::export::export_svg_with_files(
            &self.document.scene,
            self.export_background(),
            &self.document.files,
        )
    }

    fn app_state(&self) -> AppState {
        AppState {
            theme: self.document.theme,
            scroll_x: self.document.scroll.x,
            scroll_y: self.document.scroll.y,
            zoom: excalidraw_core::scene::Zoom {
                value: self.document.zoom,
            },
            files: self.document.files.clone(),
            ..Default::default()
        }
    }

    pub fn save_to(&mut self, path: &str) -> Result<(), String> {
        let json = crate::export::scene_to_json(&self.document.scene, &self.app_state())
            .map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        self.status = Some(format!("saved: {path}"));
        Ok(())
    }

    pub fn load_from(&mut self, path: &str) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (scene, app) = crate::export::json_to_scene(&text).map_err(|e| e.to_string())?;
        self.push_checkpoint();
        self.document.scene = scene;
        self.document.selected.clear();
        self.document.zoom = app.zoom.value;
        self.document.scroll = Point::new(app.scroll_x, app.scroll_y);
        self.document.theme = app.theme;
        self.document.files = app.files;
        self.image_sources.clear();
        self.sync_image_sources();
        self.document.refresh_bindings();
        self.status = Some(format!("loaded: {path}"));
        Ok(())
    }

    /// Ask the platform for a scene file, then load it.
    ///
    /// Upstream's `actionLoadScene` shows the OS file picker and loads whatever
    /// comes back. Every Open in the UI used to read a fixed
    /// `scene.excalidraw` from the process working directory and throw the
    /// result away, so the row did nothing at all unless that one file happened
    /// to exist — and said nothing when it did not.
    pub fn open_scene_picker(&mut self, cx: &mut Context<Self>) {
        let prompt = SharedString::from(self.i18n.t("menu.open"));
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(prompt),
        });
        cx.spawn(async move |this, cx| {
            match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first() {
                        let path = path.to_string_lossy().to_string();
                        let _ = this.update(cx, |editor, cx| {
                            editor.load_scene_file(&path);
                            cx.notify();
                        });
                    }
                }
                // The picker itself could not open — a session with no desktop
                // portal is the usual cause. Say so, rather than leaving the
                // row looking dead with no explanation.
                Ok(Err(_)) => {
                    let _ = this.update(cx, |editor, cx| {
                        editor.status = Some(editor.i18n.t("status.unavailable"));
                        cx.notify();
                    });
                }
                // Cancelled by the user: nothing to report.
                _ => {}
            }
        })
        .detach();
    }

    /// Load `path`, reporting a failure in the status line.
    ///
    /// [`Editor::load_from`] returns the error; the UI must surface it, because
    /// a silent no-op on Open is exactly what made the row look broken.
    pub fn load_scene_file(&mut self, path: &str) {
        if let Err(err) = self.load_from(path) {
            self.status = Some(format!("open failed: {err}"));
        }
    }

    /// Upstream's `App.getName()` fallback: `labels.untitled` joined to
    /// `getDateTime()`, which formats local time as `YYYY-MM-DD-HHMM`.
    fn untitled_name(&self) -> String {
        let stamp = chrono::Local::now().format("%Y-%m-%d-%H%M");
        format!("{}-{stamp}", self.i18n.t("labels.untitled"))
    }

    /// Raise the "Save to…" modal, seeding the name field the way
    /// `App.getName()` does for a scene that has never been named.
    ///
    /// Upstream raises this from the menu row and lets the *card's* button do
    /// the saving, so opening it never touches the filesystem.
    pub fn open_save_dialog(&mut self) {
        if self.project_name.is_empty() {
            self.project_name = self.untitled_name();
        }
        self.show_save_dialog = true;
    }

    pub fn close_save_dialog(&mut self) {
        self.show_save_dialog = false;
    }

    /// Write the scene to the path the platform's save panel returned.
    ///
    /// A failure here is a real one — a read-only directory, a full disk — so it
    /// goes to the status line and the modal stays up for another attempt.
    pub fn save_scene_file(&mut self, path: &str) {
        match self.save_to(path) {
            Ok(()) => self.show_save_dialog = false,
            Err(err) => self.status = Some(format!("save failed: {err}")),
        }
    }

    /// Ask the platform where to save, then write the scene there.
    ///
    /// Upstream hands this to `saveAsJSON`, which triggers a browser download; a
    /// native app has no downloads folder, so the OS save panel stands in for it.
    /// This is what the "save to disk" card's button runs.
    pub fn save_scene_picker(&mut self, cx: &mut Context<Self>) {
        let name = self.project_name_value();
        let name = if name.ends_with(".excalidraw") {
            name
        } else {
            format!("{name}.excalidraw")
        };
        let directory = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let receiver = cx.prompt_for_new_path(&directory, Some(name.as_str()));
        cx.spawn(async move |this, cx| {
            match receiver.await {
                Ok(Ok(Some(path))) => {
                    let path = path.to_string_lossy().to_string();
                    let _ = this.update(cx, |editor, cx| {
                        editor.save_scene_file(&path);
                        cx.notify();
                    });
                }
                // The panel itself could not open — a session with no desktop
                // portal is the usual cause. Say so rather than leave the button
                // looking dead, the way Open already reports it.
                Ok(Err(_)) => {
                    let _ = this.update(cx, |editor, cx| {
                        editor.status = Some(editor.i18n.t("status.unavailable"));
                        cx.notify();
                    });
                }
                // Cancelled by the user: nothing to report.
                _ => {}
            }
        })
        .detach();
    }

    pub fn export_png_bytes(&self, scale: f32) -> Result<Vec<u8>, String> {
        crate::export::export_png_with_files(
            &self.document.scene,
            self.export_background(),
            scale,
            &self.document.files,
        )
    }

    pub fn export_png_to(&mut self, path: &str) -> Result<(), String> {
        match self.export_png_bytes(2.0) {
            Ok(bytes) => {
                std::fs::write(path, bytes).map_err(|e| e.to_string())?;
                self.status = Some(format!("png: {path}"));
                Ok(())
            }
            Err(err) => {
                self.status = Some(err.clone());
                Err(err)
            }
        }
    }

    pub fn export_svg_to(&mut self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.export_svg_string()).map_err(|e| e.to_string())?;
        self.status = Some(format!("svg: {path}"));
        Ok(())
    }

    fn on_new_document(&mut self) {
        self.push_checkpoint();
        self.document.scene = excalidraw_core::scene::Scene::new();
        self.document.selected.clear();
    }

    // ----- keyboard ---------------------------------------------------------

    pub fn handle_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
        viewport: (f64, f64),
    ) -> bool {
        self.viewport = viewport;
        // Escaping a live colour pick only ends the pick. Upstream's eye
        // dropper swallows the key before the canvas sees it, so a pick never
        // also clears the selection underneath.
        if self.eye_dropper.is_some() && key == "escape" {
            self.cancel_eye_dropper();
            return true;
        }
        // The style popups own the keyboard while they are up, for the same
        // reason the modal overlays do: a letter typed into the hex field or the
        // font search must not also switch the tool underneath.
        if self.show_save_dialog {
            return self.handle_save_key(key, key_char, modifiers);
        }
        if self.show_link_dialog {
            return self.handle_link_key(key, key_char, modifiers);
        }
        if self.color_popup.is_some() {
            return self.handle_color_key(key, key_char, modifiers);
        }
        if self.font_popup {
            return self.handle_font_key(key, key_char, modifiers);
        }
        // Modal overlays own the keyboard while they are up: a letter must go
        // into the filter box, not switch the tool underneath it.
        if self.show_palette {
            return self.handle_palette_key(key, key_char, modifiers);
        }
        if self.show_find {
            return self.handle_find_key(key, key_char, modifiers);
        }
        if self.show_help {
            if key == "escape" || key == "?" {
                self.show_help = false;
            }
            return true;
        }

        if key == "space" {
            self.space_pan = true;
            return true;
        }
        if key == "escape" {
            if self.editing_text.is_some() {
                self.commit_text_edit();
                return true;
            }
            if !self.pending_points.is_empty() {
                self.pending_points.clear();
                self.cursor_world = None;
                return true;
            }
            self.context_menu = None;
            self.show_menu = false;
            self.show_more_tools = false;
            return true;
        }

        if let Some(id) = self.editing_text.clone() {
            return self.handle_text_key(&id, key, key_char, modifiers);
        }

        if modifiers.control || modifiers.platform {
            match key {
                "z" => {
                    if modifiers.shift {
                        self.on_redo();
                    } else {
                        self.on_undo();
                    }
                    return true;
                }
                "y" => {
                    self.on_redo();
                    return true;
                }
                "a" => {
                    self.document.select_all();
                    return true;
                }
                "c" => {
                    self.on_copy();
                    return true;
                }
                "x" => {
                    self.on_cut();
                    return true;
                }
                "v" => {
                    self.on_paste();
                    return true;
                }
                "d" => {
                    self.on_duplicate();
                    return true;
                }
                "g" => {
                    if modifiers.shift {
                        self.on_ungroup();
                    } else {
                        self.on_group();
                    }
                    return true;
                }
                // Ctrl+S — upstream splits this: Ctrl+S is
                // `actionSaveToActiveFile` (rewrite the open file handle) and
                // Ctrl+Shift+S is `actionSaveFileToDisk` (raise the dialog).
                // This build has no file handle to rewrite, so the one shortcut
                // does what the dialog's button does.
                "s" => {
                    self.pending_save_dialog = true;
                    return true;
                }
                // Ctrl+O — upstream's `actionLoadScene` keyTest. The menu row
                // advertises this shortcut, so it has to be bound.
                "o" => {
                    self.pending_scene_open = true;
                    return true;
                }
                "e" if modifiers.shift => {
                    let _ = self.export_png_to("excalidraw-export.png");
                    return true;
                }
                "/" => {
                    self.toggle_palette();
                    return true;
                }
                "f" => {
                    self.open_find();
                    return true;
                }
                // Toggle grid — upstream binds it to Ctrl+'.
                "'" => {
                    self.on_toggle_grid();
                    return true;
                }
                "0" => {
                    self.on_reset_zoom();
                    return true;
                }
                _ => {}
            }
        }

        if modifiers.shift {
            match key {
                "1" => {
                    self.document.zoom_to_fit(viewport, 40.0);
                    return true;
                }
                "2" => {
                    self.document.zoom_to_selection(viewport, 40.0);
                    return true;
                }
                _ => {}
            }
        }

        if key == "enter" || key == "numpadenter" {
            if !self.pending_points.is_empty() {
                self.finish_pending_line();
                return true;
            }
            return false;
        }

        if key == "delete" || key == "backspace" {
            self.on_delete();
            return true;
        }

        // Preferences shortcuts, matching upstream's Preferences panel:
        // Alt+S object snap, Alt+Z zen mode, Alt+R view mode, Alt+/ stats.
        if modifiers.alt && !modifiers.control && !modifiers.platform {
            match key {
                "s" => {
                    self.document.snap_to_objects = !self.document.snap_to_objects;
                    return true;
                }
                "z" => {
                    self.document.zen_mode = !self.document.zen_mode;
                    return true;
                }
                "r" => {
                    self.document.view_mode = !self.document.view_mode;
                    return true;
                }
                "/" => {
                    self.document.show_stats = !self.document.show_stats;
                    return true;
                }
                _ => return false,
            }
        }

        if modifiers.alt {
            return false;
        }

        // Help / shortcuts — upstream opens the shortcut dialog on "?".
        if key == "?" || (key == "/" && modifiers.shift) {
            self.open_help();
            return true;
        }

        match key {
            "v" | "1" => self.set_tool(Tool::Selection),
            "h" => self.set_tool(Tool::Hand),
            "r" | "2" => self.set_tool(Tool::Rectangle),
            "d" | "3" => self.set_tool(Tool::Diamond),
            "o" | "4" => self.set_tool(Tool::Ellipse),
            "a" | "5" => self.set_tool(Tool::Arrow),
            "l" | "6" => self.set_tool(Tool::Line),
            "p" | "7" => self.set_tool(Tool::FreeDraw),
            "t" | "8" => self.set_tool(Tool::Text),
            "i" | "9" => self.set_tool(Tool::Image),
            "f" => self.set_tool(Tool::Frame),
            "n" => self.set_tool(Tool::StickyNote),
            "e" | "0" => self.set_tool(Tool::Eraser),
            "k" => self.set_tool(Tool::Laser),
            // The padlock has a letter upstream too (Preferences → Tool lock Q).
            "q" => self.document.tool_locked = !self.document.tool_locked,
            _ => return false,
        }
        true
    }

    fn handle_text_key(
        &mut self,
        id: &str,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.commit_text_edit();
                return true;
            }
            "enter" | "numpadenter" => {
                if let Some(excalidraw_core::element::Element::Text(t)) =
                    self.document.scene.get_mut(id)
                {
                    t.text.push('\n');
                }
                self.refresh_text_bounds(id);
                return true;
            }
            "backspace" | "delete" => {
                if let Some(excalidraw_core::element::Element::Text(t)) =
                    self.document.scene.get_mut(id)
                {
                    t.text.pop();
                }
                self.refresh_text_bounds(id);
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return false;
        }
        let typed = match key_char {
            Some(c) if !c.is_empty() => Some(c),
            _ if key.len() == 1 => Some(key.to_string()),
            _ => None,
        };
        if let Some(ch) = typed {
            if self.replace_on_type {
                if let Some(excalidraw_core::element::Element::Text(t)) =
                    self.document.scene.get_mut(id)
                {
                    t.text.clear();
                }
                self.replace_on_type = false;
            }
            if let Some(excalidraw_core::element::Element::Text(t)) =
                self.document.scene.get_mut(id)
            {
                t.text.push_str(&ch);
            }
            self.refresh_text_bounds(id);
            return true;
        }
        false
    }

    // ----- overlay keyboards -------------------------------------------------

    fn handle_palette_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.show_palette = false;
                return true;
            }
            "enter" | "numpadenter" => {
                self.run_palette_selection();
                return true;
            }
            "up" | "arrowup" => {
                self.palette_index = self.palette_index.saturating_sub(1);
                return true;
            }
            "down" | "arrowdown" => {
                if self.palette_index + 1 < self.palette_results().len() {
                    self.palette_index += 1;
                }
                return true;
            }
            "backspace" | "delete" => {
                self.palette_query.pop();
                self.palette_index = 0;
                return true;
            }
            "space" => {
                self.palette_query.push(' ');
                self.palette_index = 0;
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }
        if let Some(ch) = printable_char(key, key_char) {
            self.palette_query.push_str(&ch);
            self.palette_index = 0;
        }
        true
    }

    fn handle_find_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.show_find = false;
                return true;
            }
            "enter" | "numpadenter" => {
                self.step_find(!modifiers.shift);
                return true;
            }
            "backspace" | "delete" => {
                self.find_query.pop();
                self.recompute_find();
                return true;
            }
            "space" => {
                self.find_query.push(' ');
                self.recompute_find();
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }
        if let Some(ch) = printable_char(key, key_char) {
            self.find_query.push_str(&ch);
            self.recompute_find();
        }
        true
    }

    /// Which row's colour popup is up, if any.
    pub(crate) fn color_popup(&self) -> Option<ColorTarget> {
        self.color_popup
    }

    /// The colour a target currently holds, read off the selection so the popup
    /// opens showing what the element actually uses.
    pub(crate) fn color_popup_color(&self, target: ColorTarget) -> String {
        // The canvas background lives on the document, not on an element, so it
        // is answered before the selection lookup ever runs.
        if target == ColorTarget::CanvasBackground {
            return self.document.view_background_color.clone();
        }
        self.document
            .selected_elements()
            .first()
            .map(|element| match target {
                ColorTarget::Stroke => element.base().stroke_color.clone(),
                ColorTarget::Background => element.base().background_color.clone(),
                // Returned above; this arm only keeps the match exhaustive.
                ColorTarget::CanvasBackground => String::new(),
            })
            .unwrap_or_else(|| "transparent".to_string())
    }

    /// The ramp step a target's grid is drawn at.
    pub(crate) fn active_shade(&self, target: ColorTarget) -> usize {
        match target {
            ColorTarget::Stroke => self.shade_stroke,
            ColorTarget::Background => self.shade_background,
            // Upstream's canvas-background picker passes `palette={null}`, so
            // there is no ramp to remember a step in.
            ColorTarget::CanvasBackground => 0,
        }
    }

    fn set_active_shade(&mut self, target: ColorTarget, shade: usize) {
        match target {
            ColorTarget::Stroke => self.shade_stroke = shade,
            ColorTarget::Background => self.shade_background = shade,
            ColorTarget::CanvasBackground => {}
        }
    }

    /// Open the picker for `target`, or close it when it is already up.
    pub(crate) fn toggle_color_popup(&mut self, target: ColorTarget) {
        if self.color_popup == Some(target) {
            self.close_color_popup();
            return;
        }
        self.font_popup = false;
        self.show_link_dialog = false;
        self.color_popup = Some(target);
        // Open on the colour in use (upstream's `activeShade` effect), so the
        // grid starts where the element is instead of at the default step.
        let color = self.color_popup_color(target);
        let shade = crate::theme::color_position(&color)
            .and_then(|(_, shade)| shade)
            .unwrap_or(match target {
                ColorTarget::Stroke => crate::theme::DEFAULT_STROKE_SHADE,
                ColorTarget::Background => crate::theme::DEFAULT_BACKGROUND_SHADE,
                ColorTarget::CanvasBackground => 0,
            });
        self.set_active_shade(target, shade);
        self.hex_buffer = color.trim_start_matches('#').to_string();
        self.hex_focused = false;
    }

    pub(crate) fn close_color_popup(&mut self) {
        self.color_popup = None;
        self.hex_focused = false;
    }

    /// Write a colour into a popup's target and keep the hex field in step.
    pub(crate) fn apply_color(&mut self, target: ColorTarget, color: &str) {
        match target {
            ColorTarget::Stroke => self.document.set_selected_stroke_color(color),
            ColorTarget::Background => self.document.set_selected_background_color(color),
            ColorTarget::CanvasBackground => {
                self.document.view_background_color = color.to_string()
            }
        }
        self.hex_buffer = color.trim_start_matches('#').to_string();
    }

    /// Step a popup's grid to another ramp step.
    pub(crate) fn set_shade(&mut self, target: ColorTarget, shade: usize) {
        self.set_active_shade(target, shade);
    }

    /// The hex field's buffer, without the leading `#`.
    pub(crate) fn hex_buffer(&self) -> &str {
        &self.hex_buffer
    }

    /// Whether keystrokes are currently going into the hex field.
    pub(crate) fn is_hex_focused(&self) -> bool {
        self.hex_focused
    }

    /// Focus or blur the hex field (the field's own click and Tab).
    pub(crate) fn set_hex_focused(&mut self, focused: bool) {
        self.hex_focused = focused;
    }

    /// Whether the font-picker popup is up.
    pub(crate) fn is_font_popup_open(&self) -> bool {
        self.font_popup
    }

    pub(crate) fn toggle_font_popup(&mut self) {
        self.font_popup = !self.font_popup;
        if self.font_popup {
            self.color_popup = None;
            self.show_link_dialog = false;
            self.font_query.clear();
        }
    }

    /// The font picker's quick-search needle.
    pub(crate) fn font_search(&self) -> &str {
        &self.font_query
    }

    /// The families the picker lists, in upstream's two groups.
    ///
    /// "In this scene" holds the families the scene already uses — every
    /// family is usable in this build, so that is the families of the
    /// non-deleted text elements.
    pub(crate) fn picker_font_groups(&self) -> (Vec<FontFamily>, Vec<FontFamily>) {
        let mut scene_families: Vec<FontFamily> = Vec::new();
        for element in self.document.scene.non_deleted() {
            if let Element::Text(text) = element {
                if !scene_families.contains(&text.font_family) {
                    scene_families.push(text.font_family);
                }
            }
        }
        let query = self.font_query.to_lowercase();
        let matches = |family: &FontFamily| {
            query.is_empty()
                || crate::properties::font_label(*family)
                    .to_lowercase()
                    .contains(&query)
        };
        // The scene set is drawn from the unfiltered families, so a family that
        // the needle hides stays out of "available" too — upstream derives both
        // groups from the same `sceneFamilies` list.
        let scene: Vec<FontFamily> = scene_families.iter().copied().filter(matches).collect();
        let available: Vec<FontFamily> = crate::properties::all_fonts()
            .into_iter()
            .filter(|family| !scene_families.contains(family) && matches(family))
            .collect();
        (scene, available)
    }

    /// The first family a filtered picker would land on (Enter picks it).
    pub(crate) fn first_picker_font(&self) -> Option<FontFamily> {
        let (scene, available) = self.picker_font_groups();
        scene.into_iter().chain(available).next()
    }

    /// Open the element-link dialog, seeded with the link already on the
    /// selection (upstream's `ElementLinkDialog` `nextLink`).
    pub(crate) fn open_link_dialog(&mut self) {
        self.link_input = self.selected_link().unwrap_or_default();
        self.show_link_dialog = true;
        self.color_popup = None;
        self.font_popup = false;
    }

    pub fn is_link_dialog_open(&self) -> bool {
        self.show_link_dialog
    }

    /// The link the selection carries, if any.
    pub fn selected_link(&self) -> Option<String> {
        self.document
            .selected_elements()
            .first()
            .and_then(|element| element.base().link.clone())
    }

    /// The dialog's text field contents.
    pub fn link_input(&self) -> &str {
        &self.link_input
    }

    /// Empty the field. Upstream only writes to the element on confirm, so a
    /// removal still has to be confirmed.
    pub(crate) fn clear_link_input(&mut self) {
        self.link_input.clear();
    }

    pub(crate) fn cancel_link_dialog(&mut self) {
        self.show_link_dialog = false;
    }

    /// Confirm the dialog: write the link onto every selected element, or clear
    /// it when the field was emptied (upstream's `handleConfirm`).
    pub(crate) fn confirm_link(&mut self) {
        let link = self.link_input.trim().to_string();
        if link.is_empty() {
            self.document.set_selected_link(None);
        } else {
            self.document.set_selected_link(Some(&link));
        }
        self.show_link_dialog = false;
    }

    /// Route a keystroke into the element-link dialog.
    fn handle_link_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.cancel_link_dialog();
                return true;
            }
            "enter" | "numpadenter" => {
                self.confirm_link();
                return true;
            }
            "backspace" | "delete" => {
                self.link_input.pop();
                return true;
            }
            "space" => {
                self.link_input.push(' ');
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }
        if let Some(ch) = printable_char(key, key_char) {
            self.link_input.push_str(&ch);
        }
        true
    }

    /// Route a keystroke into the open "Save to…" modal.
    ///
    /// The modal's file-name field is a plain `<input type="text">` upstream, so
    /// it owns the keyboard exactly as the link field does: Escape dismisses the
    /// dialog, the editing keys edit the name, and everything else is swallowed
    /// so no letter can switch the tool underneath. Enter is swallowed too —
    /// upstream's input only blurs on Enter, it does not submit.
    fn handle_save_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.close_save_dialog();
                return true;
            }
            "backspace" | "delete" => {
                self.project_name.pop();
                return true;
            }
            "space" => {
                self.project_name.push(' ');
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }
        if let Some(ch) = printable_char(key, key_char) {
            self.project_name.push_str(&ch);
        }
        true
    }

    /// Route a keystroke into the open colour picker.
    ///
    /// The bindings are upstream's (`colorPickerKeyNavHandler`): Tab walks to
    /// the hex field, the 15 palette letters pick a grid entry at the active
    /// step, `1`–`5` pick a custom colour, and Shift+`1`–`5` step the ramp.
    fn handle_color_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        let Some(target) = self.color_popup else {
            return false;
        };
        if key == "escape" {
            if self.hex_focused {
                self.hex_focused = false;
            } else {
                self.close_color_popup();
            }
            return true;
        }
        if key == "tab" {
            self.hex_focused = !self.hex_focused;
            return true;
        }

        if self.hex_focused {
            match key {
                "enter" | "numpadenter" => {
                    self.hex_focused = false;
                    return true;
                }
                "backspace" | "delete" => {
                    self.hex_buffer.pop();
                    self.apply_hex_buffer(target);
                    return true;
                }
                _ => {}
            }
            if modifiers.control || modifiers.platform || modifiers.alt {
                return true;
            }
            if let Some(ch) = printable_char(key, key_char) {
                // Only hex digits may enter the field, which is the constraint
                // upstream's `normalizeInputColor` enforces from the other side.
                if ch.chars().all(|c| c.is_ascii_hexdigit()) {
                    self.hex_buffer.push_str(&ch.to_lowercase());
                    self.apply_hex_buffer(target);
                }
            }
            return true;
        }

        // Upstream's canvas-background picker has no palette, shade ramp or
        // custom-colour list to hotkey into, so every remaining key is swallowed
        // rather than leaking through to the canvas tools behind the popup.
        if target == ColorTarget::CanvasBackground {
            return true;
        }

        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }

        if modifiers.shift {
            if let Some(index) = shade_digit(key) {
                self.set_active_shade(target, index);
                return true;
            }
        }
        if let Some(index) = shade_digit(key) {
            let custom = self.custom_colors(target);
            if let Some(color) = custom.get(index).cloned() {
                self.apply_color(target, &color);
            }
            return true;
        }
        if key.len() == 1 {
            let typed = key.to_lowercase();
            if let Some(position) = PALETTE_HOTKEYS
                .iter()
                .position(|hotkey| *hotkey == typed.as_str())
            {
                let entry = &crate::theme::element_palette()[position];
                let color = entry.at(self.active_shade(target));
                self.apply_color(target, color);
                return true;
            }
        }
        // Arrows walk the grid at the active step, the way upstream's arrow
        // handler does inside the base-colours section.
        let color = self.color_popup_color(target);
        let palette = crate::theme::element_palette();
        let current = crate::theme::color_position(&color)
            .and_then(|(name, _)| palette.iter().position(|entry| entry.name == name))
            .unwrap_or(0);
        if let Some(next) = palette_step(key, current, palette.len()) {
            self.apply_color(target, palette[next].at(self.active_shade(target)));
        }
        true
    }

    /// The custom colours of the scene, most used first, capped at five
    /// (upstream `getMostUsedCustomColors` / `MAX_CUSTOM_COLORS_USED_IN_CANVAS`).
    pub(crate) fn custom_colors(&self, target: ColorTarget) -> Vec<String> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for element in self.document.scene.non_deleted() {
            let color = match target {
                ColorTarget::Stroke => element.base().stroke_color.as_str(),
                ColorTarget::Background => element.base().background_color.as_str(),
                // The canvas-background picker is hex-only upstream, so it never
                // lists the scene's most-used custom colours.
                ColorTarget::CanvasBackground => continue,
            };
            if color == "transparent" || !crate::theme::is_custom_color(color) {
                continue;
            }
            match counts.iter_mut().find(|(existing, _)| existing == color) {
                Some((_, count)) => *count += 1,
                None => counts.push((color.to_string(), 1)),
            }
        }
        // A stable sort keeps first-seen order among equally used colours,
        // which is what upstream's `Map` iteration gives it.
        counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        counts.into_iter().map(|(color, _)| color).take(5).collect()
    }

    /// Apply the hex field when it holds a complete colour; partial input is
    /// left alone, exactly as upstream's live `normalizeInputColor` does.
    fn apply_hex_buffer(&mut self, target: ColorTarget) {
        if self.hex_buffer.len() == 6 && self.hex_buffer.chars().all(|c| c.is_ascii_hexdigit()) {
            let color = format!("#{}", self.hex_buffer.to_lowercase());
            self.apply_color(target, &color);
        }
    }

    /// Route a keystroke into the font picker.
    fn handle_font_key(
        &mut self,
        key: &str,
        key_char: Option<String>,
        modifiers: Modifiers,
    ) -> bool {
        match key {
            "escape" => {
                self.font_popup = false;
                return true;
            }
            "enter" | "numpadenter" => {
                if let Some(family) = self.first_picker_font() {
                    self.document.set_selected_font_family(family);
                }
                return true;
            }
            "backspace" | "delete" => {
                self.font_query.pop();
                return true;
            }
            "space" => {
                self.font_query.push(' ');
                return true;
            }
            _ => {}
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return true;
        }
        if let Some(ch) = printable_char(key, key_char) {
            self.font_query.push_str(&ch);
        }
        true
    }
}

/// The palette hotkeys upstream binds to its 5×3 grid, read row by row
/// (`colorPickerHotkeyBindings`).
const PALETTE_HOTKEYS: [&str; 15] = [
    "q", "w", "e", "r", "t", "a", "s", "d", "f", "g", "z", "x", "c", "v", "b",
];

/// `1`–`5` as the zero-based ramp step they address.
fn shade_digit(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0),
        "2" => Some(1),
        "3" => Some(2),
        "4" => Some(3),
        "5" => Some(4),
        _ => None,
    }
}

/// The grid index an arrow key lands on, wrapping like upstream's
/// `arrowHandler` over five columns.
fn palette_step(key: &str, index: usize, length: usize) -> Option<usize> {
    const COLUMNS: usize = 5;
    let rows = length.div_ceil(COLUMNS);
    match key {
        "left" | "arrowleft" => Some(if index == 0 { length - 1 } else { index - 1 }),
        "right" | "arrowright" => Some((index + 1) % length),
        "down" | "arrowdown" => {
            let next = index + COLUMNS;
            if next >= length {
                Some(index % COLUMNS)
            } else {
                Some(next)
            }
        }
        "up" | "arrowup" => {
            let previous = index as isize - COLUMNS as isize;
            let candidate = if previous < 0 {
                (COLUMNS * rows) as isize + previous
            } else {
                previous
            };
            if candidate < 0 || candidate as usize >= length {
                None
            } else {
                Some(candidate as usize)
            }
        }
        _ => None,
    }
}

/// The character a keystroke types, if any — the same rule the text editor
/// uses, so an IME-provided `key_char` wins over the raw key name.
fn printable_char(key: &str, key_char: Option<String>) -> Option<String> {
    match key_char {
        Some(c) if !c.is_empty() => Some(c),
        _ if key.chars().count() == 1 => Some(key.to_string()),
        _ => None,
    }
}

/// The text a "find on canvas" search can see: a text element's body.
fn element_search_text(element: &Element) -> Option<&str> {
    match element {
        Element::Text(t) => Some(t.text.as_str()),
        _ => None,
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

/// The main tool row, in upstream order (`components/Tools.tsx`).
///
/// Each entry is `(tool, icon, shortcut)`. The icon names are the upstream ones
/// — `assets/icons/<name>.svg` is the *real* Excalidraw glyph, not a stand-in —
/// and the shortcut is what Excalidraw prints in the button's corner. The hand
/// tool carries no letter: upstream leaves both it and the padlock bare even
/// though both do have key bindings, so `None` here is deliberate.
const TOOL_ROW: &[(Tool, &str, Option<&str>)] = &[
    (Tool::Hand, "handIcon", None),
    (Tool::Selection, "SelectionIcon", Some("V")),
    (Tool::Rectangle, "RectangleIcon", Some("R")),
    (Tool::Diamond, "DiamondIcon", Some("D")),
    (Tool::Ellipse, "EllipseIcon", Some("O")),
    (Tool::Arrow, "ArrowIcon", Some("A")),
    (Tool::Line, "LineIcon", Some("L")),
    (Tool::FreeDraw, "FreedrawIcon", Some("P")),
    (Tool::Text, "TextIcon", Some("T")),
    (Tool::StickyNote, "stickyNoteToolIcon", Some("N")),
    (Tool::Eraser, "EraserIcon", Some("E")),
];

/// The tools tucked behind the toolbar's vertical-dots button.
/// One row of the "more tools" popover — the full upstream list, in upstream
/// order, Generate section included.
struct MoreTool {
    key: &'static str,
    icon: &'static str,
    shortcut: Option<&'static str>,
    /// Rows that need the online AI backend carry the little "AI" chip.
    ai: bool,
    action: MoreToolAction,
}

enum MoreToolAction {
    /// Arms a real tool.
    Tool(Tool),
    /// Drops a web-embed card onto the viewport centre.
    Embed,
    /// Present in the popover; needs an upstream backend we don't ship, so the
    /// click reports itself instead of pretending.
    Stub,
}

const MORE_TOOLS: &[MoreTool] = &[
    MoreTool {
        key: "toolbar.image",
        icon: "ImageIcon",
        shortcut: Some("9"),
        ai: false,
        action: MoreToolAction::Tool(Tool::Image),
    },
    MoreTool {
        key: "toolbar.frame",
        icon: "frameToolIcon",
        shortcut: Some("F"),
        ai: false,
        action: MoreToolAction::Tool(Tool::Frame),
    },
    MoreTool {
        key: "toolbar.embed",
        icon: "EmbedIcon",
        shortcut: None,
        ai: false,
        action: MoreToolAction::Embed,
    },
    // Upstream leaves this one English even on the zh site.
    MoreTool {
        key: "toolbar.drawToShape",
        icon: "drawShapeToolIcon",
        shortcut: Some("Shift+X"),
        ai: false,
        action: MoreToolAction::Stub,
    },
    MoreTool {
        key: "toolbar.laser",
        icon: "laserPointerToolIcon",
        shortcut: Some("K"),
        ai: false,
        action: MoreToolAction::Tool(Tool::Laser),
    },
    // Ditto — English on excalidraw.com/zh-CN.
    MoreTool {
        key: "toolbar.bucketFill",
        icon: "bucketFillIcon",
        shortcut: Some("B"),
        ai: false,
        action: MoreToolAction::Stub,
    },
    MoreTool {
        key: "toolbar.lasso",
        icon: "lassoToolIcon",
        shortcut: None,
        ai: false,
        action: MoreToolAction::Stub,
    },
    MoreTool {
        key: "toolbar.textToDiagram",
        icon: "textToDiagramIcon",
        shortcut: None,
        ai: true,
        action: MoreToolAction::Stub,
    },
    MoreTool {
        key: "toolbar.mermaid",
        icon: "mermaidIcon",
        shortcut: None,
        ai: false,
        action: MoreToolAction::Stub,
    },
    MoreTool {
        key: "toolbar.wireframeToCode",
        icon: "wireframeToCodeIcon",
        shortcut: None,
        ai: true,
        action: MoreToolAction::Stub,
    },
];

/// What one button in the canvas-buttons island asks for.
#[derive(Clone, Copy)]
enum CanvasButton {
    /// Text to diagram — upstream hands the magic frame back to the AI service.
    ConvertToCode,
    /// Copy the generated iframe's source URL.
    CopySource,
    /// Fullscreen the generated iframe.
    Fullscreen,
}

/// Which checkout an OS file drag would land in, upstream `FileDropState`'s
/// `kind`.
///
/// The web build cannot see a dragged file's name before the drop, so it treats
/// everything without a MIME type as `Unknown`; a native window *can* read the
/// path, so the extension is a better signal than the browser gets and the
/// overlay names the real action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DropKind {
    /// A `.excalidraw` / exported-scene file — replaces the canvas.
    Scene,
    /// A `.excalidrawlib` file — appends to the library.
    Library,
    /// No usable extension; could be either until it is dropped.
    Unknown,
}

/// A live OS file drag hovering the window (upstream `FileDropState`).
#[derive(Clone, Copy, Debug)]
struct FileDragState {
    kind: DropKind,
    /// Shift flips "replace" into "add", exactly as upstream's overlay says.
    shift: bool,
}

impl FileDragState {
    /// Whether the drop keeps what is already on the canvas — upstream
    /// `keepsContent`.
    fn keeps_content(self) -> bool {
        self.shift || self.kind == DropKind::Library
    }
}

/// A transient tooltip shown beside the cursor (upstream `cursorHintAtom`).
#[derive(Clone, Debug)]
struct CursorHintState {
    icon: &'static str,
    /// Bumped on every trigger so a stale auto-hide timer can tell it has been
    /// superseded instead of clearing the hint that replaced it.
    nonce: u64,
}

/// The bottom-centre status toast (upstream `appState.toast`).
#[derive(Clone, Debug)]
struct ToastState {
    message: String,
    closable: bool,
    /// Bumped per toast so the previous one's auto-hide timer cannot clear a
    /// toast that replaced it.
    nonce: u64,
}

/// Which style [Editor::eye_dropper] writes a picked colour into.
///
/// `CanvasBackground` is the odd one out: it is a document property rather than
/// an element style, which is why it is the only target that reads and writes
/// nothing on the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorTarget {
    Stroke,
    Background,
    CanvasBackground,
}

/// The live colour-picking gesture (upstream `activeEyeDropperAtom`).
#[derive(Clone, Debug)]
struct EyeDropperState {
    target: ColorTarget,
    /// Pointer position, in window coordinates.
    at: (f64, f64),
    /// The colour currently under the pointer, as `#rrggbb`.
    color: String,
}

/// Whether an iframe holds finished generated content — upstream
/// `customData.generationData.status === "done"`.
///
/// Upstream only shows the canvas buttons once generation has *finished*; while
/// it is pending the element is still being written to.
fn iframe_generation_finished(base: &excalidraw_core::element::ElementBase) -> bool {
    base.custom_data
        .as_ref()
        .and_then(|data| data.get("generationData"))
        .and_then(|generation| generation.get("status"))
        .and_then(|status| status.as_str())
        == Some("done")
}

/// The id an unlock bubble stands for: the element's *outermost* group when it
/// belongs to one, otherwise the element itself.
///
/// Upstream keys `activeLockedId` this way so a locked group unlocks as a unit
/// rather than one member at a time.
fn locked_subject_id(element: &Element) -> String {
    element
        .base()
        .group_ids
        .last()
        .cloned()
        .unwrap_or_else(|| element.id().to_string())
}

/// The island's own colour scheme, driven by the document theme.
fn palette_of(theme: Theme) -> crate::design::Palette {
    match theme {
        Theme::Dark => crate::design::Palette::Dark,
        Theme::Light => crate::design::Palette::Light,
    }
}

/// How long a cursor hint stays before it starts fading out (upstream
/// `CURSOR_HINT_DURATION`).
const CURSOR_HINT_DURATION_MS: u64 = 700;
/// The fade that follows it, so a hint lives 800ms in total (upstream
/// `CURSOR_HINT_FADE_DURATION`); the renderer draws the matching opacity.
const CURSOR_HINT_FADE_MS: u64 = 100;
/// Letter tool shortcuts go quiet for this long after a hint, so re-picking a
/// tool you just used does not nag (upstream `CURSOR_HINT_COOLDOWN`).
const CURSOR_HINT_COOLDOWN_MS: u64 = 30_000;
/// The arrowhead a hint paints for the arrow tool. This build only ever draws
/// sharp arrows, so that is the variant the shortcut hint reports.
const ARROW_TOOL_ICON: &str = "sharpArrowIcon";

/// `Card.tsx`'s `COLOR_MAP.lime` — the "save to disk" card's accent, which
/// paints its icon disc and its button. The two darker stops are the button's
/// hover and pressed states (open-color lime 7/8/9).
const CARD_LIME: u32 = 0x74b816;
const CARD_LIME_DARKER: u32 = 0x66a80f;

/// Place an overlay beside the cursor, flipping it to the other side of the
/// pointer on each axis when it would overflow the container and clamping it so
/// as much as possible stays visible — upstream `positionElementBesideCursor`.
///
/// Both the pointer and the container are window-local here, so upstream's
/// `cursor - container.left/top` subtraction is already done by the caller.
fn position_beside_cursor(
    cursor: (f64, f64),
    element: (f64, f64),
    container: (f64, f64),
    gap: f64,
) -> (f64, f64) {
    fn axis(cursor: f64, element: f64, container: f64, gap: f64) -> f64 {
        let position = if cursor + gap + element > container {
            cursor - gap - element
        } else {
            cursor + gap
        };
        position.clamp(0.0, (container - element).max(0.0))
    }
    (
        axis(cursor.0, element.0, container.0, gap),
        axis(cursor.1, element.1, container.1, gap),
    )
}

/// Whether a colour is dark enough to need a light ring around it — upstream
/// `isColorDark`, applied to the eye-dropper preview's border.
///
/// Upstream reads an empty colour as black and an unparseable one as black too,
/// then compares the YIQ luma against 160.
fn is_dark_hex(hex: &str) -> bool {
    fn channel(digits: &str, index: usize) -> Option<u32> {
        u32::from_str_radix(digits.get(index * 2..index * 2 + 2)?, 16).ok()
    }
    if hex.is_empty() {
        return true;
    }
    if hex.eq_ignore_ascii_case("transparent") {
        return false;
    }
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    let (r, g, b) = match (channel(digits, 0), channel(digits, 1), channel(digits, 2)) {
        (Some(r), Some(g), Some(b)) => (r, g, b),
        _ => return true,
    };
    r * 299 + g * 587 + b * 114 < 160_000
}

/// The icon a shape-switch button paints, matching upstream's
/// `ConvertElementTypePopup` icon set.
fn convert_target_icon(target: ConvertTarget) -> &'static str {
    match target {
        ConvertTarget::Rectangle => "RectangleIcon",
        ConvertTarget::Diamond => "DiamondIcon",
        ConvertTarget::Ellipse => "EllipseIcon",
        ConvertTarget::Line => "LineIcon",
        ConvertTarget::SharpArrow => "sharpArrowIcon",
        ConvertTarget::CurvedArrow => "roundArrowIcon",
        ConvertTarget::ElbowArrow => "elbowArrowIcon",
    }
}

fn convert_target_slug(target: ConvertTarget) -> &'static str {
    match target {
        ConvertTarget::Rectangle => "rectangle",
        ConvertTarget::Diamond => "diamond",
        ConvertTarget::Ellipse => "ellipse",
        ConvertTarget::Line => "line",
        ConvertTarget::SharpArrow => "sharpArrow",
        ConvertTarget::CurvedArrow => "curvedArrow",
        ConvertTarget::ElbowArrow => "elbowArrow",
    }
}

/// Split a localized hint into runs, tagging the ones that came from a `{{…}}`
/// marker and therefore have to be drawn as a `<kbd>` chip.
///
/// Upstream writes `hold <kbd>Scroll wheel</kbd> or <kbd>Space</kbd> while
/// dragging`; the marker form keeps the key names inside the translatable
/// string without the i18n layer needing to know about markup. An unterminated
/// marker degrades to plain text rather than swallowing the rest of the
/// sentence.
pub fn split_hint(s: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        let (head, tail) = rest.split_at(start);
        if !head.is_empty() {
            parts.push((head.to_owned(), false));
        }
        match tail[2..].find("}}") {
            Some(end) => {
                parts.push((tail[2..2 + end].to_owned(), true));
                rest = &tail[2 + end + 2..];
            }
            None => {
                parts.push((tail.to_owned(), false));
                return parts;
            }
        }
    }
    if !rest.is_empty() {
        parts.push((rest.to_owned(), false));
    }
    parts
}

/// A run of plain hint text, boxed so it can share a child list with `<kbd>`
/// chips.
fn plain_text(s: &str) -> AnyElement {
    div()
        .child(SharedString::from(s.to_owned()))
        .into_any_element()
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        self.viewport = (
            window.viewport_size().width.to_f64(),
            window.viewport_size().height.to_f64(),
        );
        self.sync_image_sources();
        if self.pending_image_pick {
            self.pending_image_pick = false;
            self.open_image_picker(cx);
        }
        if self.pending_scene_open {
            self.pending_scene_open = false;
            self.open_scene_picker(cx);
        }
        if self.pending_save_dialog {
            self.pending_save_dialog = false;
            self.open_save_dialog();
        }
        if self.pending_scene_save {
            self.pending_scene_save = false;
            self.save_scene_picker(cx);
        }
        let active_tool = self.document.tool;
        let zoom_pct = (self.document.zoom * 100.0).round() as i32;
        let t = |k: &str| self.i18n.t(k);

        // One stable handle for the view's lifetime; created on the first frame
        // because it needs an `App`, and focused immediately so shortcuts work
        // before the user has clicked anything.
        let focus_handle = self
            .focus_handle
            .get_or_insert_with(|| cx.focus_handle())
            .clone();
        if !focus_handle.is_focused(window) {
            focus_handle.focus(window, cx);
        }

        // ----- toolbar island -------------------------------------------------
        //
        // Upstream order: padlock, divider, eleven tools, divider, more-tools.
        // Image is deliberately *not* in this row — it lives behind the dots.
        let mut toolbar = chrome::toolbar_island(tokens)
            .occlude()
            .child(
                // Upstream flips this glyph with the state: a *closed* padlock
                // while the tool stays active, an open one otherwise.
                chrome::tool_button(
                    "tool-lock",
                    if self.document.tool_locked {
                        "LockedIcon"
                    } else {
                        "UnlockedIcon"
                    },
                    None,
                    self.document.tool_locked,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.tool_locked = !this.document.tool_locked;
                })),
            )
            .child(chrome::toolbar_divider(tokens, true));
        for &(tool, icon_name, shortcut) in TOOL_ROW {
            toolbar = toolbar.child(
                chrome::tool_button(
                    format!("tool-{icon_name}"),
                    icon_name,
                    shortcut,
                    active_tool == tool,
                    tokens,
                )
                .on_click(Self::act(cx, move |this, _, _, _| this.set_tool(tool))),
            );
        }
        let mut toolbar = toolbar.child(chrome::toolbar_divider(tokens, false)).child(
            chrome::tool_button("more-tools", "DotsIcon", None, self.show_more_tools, tokens)
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.set_more_tools_open(!this.show_more_tools);
                })),
        );

        if self.show_more_tools {
            // Upstream rows: six tools, then the bold "Generate" section header,
            // then the three AI rows — one flat popover, no inner scroll.
            let generate_rows = &MORE_TOOLS[7..];
            let label_of = |mt: &MoreTool| t(mt.key);
            toolbar = toolbar.child(
                // Upstream's `.App-toolbar__extra-tools-dropdown`: min-width
                // 11.875rem and a 1px stack gap, both narrower than the generic
                // menu surface this reuses.
                chrome::dropdown(tokens)
                    .gap(px(1.0))
                    .min_w(px(190.0))
                    .top(px(size::BUTTON_LG + size::TOOLBAR_PADDING * 2.0 + 8.0))
                    .right(px(size::TOOLBAR_PADDING))
                    .children(MORE_TOOLS[..7].iter().map(|mt| {
                        Self::more_tool_row(mt, &label_of(mt), tokens, cx).into_any_element()
                    }))
                    .child(
                        // Upstream renders this heading inline with `margin: 6px
                        // 0; font-size: 14px; font-weight: 600` and no horizontal
                        // inset, so it hangs one step left of the icon column.
                        div()
                            .mt(px(6.0))
                            .mb(px(6.0))
                            .text_size(px(14.0))
                            .font_bold()
                            .text_color(tokens.surface_text())
                            .child(SharedString::from(t("toolbar.generate"))),
                    )
                    .children(generate_rows.iter().map(|mt| {
                        Self::more_tool_row(mt, &label_of(mt), tokens, cx).into_any_element()
                    })),
            );
        }

        let scene = self.document.scene.clone();
        let images = self.image_sources.clone();
        let zoom = self.document.zoom;
        let scroll_x = self.document.scroll.x;
        let scroll_y = self.document.scroll.y;
        let sel_bounds = selection_bounds(&self.document.scene, &self.document.selected);
        let show_grid = self.document.show_grid;
        let grid_size = self.document.grid_size;
        let marquee_bounds = match (self.marquee_start, self.marquee_current) {
            (Some(start), Some(current)) => Some(Bounds::new(
                start.x.min(current.x),
                start.y.min(current.y),
                start.x.max(current.x),
                start.y.max(current.y),
            )),
            _ => None,
        };
        let laser = self.laser_points.clone();
        let linear_points = self.document.selected_linear_points().map(|(_, p)| p);
        let pending = self.pending_points.clone();
        let pending_cursor = self.cursor_world;
        // Live preview of the shape being dragged out. Without this a new
        // rectangle only materialises on mouse-up, which reads as the editor
        // ignoring the drag.
        let preview = self.drag_preview();
        let ink = self.freedraw_preview().map(|p| p.to_vec());
        let ink_width = self.document.style.stroke_width;
        let ink_color = crate::properties::parse_hex(&self.document.style.stroke_color);
        let selection_color = tokens.accent();

        // The canvas is full-bleed and sits at the bottom of the stack; every
        // island is painted over it. Painting uses window coordinates, so the
        // surface must start at the window origin for pan/zoom maths to hold.
        // The document's view background wins once it has been explicitly
        // picked; the default white falls through to the theme's canvas token
        // so dark mode keeps its near-black surface.
        let view_bg = if self.document.view_background_color == "#ffffff"
            || self.document.view_background_color == "transparent"
        {
            tokens.canvas()
        } else {
            crate::properties::parse_hex(&self.document.view_background_color)
        };
        let canvas_area = div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .bg(view_bg)
            .on_mouse_down(
                MouseButton::Left,
                Self::act(cx, |this, e: &MouseDownEvent, _, _| {
                    this.on_canvas_mouse_down(
                        e.position.x.to_f64() as f32,
                        e.position.y.to_f64() as f32,
                        e.modifiers,
                        e.click_count,
                    );
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                Self::act(cx, |this, e: &MouseDownEvent, _, _| {
                    let world = this
                        .screen_point(e.position.x.to_f64() as f32, e.position.y.to_f64() as f32);
                    if let Some(id) = this.document.element_at(world, 5.0 / this.document.zoom) {
                        if !this.document.selected.contains(&id) {
                            this.document.select(&id);
                        }
                    }
                    this.context_menu =
                        Some(Point::new(e.position.x.to_f64(), e.position.y.to_f64()));
                }),
            )
            .on_scroll_wheel(Self::act(cx, |this, e: &ScrollWheelEvent, _, _| {
                let (dx, dy) = match e.delta {
                    ScrollDelta::Pixels(p) => (p.x.to_f64(), p.y.to_f64()),
                    ScrollDelta::Lines(p) => (p.x as f64 * 20.0, p.y as f64 * 20.0),
                };
                if e.modifiers.control || e.modifiers.platform {
                    let old = this.document.zoom;
                    if dy > 0.0 {
                        this.on_zoom_out();
                    } else {
                        this.on_zoom_in();
                    }
                    // Keep the point under the cursor fixed while zooming.
                    let factor = this.document.zoom / old;
                    if factor != 1.0 {
                        let cx_pos = e.position.x.to_f64();
                        let cy_pos = e.position.y.to_f64();
                        this.document.scroll.x =
                            cx_pos - (cx_pos - this.document.scroll.x) * factor;
                        this.document.scroll.y =
                            cy_pos - (cy_pos - this.document.scroll.y) * factor;
                    }
                } else {
                    this.document.scroll.x -= dx;
                    this.document.scroll.y -= dy;
                }
            }))
            .child(
                canvas(
                    move |_bounds, _window, _cx| (),
                    move |bounds, _state, window, cx| {
                        if show_grid {
                            crate::canvas::paint_grid(
                                grid_size,
                                zoom,
                                scroll_x,
                                scroll_y,
                                (bounds.size.width.to_f64(), bounds.size.height.to_f64()),
                                window,
                            );
                        }
                        crate::canvas::paint_scene(
                            &scene, zoom, scroll_x, scroll_y, &images, window, cx,
                        );
                        if let Some(el) = &preview {
                            crate::canvas::paint_elements(
                                std::slice::from_ref(el),
                                zoom,
                                scroll_x,
                                scroll_y,
                                &images,
                                window,
                                cx,
                            );
                        }
                        if let Some(pts) = &ink {
                            crate::canvas::paint_ink(
                                pts, ink_width, ink_color, zoom, scroll_x, scroll_y, window,
                            );
                        }
                        if let Some(pts) = &linear_points {
                            crate::canvas::paint_linear_handles(
                                pts, zoom, scroll_x, scroll_y, window,
                            );
                        }
                        if !pending.is_empty() {
                            crate::canvas::paint_guide(
                                &pending,
                                pending_cursor,
                                zoom,
                                scroll_x,
                                scroll_y,
                                window,
                            );
                        }
                        if let Some(b) = &sel_bounds {
                            crate::canvas::paint_selection_overlay(
                                b,
                                selection_color,
                                zoom,
                                scroll_x,
                                scroll_y,
                                window,
                            );
                        }
                        if !laser.is_empty() {
                            crate::canvas::paint_laser(&laser, zoom, scroll_x, scroll_y, window);
                        }
                        if let Some(b) = &marquee_bounds {
                            crate::canvas::paint_marquee(
                                b,
                                selection_color,
                                zoom,
                                scroll_x,
                                scroll_y,
                                window,
                            );
                        }
                    },
                )
                .size_full(),
            );

        // ----- hint bar -------------------------------------------------------
        //
        // Excalidraw prints the "move canvas…" hint under the toolbar. A status
        // message (saved / loaded) takes its place while it is set.
        let hint_text = self.status.clone().unwrap_or_else(|| t("hint.moveCanvas"));
        // Key names arrive wrapped in `{{…}}` so they can be drawn as `<kbd>`
        // chips while everything between them stays plain hint text — exactly
        // how upstream splits `hold <kbd>Scroll wheel</kbd> or <kbd>Space</kbd>`.
        let hint_children: Vec<AnyElement> = split_hint(&hint_text)
            .into_iter()
            .map(|(run, is_key)| {
                if is_key {
                    chrome::kbd_chip(run, tokens).into_any_element()
                } else {
                    plain_text(&run)
                }
            })
            .collect();
        // Upstream hangs two different things off the toolbar's bottom edge:
        // the hint 8px below it (island bottom 60, hint top 68) and the side
        // panels a full container-padding below it (panel top 76). They differ,
        // so they get separate offsets rather than one shared "dock".
        let toolbar_bottom =
            size::CONTAINER_PADDING + size::TOOLBAR_PADDING * 2.0 + size::BUTTON_LG;
        let dock_top = toolbar_bottom + size::CONTAINER_PADDING;
        let hint_top = toolbar_bottom + 8.0;
        let hint_bar = div()
            .absolute()
            .left(px(0.0))
            .right(px(0.0))
            .top(px(hint_top))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_size(px(12.0))
                    .text_color(tokens.shortcut_text())
                    .children(hint_children),
            );

        // ----- top-left: hamburger -------------------------------------------
        let menu_island = chrome::rail_island(tokens)
            .absolute()
            .left(px(size::CONTAINER_PADDING))
            .top(px(size::CONTAINER_PADDING))
            .occlude()
            .child(
                chrome::tool_button("menu", "HamburgerMenuIcon", None, self.show_menu, tokens)
                    .on_click(Self::act(cx, |this, _, _, _| {
                        this.set_menu_open(!this.show_menu);
                    })),
            );

        // ----- top-centre: the toolbar ---------------------------------------
        // A transparent, full-width row that only centres its child; it has no
        // hitbox of its own, so clicks in the rest of the strip reach the canvas.
        let top_centre = div()
            .absolute()
            .left(px(0.0))
            .right(px(0.0))
            .top(px(size::CONTAINER_PADDING))
            .flex()
            .flex_row()
            .justify_center()
            .child(toolbar);

        // ----- top-right: library + theme ------------------------------------
        let right_rail = chrome::rail_island(tokens)
            .absolute()
            .right(px(size::CONTAINER_PADDING))
            .top(px(size::CONTAINER_PADDING))
            .occlude()
            .child(
                chrome::tool_button(
                    "library",
                    "sidebarRightIcon",
                    None,
                    self.document.show_library,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| this.on_toggle_library())),
            )
            .child(
                chrome::tool_button(
                    "theme",
                    if tokens.palette.is_dark() {
                        "SunIcon"
                    } else {
                        "MoonIcon"
                    },
                    None,
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| this.on_toggle_theme())),
            );

        // ----- bottom-left: zoom + history -----------------------------------
        let undo_enabled = self.history.can_undo();
        let redo_enabled = self.history.can_redo();
        // ----- footer ---------------------------------------------------------
        //
        // Two chips, 10px apart, sitting 16px off the bottom-left corner.
        // Upstream: the zoom chip is 132×36 at x=16 and undo/redo starts at
        // x=158, which is where the 10px comes from.
        let footer = div()
            .absolute()
            .left(px(size::CONTAINER_PADDING))
            .bottom(px(size::CONTAINER_PADDING))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .child(
                chrome::rail_island(tokens)
                    .occlude()
                    .child(
                        chrome::chrome_button("zoom-out", "ZoomOutIcon", true, tokens)
                            .on_click(Self::act(cx, |this, _, _, _| this.on_zoom_out())),
                    )
                    .child(
                        chrome::zoom_readout(tokens)
                            .child(SharedString::from(format!("{zoom_pct}%"))),
                    )
                    .child(
                        chrome::chrome_button("zoom-in", "ZoomInIcon", true, tokens)
                            .on_click(Self::act(cx, |this, _, _, _| this.on_zoom_in())),
                    ),
            )
            .child(
                chrome::rail_island(tokens)
                    .occlude()
                    .child(
                        chrome::chrome_button("undo", "UndoIcon", undo_enabled, tokens)
                            .on_click(Self::act(cx, |this, _, _, _| this.on_undo())),
                    )
                    .child(
                        chrome::chrome_button("redo", "RedoIcon", redo_enabled, tokens)
                            .on_click(Self::act(cx, |this, _, _, _| this.on_redo())),
                    ),
            );

        // ----- side islands ---------------------------------------------------
        let panel_limit =
            (window.viewport_size().height.to_f64() as f32 - dock_top - 16.0).max(120.0);

        let properties = if self.document.selected.is_empty() {
            div().into_any_element()
        } else {
            crate::properties::render_properties_panel(self, cx)
                .absolute()
                .left(px(size::CONTAINER_PADDING))
                .top(px(dock_top))
                .max_h(px(panel_limit))
                .occlude()
                .into_any_element()
        };

        let library_panel = if self.document.show_library {
            crate::library::render_library_panel(self, cx)
                .absolute()
                .right(px(size::CONTAINER_PADDING))
                .top(px(dock_top))
                .max_h(px(panel_limit))
                .occlude()
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let layers_panel = if self.document.show_layers {
            crate::layers::render_layers_panel(self, cx)
                .absolute()
                .right(px(size::CONTAINER_PADDING))
                .bottom(px(size::CONTAINER_PADDING + size::BUTTON_LG + 8.0))
                .max_h(px(panel_limit))
                .occlude()
                .into_any_element()
        } else {
            div().into_any_element()
        };

        // ----- Canvas & shape properties island -----------------------------------
        let stats_island = if self.document.show_stats {
            self.render_stats_island(tokens).into_any_element()
        } else {
            div().into_any_element()
        };

        // ----- Zen mode ----------------------------------------------------------
        // Every island disappears — canvas only (plus the stats island if it
        // was already armed). Alt+Z brings the chrome back.
        let zen = self.document.zen_mode;

        let mut root = div()
            .size_full()
            .overflow_hidden()
            .bg(tokens.canvas())
            .track_focus(&focus_handle)
            // Keys are delivered along the focused node's dispatch path. Nothing
            // else in this view is focusable, so the editor anchors focus on
            // itself; without that anchor GPUI falls back to the window's own
            // root node and a listener on a descendant never sees a keystroke.
            .on_key_down(Self::act(cx, |this, e: &KeyDownEvent, window, cx| {
                let key = e.keystroke.key.clone();
                let char = e.keystroke.key_char.clone();
                let modifiers = e.keystroke.modifiers;
                // Paste owns Ctrl+V before `handle_key` ever sees it, because
                // `handle_key` has no clipboard access and its text branch
                // rejects every modified keystroke outright.
                if (modifiers.control || modifiers.platform) && !modifiers.shift && key == "v" {
                    if this.editing_text.is_none() && this.document.has_clipboard() {
                        // An in-app copy wins: falling through reaches
                        // `on_paste`, which duplicates the element clipboard —
                        // what Ctrl+C followed by Ctrl+V has to do. Letting the
                        // system clipboard take this keystroke instead would
                        // paste whatever stale text it happens to hold.
                    } else {
                        // While editing, an image on the clipboard must not
                        // become an image element: upstream's textarea pastes
                        // text and ignores the image.
                        if this.editing_text.is_none() && this.paste_image_from_clipboard(cx) {
                            return;
                        }
                        if this.paste_text_from_clipboard(cx) {
                            return;
                        }
                    }
                }
                let size = window.viewport_size();
                let vp = (size.width.to_f64(), size.height.to_f64());
                // The hint mirrors a tool switch, so it must observe the key
                // *before* the switch mutates the tool (upstream's
                // `CursorHints.onToolShortcut` runs from the same keydown).
                this.note_tool_shortcut(&key, modifiers, cx);
                this.handle_key(&key, char, modifiers, vp);
            }))
            .on_key_up(Self::act(cx, |this, e: &KeyUpEvent, _, _| {
                if e.keystroke.key == "space" {
                    this.space_pan = false;
                    this.panning = false;
                }
            }))
            // Pointer fallbacks. The islands occlude the canvas, so a drag that
            // strays over the toolbar would otherwise stop receiving motion and
            // strand the gesture. These only forward while something is live,
            // so hovering the chrome never leaks through to the canvas.
            .on_mouse_move(Self::act(cx, |this, e: &MouseMoveEvent, _, _| {
                if this.is_interacting() {
                    this.on_canvas_mouse_move(
                        e.position.x.to_f64() as f32,
                        e.position.y.to_f64() as f32,
                        e.modifiers,
                    );
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                Self::act(cx, |this, e: &MouseUpEvent, _, _| {
                    if this.is_interacting() {
                        this.on_canvas_mouse_up(
                            e.position.x.to_f64() as f32,
                            e.position.y.to_f64() as f32,
                            e.modifiers,
                        );
                    }
                }),
            )
            // An OS file drag has no "enter" event of its own: the platform
            // parks its paths in `cx.active_drag` and synthesises pointer
            // motion, so this capture-phase hook is what keeps the overlay
            // tracking the modifier the user is holding over the window.
            .on_drag_move::<ExternalPaths>(Self::act(
                cx,
                |this, e: &DragMoveEvent<ExternalPaths>, _, cx| {
                    let paths = e.drag(cx).paths().to_vec();
                    this.on_external_drag(&paths, e.event.modifiers.shift);
                },
            ))
            .on_file_drop_exit(Self::act(cx, |this, _: &FileDropEvent, _, _| {
                this.on_external_drag_exit();
            }))
            .on_drop(Self::act(cx, |this, paths: &ExternalPaths, _, cx| {
                if this.handle_external_drop(paths) {
                    // The toast is plain text, so the shortcut markers the
                    // file-drop hint renders as `<kbd>` chips are unfolded
                    // here instead of being shown as braces.
                    let message: String = split_hint(&this.i18n.t("fileDrop.replacedToast"))
                        .into_iter()
                        .map(|(run, _)| run)
                        .collect();
                    this.show_toast(message, true, 8000, cx);
                }
            }))
            .child(canvas_area);

        // Element-attached controls ride just above the canvas and below the
        // islands, matching upstream's low z-index for them. Their order is
        // upstream's: canvas buttons, then the unlock bubble, then the shape
        // switcher.
        if !zen {
            if let Some(panel) = self.render_canvas_buttons(cx) {
                root = root.child(panel);
            }
            if let Some(popup) = self.render_unlock_popup(cx) {
                root = root.child(popup);
            }
            if let Some(panel) = self.render_shape_switch(cx) {
                root = root.child(panel);
            }
        }

        // Welcome screen — upstream's centred logo, persistence note and quick
        // actions show whenever the scene is empty, exactly like the site.
        if !zen && self.document.scene.is_empty() {
            root = root.child(self.render_welcome(cx));
        }

        if !zen {
            root = root
                .child(hint_bar)
                .child(menu_island)
                .child(top_centre)
                .child(right_rail)
                .child(footer)
                .child(properties)
                .child(library_panel)
                .child(layers_panel);

            // The pickers are opened from the properties panel, so they dock one
            // gap to its right rather than over the canvas. The link dialog is
            // the exception: upstream pins it to the container's own corner.
            let popup_left =
                size::CONTAINER_PADDING + size::PROPERTIES_PANEL_WIDTH + size::MENU_PANEL_GAP;
            // The canvas-background trigger sits in the hamburger menu rather
            // than the properties panel, so its picker docks beside that menu —
            // the same spot the language list uses, upstream's popover being
            // anchored to the swatch.
            let popup_left = if self.color_popup() == Some(ColorTarget::CanvasBackground) {
                size::CONTAINER_PADDING + size::MENU_WIDTH + size::MENU_PANEL_GAP
            } else {
                popup_left
            };
            let popup_top = if self.color_popup() == Some(ColorTarget::CanvasBackground) {
                size::CONTAINER_PADDING + size::BUTTON_LG + 8.0
            } else {
                dock_top
            };
            if self.color_popup().is_some() {
                root = root.child(
                    crate::properties::render_color_picker(self, cx)
                        .absolute()
                        .left(px(popup_left))
                        .top(px(popup_top))
                        .max_h(px(panel_limit))
                        .occlude(),
                );
            }
            if self.is_font_popup_open() {
                root = root.child(
                    crate::properties::render_font_picker(self, cx)
                        .absolute()
                        .left(px(popup_left))
                        .top(px(dock_top))
                        .max_h(px(panel_limit))
                        .occlude(),
                );
            }
            if self.is_link_dialog_open() {
                root = root.child(
                    crate::properties::render_link_dialog(self, cx)
                        .absolute()
                        .left(px(size::CONTAINER_PADDING))
                        .top(px(size::CONTAINER_PADDING))
                        .occlude(),
                );
            }
        }
        root = root.child(stats_island);

        if self.show_menu {
            root = root.child(self.render_file_menu(cx));
            if self.show_prefs {
                root = root.child(self.render_prefs_panel(cx));
            }
            // The language options dock beside the menu rather than inside it:
            // the menu is a scroll container and would clip a popover, which is
            // exactly what a native `<select>` never suffers from on the web.
            if self.show_lang {
                root = root.child(self.render_language_panel(cx));
            }
        }
        if self.context_menu.is_some() {
            root = root.child(self.render_context_menu(cx));
        }
        // Modal overlays paint last so they sit above every island, and the
        // palette above the other two — it is the one opened on purpose.
        if self.show_find {
            root = root.child(self.render_find(cx));
        }
        if self.show_help {
            root = root.child(self.render_help(cx));
        }
        if self.show_save_dialog {
            root = root.child(self.render_save_dialog(cx));
        }
        if self.show_palette {
            root = root.child(self.render_palette(cx));
        }
        // Transient overlays. Their order is upstream's z-index ladder, which
        // only these four share: eye dropper (5–6), cursor hint (8), file drop
        // (130). Each is painted over the islands, so a later child wins.
        if let Some(overlay) = self.render_eye_dropper(cx) {
            root = root.child(overlay);
        }
        if let Some(hint) = self.render_cursor_hint(tokens) {
            root = root.child(hint);
        }
        if self.file_drag.is_some() {
            root = root.child(self.render_file_drop_overlay(tokens));
        }
        // Last, so a live gesture keeps its pointer capture above every island
        // that might otherwise swallow the release.
        if self.is_interacting() {
            root = root.child(self.render_gesture_capture(cx));
        }
        // The toast sits at the top of the ladder (upstream `--zIndex-toast`),
        // above even the gesture capture — it must be dismissable mid-drag.
        if let Some(toast) = self.render_toast(tokens, cx) {
            root = root.child(toast);
        }

        root
    }
}

impl Editor {
    /// One row of the "more tools" popover: label, optional shortcut, optional
    /// purple "AI" chip, and the action behind it.
    fn more_tool_row(
        mt: &MoreTool,
        label: &str,
        tokens: crate::design::Tokens,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let mut row = chrome::menu_row_with(
            format!("more-{}", mt.icon),
            Some(mt.icon),
            label,
            mt.shortcut,
            false,
            tokens,
        );
        if mt.ai {
            // Upstream `DropDownMenuItemBadge`: a primary-filled pill in the
            // 9px Cascadia/monospace face, ringed with a white hairline in the
            // light theme only. Sizes are the literal px from its inline style,
            // not the rem-derived tokens the rest of the chrome uses.
            let badge = div()
                .flex_shrink_0()
                .px(px(4.0))
                .py(px(2.0))
                .mr(px(2.0))
                .rounded(px(6.0))
                .bg(tokens.accent())
                .font_family("monospace")
                .text_size(px(9.0))
                .text_color(tokens.surface_lowest())
                .child("AI");
            row = row.child(if tokens.palette.is_dark() {
                badge
            } else {
                badge.border_1().border_color(rgb(0xffffff))
            });
        }
        match &mt.action {
            MoreToolAction::Tool(tool) => {
                let tool = *tool;
                row.on_click(Self::act(cx, move |this, _, _, _| {
                    this.set_tool(tool);
                    this.show_more_tools = false;
                }))
            }
            MoreToolAction::Embed => row.on_click(Self::act(cx, |this, _, _, _| {
                // Upstream drops an embed card at the viewport centre; without
                // a link picker yet we place the default-size card where the
                // user is looking.
                this.push_checkpoint();
                let (vw, vh) = (800.0, 600.0);
                let x = (vw / 2.0 - this.document.scroll.x) / this.document.zoom - 200.0;
                let y = (vh / 2.0 - this.document.scroll.y) / this.document.zoom - 150.0;
                let opts = this.current_style_options();
                let el = excalidraw_core::factory::new_embeddable(x, y, 400.0, 300.0, &opts);
                let id = el.id().to_string();
                this.document.scene.add(el);
                this.document.select(&id);
                this.show_more_tools = false;
            })),
            MoreToolAction::Stub => row.on_click(Self::act(cx, move |this, _, _, _| {
                this.status = Some(this.i18n.t("status.unavailable"));
                this.show_more_tools = false;
            })),
        }
    }

    /// A Preferences segmented control — label on the left, a two-option pill group
    /// on the right ("Select on [Wrap|Overlap]", "Input [Trackpad|Mouse]").
    /// The active option is a solid primary pill, exactly like upstream's
    /// RadioGroup__choice pair.
    fn prefs_segment(
        id_base: &'static str,
        label: &str,
        options: [&'static str; 2],
        active: usize,
        tokens: crate::design::Tokens,
        cx: &mut Context<Self>,
        on_pick: impl Fn(&mut Self, usize) + 'static + Copy,
    ) -> impl IntoElement {
        let mut row = div()
            .id(SharedString::from(format!("{id_base}-row")))
            .test_support()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .w_full()
            .min_h(px(size::MENU_ROW))
            .px(px(8.0))
            .text_size(px(14.0))
            .child(div().flex_1().child(SharedString::from(label.to_owned())));
        let mut group = div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .gap(px(2.0))
            .p(px(2.0))
            .rounded(px(8.0))
            .bg(tokens.hover());
        for (i, opt) in options.iter().enumerate() {
            let is_active = i == active;
            let mut pill = div()
                .id(SharedString::from(format!("{id_base}-opt{i}")))
                .test_support()
                .px(px(8.0))
                .py(px(3.0))
                .rounded(px(6.0))
                .text_size(px(12.0))
                .child(SharedString::from((*opt).to_owned()));
            pill = if is_active {
                pill.bg(tokens.primary_darkest()).text_color(rgb(0xffffff))
            } else {
                pill.text_color(tokens.surface_text())
                    .hover(move |s| s.bg(tokens.separator()))
            };
            group = group.child(pill.on_click(Self::act(cx, move |this, _, _, _| {
                on_pick(this, i);
            })));
        }
        row = row.child(group);
        row
    }

    /// A small square icon choice inside the Theme row (sun / moon / monitor).
    fn theme_choice(
        id: &'static str,
        icon_name: &'static str,
        active: bool,
        tokens: crate::design::Tokens,
        cx: &mut Context<Self>,
        on_pick: impl Fn(&mut Self) + 'static + Copy,
    ) -> impl IntoElement {
        let glyph = if active {
            rgb(0xffffff)
        } else {
            tokens.surface_text()
        };
        let mut pill = div()
            .id(SharedString::from(id))
            .test_support()
            .size(px(26.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(6.0))
            .text_color(glyph);
        pill = if active {
            pill.bg(tokens.primary_darkest())
        } else {
            pill.hover(move |s| s.bg(tokens.separator()))
        };
        pill.child(crate::icons::icon(icon_name, 16.0, glyph))
            .on_click(Self::act(cx, move |this, _, _, _| on_pick(this)))
    }

    /// The welcome screen — upstream's centred logo, the three-line storage
    /// note and the quick-action rows, shown whenever the scene is empty.
    ///
    /// The overlay wrapper has no hitbox of its own (same trick as the toolbar
    /// strip), so clicks land on the canvas until they hit an actual row.
    fn render_welcome(&self, cx: &mut Context<Self>) -> Div {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let fg = tokens.shortcut_text();
        let brand = if tokens.palette.is_dark() {
            crate::properties::parse_hex("#a7b6ff")
        } else {
            crate::properties::parse_hex("#6965db")
        };

        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(24.0))
            .child(
                // The X mark and the wordmark, one row, brand-tinted.
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(10.0))
                    .child(crate::icons::icon("logoMark", 40.0, brand))
                    .child(
                        svg()
                            .path(crate::icons::icon_path("logoWord"))
                            .flex_shrink_0()
                            .w(px(186.0))
                            .h(px(23.0))
                            .text_color(brand),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(2.0))
                    .text_size(px(14.0))
                    .text_color(fg)
                    .child(SharedString::from(t("welcome.center1")))
                    .child(SharedString::from(t("welcome.center2")))
                    .child(SharedString::from(t("welcome.center3"))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .id("wl-open")
                            .test_support()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .w(px(300.0))
                            .min_h(px(40.0))
                            .px(px(8.0))
                            .rounded(px(8.0))
                            .text_size(px(14.0))
                            .text_color(fg)
                            .hover(move |s| s.bg(tokens.hover()))
                            .child(crate::icons::icon("LoadIcon", 18.0, fg))
                            .child(div().flex_1().child(SharedString::from(t("menu.open"))))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .child(SharedString::from("Ctrl+O")),
                            )
                            .on_click(Self::act(cx, |this, _, _, _| {
                                this.pending_scene_open = true;
                            })),
                    )
                    .child(
                        div()
                            .id("wl-help")
                            .test_support()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .w(px(300.0))
                            .min_h(px(40.0))
                            .px(px(8.0))
                            .rounded(px(8.0))
                            .text_size(px(14.0))
                            .text_color(fg)
                            .hover(move |s| s.bg(tokens.hover()))
                            .child(crate::icons::icon("helpIcon", 18.0, fg))
                            .child(div().flex_1().child(SharedString::from(t("menu.help"))))
                            .child(div().text_size(px(12.0)).child(SharedString::from("?")))
                            .on_click(Self::act(cx, |this, _, _, _| {
                                this.status = Some(this.i18n.t("menu.helpHint"));
                            })),
                    )
                    .child(
                        div()
                            .id("wl-collab")
                            .test_support()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .w(px(300.0))
                            .min_h(px(40.0))
                            .px(px(8.0))
                            .rounded(px(8.0))
                            .text_size(px(14.0))
                            .text_color(fg)
                            .hover(move |s| s.bg(tokens.hover()))
                            .child(crate::icons::icon("collabIcon", 18.0, fg))
                            .child(div().flex_1().child(SharedString::from(t("menu.collab"))))
                            .on_click(Self::act(cx, |this, _, _, _| {
                                this.status = Some(this.i18n.t("status.unavailable"));
                            })),
                    )
                    .child(
                        div()
                            .id("wl-signup")
                            .test_support()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .w(px(300.0))
                            .min_h(px(40.0))
                            .px(px(8.0))
                            .rounded(px(8.0))
                            .text_size(px(14.0))
                            .text_color(fg)
                            .hover(move |s| s.bg(tokens.hover()))
                            .child(crate::icons::icon("signUpIcon", 18.0, fg))
                            .child(div().flex_1().child(SharedString::from(t("menu.signUp"))))
                            .on_click(Self::act(cx, |_, _, _, _| {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg("https://excalidraw.com")
                                    .spawn();
                            })),
                    ),
            )
    }

    /// The Canvas & shape properties island (upstream's stats dialog, panel form): scene
    /// totals plus the selected element's geometry.
    fn render_stats_island(&self, tokens: crate::design::Tokens) -> Div {
        let t = |k: &str| self.i18n.t(k);
        let fg = tokens.surface_text();
        let muted = tokens.shortcut_text();

        let mut rows = vec![(
            t("stats.shapes"),
            self.document.scene.non_deleted().count().to_string(),
        )];
        if let Some(el) = self
            .document
            .scene
            .non_deleted()
            .find(|e| self.document.selected.contains(e.id()))
        {
            let b = el.base();
            rows.push((t("stats.selectedType"), format!("{:?}", el.kind())));
            rows.push(("x".to_string(), format!("{:.0}", b.x)));
            rows.push(("y".to_string(), format!("{:.0}", b.y)));
            rows.push(("w".to_string(), format!("{:.0}", b.width)));
            rows.push(("h".to_string(), format!("{:.0}", b.height)));
            rows.push(("∠".to_string(), format!("{:.1}°", b.angle.to_degrees())));
        }

        let mut island = chrome::rail_island(tokens)
            .absolute()
            .right(px(size::CONTAINER_PADDING))
            .top(px(size::CONTAINER_PADDING + size::BUTTON_LG + 16.0))
            .w(px(200.0))
            .p(px(12.0))
            .occlude()
            .child(
                div()
                    .font_bold()
                    .text_size(px(13.0))
                    .text_color(fg)
                    .pb(px(6.0))
                    .child(SharedString::from(t("stats.title"))),
            );
        for (label, value) in rows {
            island = island.child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .py(px(1.0))
                    .text_size(px(12.0))
                    .child(div().text_color(muted).child(SharedString::from(label)))
                    .child(div().text_color(fg).child(SharedString::from(value))),
            );
        }
        island
    }

    /// The hamburger dropdown (`.dropdown`), anchored under the menu island.
    ///
    /// Row order mirrors excalidraw.com's hamburger exactly (verified against
    /// the live zh-CN DOM): file actions, promo rows, destructive action, then
    /// the brand links, then Preferences with its second-level panel, the theme
    /// segmented control, the language and the canvas background.
    fn render_file_menu(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        // 8px under the hamburger chip, which is 36px tall at y=16.
        let top = size::CONTAINER_PADDING + size::BUTTON_LG + 8.0;

        let rule = || {
            div()
                .h(px(1.0))
                .my(px(4.0))
                .mx(px(4.0))
                .bg(tokens.separator())
        };
        let promo = tokens.primary_darkest();

        chrome::dropdown(tokens)
            .id("file-menu")
            .test_support()
            .w(px(size::MENU_WIDTH))
            .left(px(size::CONTAINER_PADDING))
            .top(px(top))
            // `.dropdown-menu-container`: `overflow-y: auto` with
            // `max-height: calc(100svh - editor-container-padding * 2 - lg-button-size)`.
            // Without the cap the menu runs past the bottom of a short window and
            // the last sections — the language select included — become
            // unreachable.
            .max_h(px(self.menu_max_height()))
            .overflow_y_scroll()
            .child(
                chrome::menu_row_with(
                    "fm-open",
                    Some("LoadIcon"),
                    &t("menu.open"),
                    Some("Ctrl+O"),
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.pending_scene_open = true;
                    this.show_menu = false;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "fm-save",
                    Some("ExportIcon"),
                    &t("menu.save"),
                    None,
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    // Upstream's `Export` menu row only raises the dialog
                    // (`setAppState({ openDialog: { name: "jsonExport" } })`);
                    // the card's button is what writes the file.
                    this.pending_save_dialog = true;
                    this.show_menu = false;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "fm-png",
                    Some("ImageIcon"),
                    &t("menu.exportImage"),
                    Some("Ctrl+Shift+E"),
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    let _ = this.export_png_to("excalidraw-export.png");
                    this.show_menu = false;
                })),
            )
            .child(
                chrome::menu_row_colored(
                    "fm-palette",
                    Some("commandPaletteIcon"),
                    &t("menu.commandPalette"),
                    Some("Ctrl+/"),
                    false,
                    tokens,
                    promo,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.open_palette();
                })),
            )
            .child(
                chrome::menu_row_with(
                    "fm-find",
                    Some("searchIcon"),
                    &t("menu.findOnCanvas"),
                    Some("Ctrl+F"),
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.open_find();
                })),
            )
            .child(
                chrome::menu_row_with(
                    "fm-help",
                    Some("HelpIcon"),
                    &t("menu.help"),
                    Some("?"),
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.open_help();
                })),
            )
            .child(
                chrome::menu_row_with(
                    "fm-reset",
                    Some("TrashIcon"),
                    &t("menu.reset"),
                    None,
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.on_new_document();
                    this.show_menu = false;
                })),
            )
            .child(rule())
            // Preferences — opens the second-level panel to the right; the menu
            // itself stays open, exactly like upstream.
            .child(
                chrome::menu_row_with(
                    "fm-prefs",
                    Some("slidersIcon"),
                    &t("menu.preferences"),
                    Some("›"),
                    false,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.show_prefs = !this.show_prefs;
                })),
            )
            .child(
                // Theme — the three-way segmented control (sun / moon / monitor).
                div()
                    .id("fm-theme-row")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.0))
                    .w_full()
                    .min_h(px(size::MENU_ROW))
                    .px(px(8.0))
                    .text_size(px(14.0))
                    .child(div().flex_1().child(SharedString::from(t("menu.theme"))))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_shrink_0()
                            .gap(px(2.0))
                            .p(px(2.0))
                            .rounded(px(8.0))
                            .bg(tokens.hover())
                            .child(Self::theme_choice(
                                "theme-light",
                                "themeLightIcon",
                                self.document.theme == Theme::Light,
                                tokens,
                                cx,
                                |this| this.document.set_theme(Theme::Light),
                            ))
                            .child(Self::theme_choice(
                                "theme-dark",
                                "themeDarkIcon",
                                self.document.theme == Theme::Dark,
                                tokens,
                                cx,
                                |this| this.document.set_theme(Theme::Dark),
                            ))
                            .child(Self::theme_choice(
                                "theme-system",
                                "themeSystemIcon",
                                false,
                                tokens,
                                cx,
                                |this| this.on_toggle_theme(),
                            )),
                    ),
            )
            .child(
                Self::language_select(SharedString::from(self.i18n.language().label()), tokens)
                    .on_click(Self::act(cx, |this, _, _, _| {
                        this.show_lang = !this.show_lang;
                        this.show_prefs = false;
                    })),
            )
            // Canvas background — upstream's `ColorPicker` with
            // `palette={null}`: the five canvas tints, a hairline, then the
            // active-colour trigger that opens the hex picker. Picking a tint
            // keeps the menu open, exactly like the web app.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .px(px(12.0))
                    .py(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(tokens.shortcut_text())
                            .child(SharedString::from(t("menu.canvasBackground"))),
                    )
                    .child({
                        let mut picks = div().flex().flex_row().items_center().gap(px(5.0));
                        for hex in ["#ffffff", "#f8f9fa", "#f5faff", "#fffce8", "#fdf8f6"] {
                            let active = self
                                .document
                                .view_background_color
                                .eq_ignore_ascii_case(hex);
                            let mut swatch = div()
                                .id(SharedString::from(format!(
                                    "viewbg-{}",
                                    hex.trim_start_matches('#')
                                )))
                                .test_support()
                                .size(px(22.0))
                                .rounded(px(4.0))
                                .bg(crate::properties::parse_hex(hex));
                            swatch = if active {
                                swatch.border_1().border_color(tokens.primary_darkest())
                            } else if crate::properties::needs_outline(hex) {
                                // All five tints are pale enough that upstream's
                                // `has-outline` gives each one a hairline, so
                                // none of them dissolves into the menu surface.
                                swatch.border_1().border_color(tokens.swatch_outline())
                            } else {
                                swatch
                            };
                            picks = picks.child(swatch.on_click(Self::act(
                                cx,
                                move |this, _, _, _| {
                                    this.document.view_background_color = hex.to_string();
                                },
                            )));
                        }
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            // Upstream's `.color-picker-container` is a
                            // three-column grid with no gap — the 5px spacing
                            // lives inside the picks strip — so the hairline
                            // column sits flush against both neighbours.
                            .child(picks)
                            .child(crate::properties::button_separator(tokens))
                            // Upstream closes this row with the active-colour
                            // trigger rather than a sixth preset: it wears
                            // whatever the canvas currently uses and is the only
                            // way into the custom (hex) picker.
                            .child(crate::properties::color_trigger(
                                "viewbg-custom",
                                ColorTarget::CanvasBackground,
                                &self.document.view_background_color,
                                self.color_popup() == Some(ColorTarget::CanvasBackground),
                                tokens,
                                cx,
                            ))
                    }),
            )
            .into_any_element()
    }

    /// The language `<select>` (`.dropdown-select__language`).
    ///
    /// Upstream's is a native `<select>` spanning the menu's width: a 32px
    /// island-coloured box with a 1px hairline, 4px radius, the current
    /// language's own name, and the `--dropdown-icon` triangle pinned to the
    /// right edge.
    fn language_select(label: SharedString, tokens: Tokens) -> chrome::Button {
        let fg = tokens.surface_text();
        let caret = size::language_select::CARET;
        div()
            .id("fm-lang")
            .test_support()
            .relative()
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(size::language_select::HEIGHT))
            .pl(px(size::language_select::PAD_START))
            .pr(px(size::language_select::PAD_END))
            .rounded(px(size::language_select::RADIUS))
            .border_1()
            .border_color(tokens.island_border())
            .bg(tokens.island())
            .text_size(px(size::language_select::FONT))
            .text_color(fg)
            .child(div().flex_1().child(label))
            .child(
                div()
                    .absolute()
                    .right(px(size::language_select::CARET_RIGHT))
                    .top(px(size::language_select::CARET_TOP))
                    .child(crate::icons::icon("dropdownIcon", caret, fg)),
            )
            .hover(move |s| s.bg(tokens.hover()))
    }

    /// The option list a `<select>` opens.
    ///
    /// The web app gets this for free from the OS: a native select paints its
    /// options over the page, outside any scroll container. GPUI has no such
    /// control, so the list is a floating panel beside the menu — the same
    /// second-level-panel idiom Preferences already uses, and the one place it cannot
    /// be clipped by the menu's own scrollbar.
    fn render_language_panel(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = Tokens::for_palette(palette_of(self.document.theme));
        let current = self.i18n.language();
        let top = size::CONTAINER_PADDING + size::BUTTON_LG + 8.0;
        let mut panel = chrome::dropdown(tokens)
            .id("lang-list")
            .test_support()
            .left(px(size::CONTAINER_PADDING
                + size::MENU_WIDTH
                + size::MENU_PANEL_GAP))
            .top(px(top))
            // Upstream's native `<select>` paints all 25 locales over the page;
            // ours is a panel, so it needs the same cap the hamburger has or the
            // tail of the list falls off a short window.
            .max_h(px(self.menu_max_height()))
            .overflow_y_scroll();
        for language in Language::all() {
            let selected = language == current;
            let fg = if selected {
                tokens.selected_fg()
            } else {
                tokens.surface_text()
            };
            let row = div()
                .id(SharedString::from(format!("lang-{}", language.code())))
                .test_support()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .min_h(px(size::MENU_ROW))
                .px(px(8.0))
                .rounded(px(size::RADIUS_MD))
                .text_size(px(14.0))
                .text_color(fg)
                .child(SharedString::from(language.label()));
            let row = if selected {
                row.bg(tokens.selected_bg())
            } else {
                row.hover(move |s| s.bg(tokens.hover()))
            };
            panel = panel.child(row.on_click(Self::act(cx, move |this, _, _, _| {
                this.set_language(language);
                this.show_lang = false;
            })));
        }
        panel.into_any_element()
    }

    /// The Preferences second-level panel — anchored right of the file menu.
    ///
    /// Upstream opens "Select on / Input / Tool lock / Snap to objects / Toggle
    /// grid / Zen mode / View mode / Canvas & shape properties / Arrow binding /
    /// Snap to midpoints" here; toggles flip in place without dismissing either
    /// panel.
    fn render_prefs_panel(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let top = size::CONTAINER_PADDING + size::BUTTON_LG + 8.0;
        chrome::dropdown(tokens)
            .min_w(px(250.0))
            .left(px(size::CONTAINER_PADDING
                + size::MENU_WIDTH
                + size::MENU_PANEL_GAP))
            .top(px(top))
            .child(Self::prefs_segment(
                "prefs-select",
                &t("prefs.selectOn"),
                ["Wrap", "Overlap"],
                if self.document.select_wrap { 0 } else { 1 },
                tokens,
                cx,
                |this, i| this.document.select_wrap = i == 0,
            ))
            .child(Self::prefs_segment(
                "prefs-input",
                &t("prefs.input"),
                ["Trackpad", "Mouse"],
                if self.document.input_trackpad { 0 } else { 1 },
                tokens,
                cx,
                |this, i| this.document.input_trackpad = i == 0,
            ))
            .child(
                chrome::menu_row_with(
                    "prefs-lock",
                    None,
                    &t("menu.toolLock"),
                    Some("Q"),
                    self.document.tool_locked,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.tool_locked = !this.document.tool_locked;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-snap-objects",
                    None,
                    &t("menu.snapToObjects"),
                    Some("Alt+S"),
                    self.document.snap_to_objects,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.snap_to_objects = !this.document.snap_to_objects;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-grid",
                    None,
                    &t("menu.grid"),
                    Some("Ctrl+'"),
                    self.document.show_grid,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.on_toggle_grid();
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-zen",
                    None,
                    &t("menu.zenMode"),
                    Some("Alt+Z"),
                    self.document.zen_mode,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.zen_mode = !this.document.zen_mode;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-view",
                    None,
                    &t("menu.viewMode"),
                    Some("Alt+R"),
                    self.document.view_mode,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.view_mode = !this.document.view_mode;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-stats",
                    None,
                    &t("menu.canvasStats"),
                    Some("Alt+/"),
                    self.document.show_stats,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.show_stats = !this.document.show_stats;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-arrow-binding",
                    None,
                    &t("menu.arrowBinding"),
                    None,
                    self.document.arrow_binding,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.arrow_binding = !this.document.arrow_binding;
                })),
            )
            .child(
                chrome::menu_row_with(
                    "prefs-snap-midpoints",
                    None,
                    &t("menu.snapToMidpoints"),
                    None,
                    self.document.snap_to_midpoints,
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.document.snap_to_midpoints = !this.document.snap_to_midpoints;
                })),
            )
            .into_any_element()
    }

    /// Pointer capture for a live gesture.
    ///
    /// The canvas is the whole window, but the islands over it are `occlude`d,
    /// and at least one of them can appear *under the cursor mid-gesture*: the
    /// properties panel springs up the moment the mouse-down selects something,
    /// and a panel control swallows the following mouse-up. That strands the
    /// gesture — `is_interacting` stays true, so the canvas keeps moving the
    /// element on hover and the shape switcher never appears. While a gesture is
    /// live this transparent sheet is painted last, so it is the first listener
    /// in the bubble phase and the release always lands here instead.
    fn render_gesture_capture(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        div()
            .id("gesture-capture")
            .test_support()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .occlude()
            .on_mouse_move(Self::act(cx, |this, e: &MouseMoveEvent, _, _| {
                this.on_canvas_mouse_move(
                    e.position.x.to_f64() as f32,
                    e.position.y.to_f64() as f32,
                    e.modifiers,
                );
            }))
            .on_mouse_up(
                MouseButton::Left,
                Self::act(cx, |this, e: &MouseUpEvent, _, cx| {
                    this.on_canvas_mouse_up(
                        e.position.x.to_f64() as f32,
                        e.position.y.to_f64() as f32,
                        e.modifiers,
                    );
                    cx.stop_propagation();
                }),
            )
            .into_any_element()
    }

    /// The shape-switch popup (`.ConvertElementTypePopup`).
    ///
    /// Selecting a single shape puts a small island of type buttons just under
    /// its bottom-left corner, so a rectangle can become a diamond without
    /// going back to the toolbar. Upstream hides it while anything else is
    /// live — drawing, resizing, a marquee, a menu — which is what keeps it
    /// from flickering through a drag.
    fn render_shape_switch(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        if self.is_interacting() || self.context_menu.is_some() || self.show_menu {
            return None;
        }
        let element = self.document.lone_selected_element()?;
        let targets = element.convertible_targets();
        if targets.is_empty() {
            return None;
        }
        let current = element.convert_target();
        let (left, top) = self.shape_switch_anchor()?;
        let tokens = Tokens::for_palette(palette_of(self.document.theme));
        let mut panel = div()
            .id("shape-switch")
            .test_support()
            .absolute()
            .occlude()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .justify_center()
            .gap(px(size::SHAPE_SWITCH_GAP))
            .p(px(size::SHAPE_SWITCH_PADDING))
            .rounded(px(size::RADIUS_LG))
            .bg(tokens.island())
            .shadow(crate::design::island_shadow())
            .left(px(left))
            .top(px(top));
        for target in targets {
            panel = panel.child(Self::shape_switch_button(
                *target,
                current == Some(*target),
                tokens,
                cx,
            ));
        }
        Some(panel.into_any_element())
    }

    /// Where the shape-switch popup hangs, in window coordinates.
    fn shape_switch_anchor(&self) -> Option<(f32, f32)> {
        let bounds = selection_bounds(&self.document.scene, &self.document.selected)?;
        let zoom = self.document.zoom;
        let x = self.document.scroll.x + bounds.min_x * zoom - size::SHAPE_SWITCH_OFFSET_X as f64;
        let y = self.document.scroll.y
            + bounds.max_y * zoom
            + size::SHAPE_SWITCH_OFFSET_Y as f64 * zoom;
        Some((x as f32, y as f32))
    }

    fn shape_switch_button(
        target: ConvertTarget,
        checked: bool,
        tokens: Tokens,
        cx: &Context<Self>,
    ) -> chrome::Button {
        let fg = if checked {
            tokens.selected_fg()
        } else {
            tokens.surface_text()
        };
        let base = div()
            .id(SharedString::from(format!(
                "shape-{}",
                convert_target_slug(target)
            )))
            .test_support()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .size(px(size::BUTTON))
            .rounded(px(size::RADIUS_MD))
            .text_color(fg)
            .child(crate::icons::icon(
                convert_target_icon(target),
                size::ICON,
                fg,
            ));
        let base = if checked {
            base.bg(tokens.selected_bg())
        } else {
            base.hover(move |s| s.bg(tokens.hover()))
        };
        base.on_click(Self::act(cx, move |this, _, _, _| {
            this.on_convert_target(target);
        }))
    }

    /// The unlock bubble (`.UnlockPopup`).
    ///
    /// A click that lands on a locked element — and nothing else — has no
    /// selection to give, so instead of doing nothing Excalidraw sits a padlock
    /// just above the element's top-left corner and lets one click both unlock
    /// and select it.
    fn render_unlock_popup(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        if self.is_interacting()
            || self.context_menu.is_some()
            || self.show_menu
            || self.document.view_mode
        {
            return None;
        }
        let id = self.active_locked_id.clone()?;
        let elements = self.document.locked_target(&id);
        let bounds = excalidraw_core::bounds::common_bounds(&elements)?;
        let tokens = Tokens::for_palette(palette_of(self.document.theme));
        let zoom = self.document.zoom;
        let left = self.document.scroll.x + bounds.min_x * zoom;
        // Upstream anchors by the *bottom* edge (`height + 12 - viewY`), which is
        // what keeps the bubble clear of the element however tall it is.
        let bottom = self.viewport.1 + size::UNLOCK_OFFSET_Y as f64
            - (self.document.scroll.y + bounds.min_y * zoom);
        Some(
            div()
                .id("unlock-popup")
                .test_support()
                .absolute()
                .occlude()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(size::UNLOCK_GAP))
                .p(px(size::UNLOCK_PADDING))
                .rounded(px(size::RADIUS_LG))
                .bg(tokens.island())
                .shadow(crate::design::island_shadow())
                .text_color(tokens.gray_60())
                .left(px(left as f32))
                .bottom(px(bottom as f32))
                .child(crate::icons::icon(
                    "LockedIconFilled",
                    size::UNLOCK_ICON,
                    tokens.gray_60(),
                ))
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.on_unlock_locked_target();
                }))
                .into_any_element(),
        )
    }

    /// The island of buttons floating off a selected element's top-right corner
    /// (`.excalidraw-canvas-buttons`).
    ///
    /// Upstream only ever builds this for the two elements whose existence
    /// depends on a server round-trip: a magic frame (convert the sketch to
    /// code) and a generated iframe (copy its source, go fullscreen). Both
    /// conditions are reproduced verbatim here, so the island shows exactly when
    /// the real editor shows it.
    fn render_canvas_buttons(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        if self.is_interacting()
            || self.context_menu.is_some()
            || self.show_menu
            || self.document.view_mode
        {
            return None;
        }
        let element = self.document.lone_selected_element()?;
        let buttons: Vec<(&str, CanvasButton)> = match element {
            Element::MagicFrame(_) => vec![("MagicIcon", CanvasButton::ConvertToCode)],
            Element::Iframe(iframe) if iframe_generation_finished(&iframe.base) => vec![
                ("copyIcon", CanvasButton::CopySource),
                ("fullscreenIcon", CanvasButton::Fullscreen),
            ],
            _ => return None,
        };
        let bounds = selection_bounds(&self.document.scene, &self.document.selected)?;
        let tokens = Tokens::for_palette(palette_of(self.document.theme));
        let zoom = self.document.zoom;
        let left =
            self.document.scroll.x + bounds.max_x * zoom + size::CANVAS_BUTTONS_OFFSET_X as f64;
        let top = self.document.scroll.y + bounds.min_y * zoom;
        let mut island = div()
            .id("canvas-buttons")
            .test_support()
            .absolute()
            .occlude()
            .flex()
            .flex_col()
            .gap(px(size::CANVAS_BUTTONS_GAP))
            .p(px(size::CANVAS_BUTTONS_PADDING))
            .rounded(px(size::RADIUS_LG))
            .bg(tokens.island())
            .shadow(crate::design::canvas_buttons_shadow())
            .left(px(left as f32))
            .top(px(top as f32));
        for (icon_name, action) in buttons {
            island = island.child(Self::canvas_button(icon_name, action, tokens, cx));
        }
        Some(island.into_any_element())
    }

    /// One 32px toggle inside the canvas-buttons island.
    fn canvas_button(
        icon_name: &str,
        action: CanvasButton,
        tokens: Tokens,
        cx: &mut Context<Self>,
    ) -> chrome::Button {
        let fg = tokens.surface_text();
        div()
            .id(SharedString::from(format!("canvas-{icon_name}")))
            .test_support()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .size(px(size::BUTTON))
            .rounded(px(size::RADIUS_MD))
            .text_color(fg)
            .child(crate::icons::icon(icon_name, size::ICON, fg))
            .hover(move |s| s.bg(tokens.hover()))
            .on_click(Self::act(cx, move |this, _, _, cx| {
                this.on_canvas_button(action, cx);
            }))
    }

    fn on_canvas_button(&mut self, action: CanvasButton, cx: &mut Context<Self>) {
        match action {
            // Copying the source is a real clipboard write; the other two need
            // machinery this build does not ship (an AI backend, a hosted
            // embeddable), so they report themselves rather than pretend.
            CanvasButton::CopySource => {
                let source = self
                    .document
                    .lone_selected_element()
                    .and_then(|element| element.base().link.clone())
                    .unwrap_or_default();
                cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(source));
                self.status = Some(self.i18n.t("labels.copySource"));
            }
            CanvasButton::ConvertToCode | CanvasButton::Fullscreen => {
                self.status = Some(self.i18n.t("status.unavailable"));
            }
        }
    }

    // ----- transient overlays ------------------------------------------------

    /// Bump the counter the transient overlays' auto-hide timers race on.
    fn next_overlay_nonce(&mut self) -> u64 {
        self.overlay_nonce += 1;
        self.overlay_nonce
    }

    /// Run `f` once, `after_ms` from now.
    ///
    /// React tears an overlay's timer down with its effect; here the callback
    /// re-checks the overlay's nonce first, so a hint that was replaced by a
    /// newer one is not cleared by the older one's stale timer.
    fn arm_overlay_hide<F>(&self, nonce: u64, after_ms: u64, f: F, cx: &mut Context<Self>)
    where
        F: Fn(&mut Self, u64) + 'static,
    {
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(after_ms))
                .await;
            let _ = this.update(cx, |editor, cx| {
                f(editor, nonce);
                cx.notify();
            });
        })
        .detach();
    }

    /// The icon shown beside the pointer when a tool shortcut fires.
    pub fn cursor_hint_icon(&self) -> Option<&'static str> {
        self.cursor_hint.as_ref().map(|hint| hint.icon)
    }

    /// The toast's current message, if a toast is up.
    pub fn toast_message(&self) -> Option<&str> {
        self.toast.as_ref().map(|toast| toast.message.as_str())
    }

    /// Which checkout a live file drag is heading for: `"scene"`, `"library"`,
    /// `"unknown"`, or `None` when no drag is over the window.
    pub fn file_drag_kind(&self) -> Option<&'static str> {
        self.file_drag.map(|drag| match drag.kind {
            DropKind::Scene => "scene",
            DropKind::Library => "library",
            DropKind::Unknown => "unknown",
        })
    }

    /// Whether the eye-dropper gesture is in flight.
    pub fn is_eye_dropper_active(&self) -> bool {
        self.eye_dropper.is_some()
    }

    /// Which colour the live pick will land on, so the panel can light up the
    /// trigger that started it.
    pub(crate) fn eye_dropper_target(&self) -> Option<ColorTarget> {
        self.eye_dropper.as_ref().map(|state| state.target)
    }

    /// The colour the eye-dropper currently has under the pointer.
    pub fn eye_dropper_color(&self) -> Option<&str> {
        self.eye_dropper.as_ref().map(|state| state.color.as_str())
    }

    /// Whether a path names an exported scene — `.excalidraw`, or an export such
    /// as `foo.excalidraw.png` that still carries the marker in its name.
    fn is_scene_file(path: &std::path::Path) -> bool {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        name.ends_with(".excalidraw") || name.contains(".excalidraw.")
    }

    /// The checkout an OS file drag is heading for, from the dragged paths.
    ///
    /// The web build cannot read a filename before the drop, so everything it
    /// cannot type is `Unknown`; a native window *can* read the path, and the
    /// extension is a sharper signal than the browser ever gets.
    fn drop_kind(paths: &[std::path::PathBuf]) -> DropKind {
        let mut kind = DropKind::Unknown;
        for path in paths {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if name.ends_with(".excalidrawlib") {
                return DropKind::Library;
            }
            if Self::is_scene_file(path) {
                kind = DropKind::Scene;
            }
        }
        kind
    }

    /// A drag entered or moved over the window (upstream `dragenter` /
    /// `dragover` on the Excalidraw container).
    fn on_external_drag(&mut self, paths: &[std::path::PathBuf], shift: bool) {
        self.file_drag = Some(FileDragState {
            kind: Self::drop_kind(paths),
            shift,
        });
    }

    /// The drag left the window, or its drop landed (upstream `dragleave` /
    /// `drop`).
    fn on_external_drag_exit(&mut self) {
        self.file_drag = None;
    }

    /// Show the transient hint beside the pointer (upstream `cursorHints.show`).
    ///
    /// The hint is not pinned where it appeared: upstream re-reads
    /// `viewport.lastPosition` on every pointer move, so it trails the cursor
    /// until it fades.
    fn show_cursor_hint(&mut self, icon: &'static str, cx: &mut Context<Self>) {
        // `viewport.lastPosition` stays at (0, 0) until the first pointer move,
        // so a keyboard-only session has nowhere to hang the hint.
        if self.cursor_screen == (0.0, 0.0) {
            return;
        }
        let nonce = self.next_overlay_nonce();
        self.cursor_hint = Some(CursorHintState { icon, nonce });
        self.arm_overlay_hide(
            nonce,
            CURSOR_HINT_DURATION_MS + CURSOR_HINT_FADE_MS,
            |editor, nonce| {
                if editor
                    .cursor_hint
                    .as_ref()
                    .is_some_and(|hint| hint.nonce == nonce)
                {
                    editor.cursor_hint = None;
                }
            },
            cx,
        );
    }

    /// Upstream `CursorHints.onToolShortcut` for the arrow and line tools.
    ///
    /// A digit always hints — it is often pressed blind. A letter only hints
    /// once the last one has gone stale, so re-picking a tool you just used does
    /// not nag.
    pub fn note_tool_shortcut(&mut self, key: &str, modifiers: Modifiers, cx: &mut Context<Self>) {
        if modifiers.control || modifiers.platform || modifiers.alt || modifiers.shift {
            return;
        }
        // A modal overlay owns the keyboard; the letter belongs to its filter.
        if self.show_palette || self.show_find || self.show_help || self.editing_text.is_some() {
            return;
        }
        let (icon, is_digit) = match key {
            "a" => (ARROW_TOOL_ICON, false),
            "5" => (ARROW_TOOL_ICON, true),
            "l" => ("LineIcon", false),
            "6" => ("LineIcon", true),
            _ => return,
        };
        if !is_digit {
            let fresh = self.cursor_hint_shown_at.is_some_and(|shown| {
                shown.elapsed() < std::time::Duration::from_millis(CURSOR_HINT_COOLDOWN_MS)
            });
            if fresh {
                return;
            }
        }
        self.cursor_hint_shown_at = Some(std::time::Instant::now());
        self.show_cursor_hint(icon, cx);
    }

    /// Show a status toast, replacing any current one (upstream `Toast`).
    ///
    /// `u64::MAX` stands in for upstream's `Infinity`: no auto-close.
    fn show_toast(
        &mut self,
        message: String,
        closable: bool,
        duration_ms: u64,
        cx: &mut Context<Self>,
    ) {
        let nonce = self.next_overlay_nonce();
        self.toast = Some(ToastState {
            message,
            closable,
            nonce,
        });
        if duration_ms != u64::MAX {
            self.arm_overlay_hide(
                nonce,
                duration_ms,
                |editor, nonce| {
                    if editor
                        .toast
                        .as_ref()
                        .is_some_and(|toast| toast.nonce == nonce)
                    {
                        editor.toast = None;
                    }
                },
                cx,
            );
        }
    }

    /// Dismiss the toast early (the close button).
    pub fn dismiss_toast(&mut self) {
        self.toast = None;
    }

    /// Start picking a colour for `target`, sampling wherever the pointer is
    /// (upstream `activeEyeDropperAtom`).
    fn start_eye_dropper(&mut self, target: ColorTarget) {
        let at = self.cursor_screen;
        let color = self.sample_color_at(at);
        self.eye_dropper = Some(EyeDropperState { target, at, color });
    }

    /// Toggle the eye-dropper for one of the panel's trigger buttons.
    pub(crate) fn toggle_eye_dropper(&mut self, target: ColorTarget) {
        match &self.eye_dropper {
            Some(state) if state.target == target => self.eye_dropper = None,
            _ => self.start_eye_dropper(target),
        }
    }

    /// Track the pointer while picking (upstream's `pointermove` listener).
    fn update_eye_dropper(&mut self, at: (f64, f64)) {
        if self.eye_dropper.is_none() {
            return;
        }
        let color = self.sample_color_at(at);
        if let Some(state) = self.eye_dropper.as_mut() {
            state.at = at;
            state.color = color;
        }
    }

    /// Apply the picked colour and end the gesture (upstream `pointerup`).
    fn commit_eye_dropper(&mut self) {
        let Some(state) = self.eye_dropper.take() else {
            return;
        };
        match state.target {
            ColorTarget::Stroke => self.document.set_selected_stroke_color(&state.color),
            ColorTarget::Background => self.document.set_selected_background_color(&state.color),
            ColorTarget::CanvasBackground => self.document.view_background_color = state.color,
        }
    }

    /// Abandon the gesture — Escape, or a click outside the picker.
    fn cancel_eye_dropper(&mut self) {
        self.eye_dropper = None;
    }

    /// The colour under a window-space point, as `#rrggbb`.
    ///
    /// The web build reads the real canvas pixel. This build paints the canvas
    /// by hand with no readable framebuffer, so it asks the document instead:
    /// the topmost element's background when it has a real one, else its stroke,
    /// else the canvas background — which is the colour the pixel would be for
    /// everything this build can paint.
    fn sample_color_at(&self, at: (f64, f64)) -> String {
        let world = self.screen_point(at.0 as f32, at.1 as f32);
        let threshold = 10.0 / self.document.zoom;
        let canvas = self.export_background().to_ascii_lowercase();
        let sampled = self
            .document
            .elements_at_including_locked(world, threshold)
            .last()
            .and_then(|element| {
                let base = element.base();
                let background = base.background_color.to_ascii_lowercase();
                if background != "transparent" && background.starts_with('#') {
                    return Some(background);
                }
                let stroke = base.stroke_color.to_ascii_lowercase();
                (stroke != "transparent" && stroke.starts_with('#')).then_some(stroke)
            });
        sampled.unwrap_or(canvas)
    }

    /// Apply a completed OS file drop (upstream `App`'s `drop` handler).
    ///
    /// A scene file dropped without shift replaces the canvas — exactly what the
    /// overlay promised — and reports itself through a toast; everything else is
    /// treated as an image and appended at the viewport centre. Returns whether
    /// the canvas was replaced.
    pub fn handle_external_drop(&mut self, paths: &ExternalPaths) -> bool {
        let list = paths.paths();
        let drag = self.file_drag.take();
        let kind = drag
            .map(|state| state.kind)
            .unwrap_or_else(|| Self::drop_kind(list));
        let keeps = drag
            .map(|state| state.keeps_content())
            .unwrap_or(kind == DropKind::Library);
        if kind == DropKind::Scene && !keeps {
            if let Some(path) = list.iter().find(|path| Self::is_scene_file(path)) {
                let path = path.to_string_lossy().to_string();
                if self.load_from(&path).is_ok() {
                    return true;
                }
            }
        }
        self.insert_image_paths(list);
        false
    }

    /// The OS file-drag overlay: a dimmed backdrop with a card naming the
    /// checkout (upstream `FileDropOverlay`).
    fn render_file_drop_overlay(&self, tokens: Tokens) -> gpui_kit::AnyElement {
        let Some(state) = self.file_drag else {
            return div().into_any_element();
        };
        let t = |k: &str| self.i18n.t(k);
        let keeps = state.keeps_content();
        let title = match state.kind {
            DropKind::Library => t("fileDrop.importLibrary"),
            _ if state.shift => t("fileDrop.add"),
            _ => t("fileDrop.replace"),
        };
        let hint: Vec<AnyElement> = if keeps {
            vec![plain_text(&t("fileDrop.keepHint"))]
        } else {
            split_hint(&t("fileDrop.replaceHint"))
                .into_iter()
                .map(|(run, is_key)| {
                    if is_key {
                        div()
                            .flex_shrink_0()
                            .mx(px(2.0))
                            .px(px(size::file_drop::KBD_PAD_X))
                            .py(px(size::file_drop::KBD_PAD_Y))
                            .rounded(px(size::file_drop::KBD_RADIUS))
                            .border_1()
                            .border_color(tokens.surface_high())
                            .bg(tokens.surface_low())
                            .child(SharedString::from(run))
                            .into_any_element()
                    } else {
                        plain_text(&run)
                    }
                })
                .collect()
        };
        let mut card = div()
            .w(px(size::file_drop::CARD_WIDTH))
            .flex()
            .flex_col()
            .items_center()
            .pt(px(size::file_drop::CARD_PAD_TOP))
            .px(px(size::file_drop::CARD_PAD_X))
            .pb(px(size::file_drop::CARD_PAD_BOTTOM))
            .rounded(px(size::file_drop::CARD_RADIUS))
            .border_1()
            .border_color(tokens.primary_light_darker())
            .bg(tokens.island())
            .shadow(crate::design::file_drop_card_shadow())
            .child(
                div()
                    .text_size(px(size::file_drop::TITLE_FONT))
                    .font_bold()
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(title)),
            )
            .child(
                div()
                    .mt(px(size::file_drop::HINT_GAP))
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .justify_center()
                    .text_size(px(size::file_drop::HINT_FONT))
                    .text_color(tokens.surface_text())
                    .opacity(0.7)
                    .children(hint),
            );
        if state.kind == DropKind::Unknown {
            card = card.child(
                div()
                    .text_size(px(size::file_drop::SECONDARY_FONT))
                    .text_color(tokens.surface_text())
                    .opacity(0.45)
                    .child(SharedString::from(format!(
                        "({})",
                        t("fileDrop.libraryHint")
                    ))),
            );
        }
        card = card.child(self.file_drop_illustration(tokens, keeps));
        div()
            .id("file-drop-overlay")
            .test_support()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(size::file_drop::OVERLAY_PADDING))
            .bg(tokens.overlay_backdrop())
            .child(card)
            .into_any_element()
    }

    /// The two floating sheets illustrating a scene drop: a front sheet, and a
    /// back one that is either crossed out (replace) or plain (keep).
    ///
    /// Upstream builds this from one inline `<svg>` per sheet and switches the
    /// three states in CSS: the back sheet's ruled lines hide when replacing,
    /// its cross hides when keeping, and its colour lifts to `--color-primary`
    /// when keeping. The markup is therefore assembled here rather than shipped
    /// as an asset — the paper is filled with `--island-bg-color`, and an asset
    /// cannot carry a colour that changes with the theme.
    fn file_drop_illustration(&self, tokens: Tokens, keeps: bool) -> Div {
        let island = tokens.island_bg;
        let sheet = |x: f32, angle: f32, top: f32, stroke: u32, marks: &str| {
            let markup = format!(
                concat!(
                    r##"<svg viewBox="0 0 100 128" fill="none" stroke="#{stroke:06x}" "##,
                    r##"stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">"##,
                    r##"<path fill="#{fill:06x}" d="M16 3h43l28 28v81c0 7-4 12-12 12H16c-8 0-12-5-12-12V15C4 7 8 3 16 3Z"/>"##,
                    r##"<path d="M59 3v20c0 6 3 8 9 8h19"/>{marks}</svg>"##,
                ),
                stroke = stroke,
                fill = island,
                marks = marks,
            );
            svg()
                .data(markup.as_bytes())
                .absolute()
                .top(px(top))
                .left(px(0.0))
                .w(px(size::file_drop::SHEET_W))
                .h(px(size::file_drop::SHEET_H))
                // The rasteriser resolves `currentColor` from the style's text
                // colour and skips the element entirely without one, so this is
                // required even though the markup names every colour itself.
                .text_color(rgb(stroke))
                .with_transformation(
                    Transformation::rotate(radians(angle)).with_translation(point(px(x), px(0.0))),
                )
        };
        let (back_stroke, back_marks) = if keeps {
            (
                tokens.primary,
                r#"<path opacity="0.35" d="M24 61h43M24 77h43M24 93h26"/>"#,
            )
        } else {
            (
                tokens.primary_light_darker,
                r#"<path stroke-width="4" d="M18 44l24 24M42 44L18 68"/>"#,
            )
        };
        div()
            .relative()
            .w(px(size::file_drop::SHEET_W))
            .h(px(size::file_drop::ILLUSTRATION_HEIGHT))
            .mt(px(size::file_drop::ILLUSTRATION_GAP))
            .child(sheet(
                -24.0,
                (-18.0_f32).to_radians(),
                size::file_drop::SHEET_TOP_BACK,
                back_stroke,
                back_marks,
            ))
            .child(sheet(
                24.0,
                (12.0_f32).to_radians(),
                size::file_drop::SHEET_TOP,
                tokens.primary,
                r#"<path opacity="0.35" d="M24 61h43M24 77h43M24 93h26"/>"#,
            ))
    }

    /// The transient hint that follows the pointer (upstream `CursorHint`).
    fn render_cursor_hint(&self, tokens: Tokens) -> Option<gpui_kit::AnyElement> {
        let hint = self.cursor_hint.as_ref()?;
        let box_size = size::cursor_hint::ICON + size::cursor_hint::PADDING * 2.0;
        let (left, top) = position_beside_cursor(
            self.cursor_screen,
            (box_size as f64, box_size as f64),
            self.viewport,
            size::cursor_hint::GAP as f64,
        );
        Some(
            div()
                .id("cursor-hint")
                .test_support()
                .absolute()
                .left(px(left as f32))
                .top(px(top as f32))
                .flex()
                .items_center()
                .justify_center()
                .p(px(size::cursor_hint::PADDING))
                .rounded(px(size::RADIUS_MD))
                .bg(tokens.cursor_hint_bg())
                .shadow(crate::design::island_shadow())
                .child(crate::icons::icon(
                    hint.icon,
                    size::cursor_hint::ICON,
                    tokens.cursor_hint_fg(),
                ))
                .into_any_element(),
        )
    }

    /// The bottom-centre status toast (upstream `.floating-status-stack`).
    fn render_toast(&self, tokens: Tokens, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        let toast = self.toast.as_ref()?;
        let mut card = div()
            .id("toast")
            .test_support()
            .relative()
            .min_w(px(size::toast::MIN_WIDTH))
            .max_w(px(size::toast::MAX_WIDTH))
            .px(px(size::toast::PAD_X))
            .py(px(size::toast::PAD_Y))
            .rounded(px(size::RADIUS_LG))
            .border_1()
            .border_color(tokens.surface_high())
            .bg(tokens.island())
            .shadow(crate::design::chip_ring_shadow(tokens))
            .text_size(px(size::toast::FONT))
            .text_color(tokens.surface_text())
            .child(
                // Upstream reserves the close button's width on both sides even
                // when the toast is not closable, so the text stays centred.
                div()
                    .px(px(size::toast::CLOSE_ICON + size::toast::CLOSE_PAD))
                    .text_center()
                    .child(SharedString::from(toast.message.clone())),
            );
        if toast.closable {
            card = card.child(
                div()
                    .id("toast-close")
                    .test_support()
                    .absolute()
                    .top(px(0.0))
                    .right(px(0.0))
                    .p(px(size::toast::CLOSE_PAD))
                    .text_color(tokens.surface_text())
                    .child(crate::icons::icon(
                        "CloseIcon",
                        size::toast::CLOSE_ICON,
                        tokens.surface_text(),
                    ))
                    .on_click(Self::act(cx, |this, _, _, _| this.dismiss_toast())),
            );
        }
        Some(
            div()
                .absolute()
                .left(px(0.0))
                .right(px(0.0))
                .bottom(px(size::toast::BOTTOM))
                .flex()
                .flex_col()
                .items_center()
                .child(card)
                .into_any_element(),
        )
    }

    /// The eye-dropper: a full-window backdrop that owns the pointer plus the
    /// colour preview hanging beside it (upstream `EyeDropper`).
    fn render_eye_dropper(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        let state = self.eye_dropper.as_ref()?;
        let (left, top) = position_beside_cursor(
            state.at,
            (
                size::eye_dropper::PREVIEW as f64,
                size::eye_dropper::PREVIEW as f64,
            ),
            self.viewport,
            size::eye_dropper::PREVIEW_GAP as f64,
        );
        let ring = if is_dark_hex(&state.color) {
            rgb(0xffffff)
        } else {
            rgb(0x222222)
        };
        let preview = div()
            .absolute()
            .left(px(left as f32))
            .top(px(top as f32))
            .size(px(size::eye_dropper::PREVIEW))
            .rounded_full()
            .bg(crate::properties::parse_hex(&state.color))
            .border_1()
            .border_color(ring)
            .shadow(crate::design::eye_dropper_shadow());
        Some(
            div()
                .id("eye-dropper")
                .test_support()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .size_full()
                // The picker owns the pointer while it is up: a click anywhere
                // commits the sample instead of reaching the canvas or the panel
                // underneath.
                .occlude()
                .on_mouse_move(Self::act(cx, |this, e: &MouseMoveEvent, _, _| {
                    this.update_eye_dropper((e.position.x.to_f64(), e.position.y.to_f64()));
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    Self::act(cx, |this, e: &MouseDownEvent, _, _| {
                        this.update_eye_dropper((e.position.x.to_f64(), e.position.y.to_f64()));
                        this.commit_eye_dropper();
                    }),
                )
                .child(preview)
                .into_any_element(),
        )
    }

    /// The right-click menu, positioned at the cursor.
    fn render_context_menu(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let pos = self.context_menu.unwrap_or(Point::zero());
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let rule = || {
            div()
                .h(px(1.0))
                .my(px(4.0))
                .mx(px(4.0))
                .bg(tokens.separator())
        };

        chrome::dropdown(tokens)
            .left(px(pos.x as f32))
            .top(px(pos.y as f32))
            .child(
                chrome::menu_row("cm-copy", Some("copyIcon"), &t("ctx.copy"), tokens).on_click(
                    Self::act(cx, |this, _, _, _| {
                        this.on_copy();
                        this.context_menu = None;
                    }),
                ),
            )
            .child(
                chrome::menu_row("cm-paste", Some("clipboard"), &t("ctx.paste"), tokens).on_click(
                    Self::act(cx, |this, _, _, _| {
                        this.on_paste();
                        this.context_menu = None;
                    }),
                ),
            )
            .child(
                chrome::menu_row("cm-dup", Some("DuplicateIcon"), &t("ctx.duplicate"), tokens)
                    .on_click(Self::act(cx, |this, _, _, _| {
                        this.on_duplicate();
                        this.context_menu = None;
                    })),
            )
            .child(rule())
            .child(
                chrome::menu_row(
                    "cm-front",
                    Some("BringToFrontIcon"),
                    &t("ctx.bringFront"),
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.push_checkpoint();
                    this.document.bring_to_front();
                    this.context_menu = None;
                })),
            )
            .child(
                chrome::menu_row(
                    "cm-back",
                    Some("SendToBackIcon"),
                    &t("ctx.sendBack"),
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.push_checkpoint();
                    this.document.send_to_back();
                    this.context_menu = None;
                })),
            )
            .child(rule())
            .child(
                chrome::menu_row("cm-group", Some("GroupIcon"), &t("action.group"), tokens)
                    .on_click(Self::act(cx, |this, _, _, _| {
                        this.on_group();
                        this.context_menu = None;
                    })),
            )
            .child(
                chrome::menu_row(
                    "cm-ungroup",
                    Some("UngroupIcon"),
                    &t("action.ungroup"),
                    tokens,
                )
                .on_click(Self::act(cx, |this, _, _, _| {
                    this.on_ungroup();
                    this.context_menu = None;
                })),
            )
            .child(
                chrome::menu_row("cm-del", Some("TrashIcon"), &t("ctx.delete"), tokens).on_click(
                    Self::act(cx, |this, _, _, _| {
                        this.on_delete();
                        this.context_menu = None;
                    }),
                ),
            )
            .into_any_element()
    }
}

/// One entry of the command palette.
struct PaletteCommand {
    /// Stable id used by tests and by the click handler.
    id: &'static str,
    /// i18n key for the label; the palette filters on the *translated* text.
    label: &'static str,
    shortcut: Option<&'static str>,
    icon: &'static str,
    action: PaletteAction,
}

#[derive(Clone, Copy)]
enum PaletteAction {
    Tool(Tool),
    Undo,
    Redo,
    Duplicate,
    Delete,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Group,
    Ungroup,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    ZoomSelection,
    ToggleTheme,
    ToggleGrid,
    ToggleZen,
    ToggleView,
    ToggleStats,
    ToggleLock,
    Save,
    Open,
    ExportPng,
    NewCanvas,
    Find,
    Help,
}

impl PaletteAction {
    fn run(self, editor: &mut Editor) {
        let viewport = editor.viewport;
        match self {
            PaletteAction::Tool(tool) => editor.set_tool(tool),
            PaletteAction::Undo => editor.on_undo(),
            PaletteAction::Redo => editor.on_redo(),
            PaletteAction::Duplicate => editor.on_duplicate(),
            PaletteAction::Delete => editor.on_delete(),
            PaletteAction::SelectAll => editor.document.select_all(),
            PaletteAction::Copy => editor.on_copy(),
            PaletteAction::Cut => editor.on_cut(),
            PaletteAction::Paste => editor.on_paste(),
            PaletteAction::Group => editor.on_group(),
            PaletteAction::Ungroup => editor.on_ungroup(),
            PaletteAction::ZoomIn => editor.on_zoom_in(),
            PaletteAction::ZoomOut => editor.on_zoom_out(),
            PaletteAction::ZoomReset => editor.on_reset_zoom(),
            PaletteAction::ZoomFit => editor.document.zoom_to_fit(viewport, 40.0),
            PaletteAction::ZoomSelection => editor.document.zoom_to_selection(viewport, 40.0),
            PaletteAction::ToggleTheme => editor.on_toggle_theme(),
            PaletteAction::ToggleGrid => editor.on_toggle_grid(),
            PaletteAction::ToggleZen => editor.document.zen_mode = !editor.document.zen_mode,
            PaletteAction::ToggleView => editor.document.view_mode = !editor.document.view_mode,
            PaletteAction::ToggleStats => editor.document.show_stats = !editor.document.show_stats,
            PaletteAction::ToggleLock => editor.document.tool_locked = !editor.document.tool_locked,
            // Same as Ctrl+S: raise the dialog, which is where the save happens.
            PaletteAction::Save => editor.pending_save_dialog = true,
            // The palette runs from `handle_key` too, which has no `Context`
            // for a platform picker; `render` picks the request up.
            PaletteAction::Open => editor.pending_scene_open = true,
            PaletteAction::ExportPng => {
                let _ = editor.export_png_to("excalidraw-export.png");
            }
            PaletteAction::NewCanvas => editor.on_new_document(),
            PaletteAction::Find => editor.open_find(),
            PaletteAction::Help => editor.open_help(),
        }
    }
}

/// The commands the palette offers, in upstream's grouping: tools, then
/// actions, then view toggles, then file operations. Label keys are reused
/// from the toolbar / menu / context-menu tables so no string is duplicated.
const PALETTE_COMMANDS: &[PaletteCommand] = &[
    PaletteCommand {
        id: "tool-select",
        label: "toolbar.select",
        shortcut: Some("V"),
        icon: "SelectionIcon",
        action: PaletteAction::Tool(Tool::Selection),
    },
    PaletteCommand {
        id: "tool-hand",
        label: "toolbar.hand",
        shortcut: Some("H"),
        icon: "handIcon",
        action: PaletteAction::Tool(Tool::Hand),
    },
    PaletteCommand {
        id: "tool-rectangle",
        label: "toolbar.rectangle",
        shortcut: Some("R"),
        icon: "RectangleIcon",
        action: PaletteAction::Tool(Tool::Rectangle),
    },
    PaletteCommand {
        id: "tool-diamond",
        label: "toolbar.diamond",
        shortcut: Some("D"),
        icon: "DiamondIcon",
        action: PaletteAction::Tool(Tool::Diamond),
    },
    PaletteCommand {
        id: "tool-ellipse",
        label: "toolbar.ellipse",
        shortcut: Some("O"),
        icon: "EllipseIcon",
        action: PaletteAction::Tool(Tool::Ellipse),
    },
    PaletteCommand {
        id: "tool-arrow",
        label: "toolbar.arrow",
        shortcut: Some("A"),
        icon: "ArrowIcon",
        action: PaletteAction::Tool(Tool::Arrow),
    },
    PaletteCommand {
        id: "tool-line",
        label: "toolbar.line",
        shortcut: Some("L"),
        icon: "LineIcon",
        action: PaletteAction::Tool(Tool::Line),
    },
    PaletteCommand {
        id: "tool-freedraw",
        label: "toolbar.freedraw",
        shortcut: Some("P"),
        icon: "FreedrawIcon",
        action: PaletteAction::Tool(Tool::FreeDraw),
    },
    PaletteCommand {
        id: "tool-text",
        label: "toolbar.text",
        shortcut: Some("T"),
        icon: "TextIcon",
        action: PaletteAction::Tool(Tool::Text),
    },
    PaletteCommand {
        id: "tool-image",
        label: "toolbar.image",
        shortcut: Some("9"),
        icon: "ImageIcon",
        action: PaletteAction::Tool(Tool::Image),
    },
    PaletteCommand {
        id: "tool-frame",
        label: "toolbar.frame",
        shortcut: Some("F"),
        icon: "frameToolIcon",
        action: PaletteAction::Tool(Tool::Frame),
    },
    PaletteCommand {
        id: "tool-eraser",
        label: "toolbar.eraser",
        shortcut: Some("E"),
        icon: "EraserIcon",
        action: PaletteAction::Tool(Tool::Eraser),
    },
    PaletteCommand {
        id: "tool-laser",
        label: "toolbar.laser",
        shortcut: Some("K"),
        icon: "laserPointerToolIcon",
        action: PaletteAction::Tool(Tool::Laser),
    },
    PaletteCommand {
        id: "action-undo",
        label: "action.undo",
        shortcut: Some("Ctrl+Z"),
        icon: "UndoIcon",
        action: PaletteAction::Undo,
    },
    PaletteCommand {
        id: "action-redo",
        label: "action.redo",
        shortcut: Some("Ctrl+Shift+Z"),
        icon: "RedoIcon",
        action: PaletteAction::Redo,
    },
    PaletteCommand {
        id: "action-duplicate",
        label: "action.duplicate",
        shortcut: Some("Ctrl+D"),
        icon: "DuplicateIcon",
        action: PaletteAction::Duplicate,
    },
    PaletteCommand {
        id: "action-delete",
        label: "action.delete",
        shortcut: Some("Del"),
        icon: "TrashIcon",
        action: PaletteAction::Delete,
    },
    PaletteCommand {
        id: "action-select-all",
        label: "ctx.selectAll",
        shortcut: Some("Ctrl+A"),
        icon: "selectAllIcon",
        action: PaletteAction::SelectAll,
    },
    PaletteCommand {
        id: "action-copy",
        label: "action.copyToClipboard",
        shortcut: Some("Ctrl+C"),
        icon: "copyIcon",
        action: PaletteAction::Copy,
    },
    PaletteCommand {
        id: "action-cut",
        label: "ctx.cut",
        shortcut: Some("Ctrl+X"),
        icon: "cutIcon",
        action: PaletteAction::Cut,
    },
    PaletteCommand {
        id: "action-paste",
        label: "ctx.paste",
        shortcut: Some("Ctrl+V"),
        icon: "clipboard",
        action: PaletteAction::Paste,
    },
    PaletteCommand {
        id: "action-group",
        label: "action.group",
        shortcut: Some("Ctrl+G"),
        icon: "GroupIcon",
        action: PaletteAction::Group,
    },
    PaletteCommand {
        id: "action-ungroup",
        label: "action.ungroup",
        shortcut: Some("Ctrl+Shift+G"),
        icon: "UngroupIcon",
        action: PaletteAction::Ungroup,
    },
    PaletteCommand {
        id: "view-zoom-in",
        label: "action.zoomIn",
        shortcut: Some("Ctrl++"),
        icon: "ZoomInIcon",
        action: PaletteAction::ZoomIn,
    },
    PaletteCommand {
        id: "view-zoom-out",
        label: "action.zoomOut",
        shortcut: Some("Ctrl+-"),
        icon: "ZoomOutIcon",
        action: PaletteAction::ZoomOut,
    },
    PaletteCommand {
        id: "view-zoom-reset",
        label: "action.resetZoom",
        shortcut: Some("Ctrl+0"),
        icon: "resetZoom",
        action: PaletteAction::ZoomReset,
    },
    PaletteCommand {
        id: "view-zoom-fit",
        label: "action.zoomToFit",
        shortcut: Some("Shift+1"),
        icon: "zoomAreaIcon",
        action: PaletteAction::ZoomFit,
    },
    PaletteCommand {
        id: "view-zoom-selection",
        label: "menu.viewMode",
        shortcut: Some("Shift+2"),
        icon: "zoomAreaIcon",
        action: PaletteAction::ZoomSelection,
    },
    PaletteCommand {
        id: "view-theme",
        label: "action.toggleTheme",
        shortcut: None,
        icon: "themeDarkIcon",
        action: PaletteAction::ToggleTheme,
    },
    PaletteCommand {
        id: "view-grid",
        label: "action.toggleGrid",
        shortcut: Some("Ctrl+'"),
        icon: "gridIcon",
        action: PaletteAction::ToggleGrid,
    },
    PaletteCommand {
        id: "view-zen",
        label: "menu.zenMode",
        shortcut: Some("Alt+Z"),
        icon: "eyeIcon",
        action: PaletteAction::ToggleZen,
    },
    PaletteCommand {
        id: "view-readonly",
        label: "menu.viewMode",
        shortcut: Some("Alt+R"),
        icon: "eyeClosedIcon",
        action: PaletteAction::ToggleView,
    },
    PaletteCommand {
        id: "view-stats",
        label: "menu.canvasStats",
        shortcut: Some("Alt+/"),
        icon: "slidersIcon",
        action: PaletteAction::ToggleStats,
    },
    PaletteCommand {
        id: "view-tool-lock",
        label: "menu.toolLock",
        shortcut: Some("Q"),
        icon: "LockedIcon",
        action: PaletteAction::ToggleLock,
    },
    PaletteCommand {
        id: "file-save",
        label: "menu.save",
        shortcut: Some("Ctrl+S"),
        icon: "ExportIcon",
        action: PaletteAction::Save,
    },
    PaletteCommand {
        id: "file-open",
        label: "menu.open",
        shortcut: Some("Ctrl+O"),
        icon: "LoadIcon",
        action: PaletteAction::Open,
    },
    PaletteCommand {
        id: "file-export-png",
        label: "action.exportPng",
        shortcut: Some("Ctrl+Shift+E"),
        icon: "ExportImageIcon",
        action: PaletteAction::ExportPng,
    },
    PaletteCommand {
        id: "file-new",
        label: "menu.reset",
        shortcut: None,
        icon: "TrashIcon",
        action: PaletteAction::NewCanvas,
    },
    PaletteCommand {
        id: "overlay-find",
        label: "menu.findOnCanvas",
        shortcut: Some("Ctrl+F"),
        icon: "searchIcon",
        action: PaletteAction::Find,
    },
    PaletteCommand {
        id: "overlay-help",
        label: "menu.help",
        shortcut: Some("?"),
        icon: "HelpIcon",
        action: PaletteAction::Help,
    },
];

/// The shortcut rows the Help dialog lists, grouped as sections of
/// `(i18n label key, key names)`.
const HELP_SHORTCUTS: &[(&str, &[(&str, &str)])] = &[
    (
        "help.tools",
        &[
            ("toolbar.rectangle", "R"),
            ("toolbar.ellipse", "O"),
            ("toolbar.arrow", "A"),
            ("toolbar.line", "L"),
            ("toolbar.freedraw", "P"),
            ("toolbar.text", "T"),
            ("toolbar.eraser", "E"),
            ("toolbar.laser", "K"),
            ("menu.toolLock", "Q"),
        ],
    ),
    (
        "help.actions",
        &[
            ("action.undo", "Ctrl+Z"),
            ("action.redo", "Ctrl+Shift+Z"),
            ("action.duplicate", "Ctrl+D"),
            ("action.delete", "Delete"),
            ("ctx.selectAll", "Ctrl+A"),
            ("action.copyToClipboard", "Ctrl+C"),
            ("ctx.cut", "Ctrl+X"),
            ("ctx.paste", "Ctrl+V"),
            ("action.group", "Ctrl+G"),
            ("action.ungroup", "Ctrl+Shift+G"),
        ],
    ),
    (
        "help.view",
        &[
            ("action.zoomIn", "Ctrl++"),
            ("action.zoomOut", "Ctrl+-"),
            ("action.resetZoom", "Ctrl+0"),
            ("action.zoomToFit", "Shift+1"),
            ("action.toggleGrid", "Ctrl+'"),
            ("menu.zenMode", "Alt+Z"),
            ("menu.viewMode", "Alt+R"),
            ("menu.canvasStats", "Alt+/"),
        ],
    ),
    (
        "help.files",
        &[
            ("menu.open", "Ctrl+O"),
            ("menu.save", "Ctrl+S"),
            ("action.exportPng", "Ctrl+Shift+E"),
            ("menu.commandPalette", "Ctrl+/"),
            ("menu.findOnCanvas", "Ctrl+F"),
            ("menu.help", "?"),
        ],
    ),
];

/// The ‹ › stepper buttons the find bar uses to walk its matches. Drawn as
/// glyph text rather than an icon because the bundled set has no chevrons.
fn find_nav_button(
    id: &'static str,
    glyph: &'static str,
    forward: bool,
    tokens: crate::design::Tokens,
    cx: &mut Context<Editor>,
) -> impl IntoElement + use<> {
    div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size::BUTTON))
        .rounded(px(size::RADIUS_MD))
        .text_color(tokens.surface_text())
        .hover(move |s| s.bg(tokens.hover()))
        .child(SharedString::from(glyph.to_owned()))
        .on_click(Editor::act(cx, move |this, _, _, _| {
            this.step_find(forward)
        }))
}

impl Editor {
    fn render_palette(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let results = self.palette_results();
        let width = 560.0_f32;
        let left = ((self.viewport.0 as f32 - width) / 2.0).max(size::CONTAINER_PADDING);

        let query_color = if self.palette_query.is_empty() {
            tokens.shortcut_text()
        } else {
            tokens.surface_text()
        };
        let query_text = if self.palette_query.is_empty() {
            t("palette.placeholder")
        } else {
            self.palette_query.clone()
        };

        let mut panel = chrome::dropdown(tokens)
            .left(px(left))
            .top(px(80.0))
            .w(px(width))
            .min_w(px(width))
            .max_h(px(420.0))
            .child(
                div()
                    .id("palette-query")
                    .test_support()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(8.0))
                    .py(px(8.0))
                    .text_size(px(15.0))
                    .text_color(query_color)
                    .child(crate::icons::icon(
                        "searchIcon",
                        size::ICON,
                        tokens.shortcut_text(),
                    ))
                    .child(div().flex_1().child(SharedString::from(query_text)))
                    .child(chrome::kbd_chip("Esc", tokens)),
            );

        if results.is_empty() {
            panel = panel.child(
                div()
                    .px(px(8.0))
                    .py(px(12.0))
                    .text_size(px(14.0))
                    .text_color(tokens.shortcut_text())
                    .child(SharedString::from(t("palette.noResults"))),
            );
        } else {
            let mut list = div().flex().flex_col().gap(px(2.0)).overflow_hidden();
            for (i, (_id, label, shortcut, icon_name)) in results.iter().enumerate() {
                let selected = i == self.palette_index;
                let row = chrome::menu_row_with(
                    format!("palette-row-{i}"),
                    Some(icon_name),
                    &t(label),
                    *shortcut,
                    false,
                    tokens,
                );
                let row = if selected {
                    row.bg(tokens.selected_bg())
                        .text_color(tokens.selected_fg())
                } else {
                    row
                };
                list = list.child(row.on_click(Self::act(cx, move |this, _, _, _| {
                    this.palette_index = i;
                    this.run_palette_selection();
                })));
            }
            panel = panel.child(list);
        }

        // The backdrop is a *sibling* of the panel, not its parent: a click on
        // the panel must not bubble into a dismiss handler and close the very
        // overlay the user is interacting with.
        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .child(
                div()
                    .id("palette-backdrop")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .occlude()
                    .bg(hsla(0.0, 0.0, 0.0, 0.18))
                    .on_click(Self::act(cx, |this, _, _, _| this.show_palette = false)),
            )
            .child(panel)
            .into_any_element()
    }

    fn render_find(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let width = 340.0_f32;
        let left =
            (self.viewport.0 as f32 - width - size::CONTAINER_PADDING).max(size::CONTAINER_PADDING);
        let count = self.find_matches.len();
        let position = if count == 0 { 0 } else { self.find_index + 1 };
        let counter = format!("{position}/{count}");

        let query_color = if self.find_query.is_empty() {
            tokens.shortcut_text()
        } else {
            tokens.surface_text()
        };
        let query_text = if self.find_query.is_empty() {
            t("find.placeholder")
        } else {
            self.find_query.clone()
        };

        let body = if count == 0 && !self.find_query.is_empty() {
            div()
                .px(px(8.0))
                .py(px(6.0))
                .text_size(px(13.0))
                .text_color(tokens.shortcut_text())
                .child(SharedString::from(t("find.noResults")))
        } else {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .child(find_nav_button("find-prev", "\u{2039}", false, tokens, cx))
                .child(find_nav_button("find-next", "\u{203a}", true, tokens, cx))
                .child(
                    div()
                        .flex_1()
                        .text_center()
                        .text_size(px(13.0))
                        .text_color(tokens.surface_text())
                        .child(SharedString::from(counter)),
                )
                .child(
                    div()
                        .id("find-close")
                        .test_support()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(size::BUTTON))
                        .rounded(px(size::RADIUS_MD))
                        .hover(move |s| s.bg(tokens.hover()))
                        .child(crate::icons::icon(
                            "CloseIcon",
                            size::ICON,
                            tokens.surface_text(),
                        ))
                        .on_click(Self::act(cx, |this, _, _, _| this.show_find = false)),
                )
        };

        chrome::dropdown(tokens)
            .left(px(left))
            .top(px(80.0))
            .w(px(width))
            .min_w(px(width))
            .child(
                div()
                    .id("find-query")
                    .test_support()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(8.0))
                    .py(px(8.0))
                    .text_size(px(15.0))
                    .text_color(query_color)
                    .child(crate::icons::icon(
                        "searchIcon",
                        size::ICON,
                        tokens.shortcut_text(),
                    ))
                    .child(div().flex_1().child(SharedString::from(query_text))),
            )
            .child(body)
            .into_any_element()
    }

    fn render_help(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        let width = 520.0_f32;
        let left = ((self.viewport.0 as f32 - width) / 2.0).max(size::CONTAINER_PADDING);

        let mut panel = chrome::dropdown(tokens)
            .left(px(left))
            .top(px(64.0))
            .w(px(width))
            .min_w(px(width))
            .max_h(px(self.viewport.1 as f32 - 128.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(8.0))
                    .py(px(6.0))
                    .child(crate::icons::icon(
                        "HelpIcon",
                        size::ICON,
                        tokens.surface_text(),
                    ))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(16.0))
                            .text_color(tokens.surface_text())
                            .child(SharedString::from(t("menu.help"))),
                    )
                    .child(
                        div()
                            .id("help-close")
                            .test_support()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(size::BUTTON))
                            .rounded(px(size::RADIUS_MD))
                            .hover(move |s| s.bg(tokens.hover()))
                            .child(crate::icons::icon(
                                "CloseIcon",
                                size::ICON,
                                tokens.surface_text(),
                            ))
                            .on_click(Self::act(cx, |this, _, _, _| this.show_help = false)),
                    ),
            );

        let mut list = div().flex().flex_col().gap(px(2.0)).overflow_hidden();
        for &(section, rows) in HELP_SHORTCUTS {
            list = list.child(
                div()
                    .px(px(8.0))
                    .pt(px(8.0))
                    .pb(px(2.0))
                    .text_size(px(13.0))
                    .font_bold()
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(t(section))),
            );
            for &(label, keys) in rows {
                list = list.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.0))
                        .w_full()
                        .min_h(px(size::MENU_ROW))
                        .px(px(8.0))
                        .rounded(px(size::RADIUS_MD))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(14.0))
                                .text_color(tokens.surface_text())
                                .child(SharedString::from(t(label))),
                        )
                        .child(chrome::kbd_chip(keys, tokens)),
                );
            }
        }
        panel = panel.child(list);

        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .child(
                div()
                    .id("help-backdrop")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .occlude()
                    .bg(hsla(0.0, 0.0, 0.0, 0.18))
                    .on_click(Self::act(cx, |this, _, _, _| this.show_help = false)),
            )
            .child(panel)
            .into_any_element()
    }

    /// Upstream's `JSONExportDialog`, drawn as a modal.
    ///
    /// It renders one `Card` per export route, and all of them are conditional:
    /// the share-link card needs excalidraw.com's backend (`onExportToBackend`)
    /// and the Excalidraw+ card is injected by the hosted app
    /// (`exportOpts.renderCustomUI`). Neither exists in this build, so only the
    /// local-file card is drawn, exactly as a self-hosted upstream install
    /// without those options would.
    fn render_save_dialog(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let tokens = crate::design::Tokens::for_palette(palette_of(self.document.theme));
        let t = |k: &str| self.i18n.t(k);
        // The modal's geometry, aliased so each use reads as `d::PADDING`.
        use crate::design::size::save_dialog as d;

        // `.ProjectName`: a fit-content column, centred in the card, whose own
        // contents are left-aligned under the card's centred prose.
        let project_name = div()
            .flex()
            .flex_col()
            .items_start()
            .my(px(d::NAME_MARGIN))
            .child(
                div()
                    .my(px(d::NAME_LABEL_MARGIN))
                    .font_bold()
                    .child(SharedString::from(format!("{}:", t("labels.fileTitle")))),
            )
            .child(
                // A div rather than a real input, the way the link dialog works:
                // `handle_save_key` routes the keystrokes while there is no
                // native text field to focus.
                div()
                    .id("save-filename")
                    .test_support()
                    // `.TextInput` is a real `<input type="text">` upstream, so
                    // it exposes its content as an accessible value.
                    .aria_label(SharedString::from(t("labels.fileTitle")))
                    .aria_value(SharedString::from(self.project_name_value()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(d::NAME_FIELD_W))
                    .h(px(d::NAME_FIELD_H))
                    .px(px(d::NAME_FIELD_PAD))
                    .border(px(d::NAME_FIELD_BORDER))
                    .border_color(tokens.input_border())
                    .hover(move |s| s.border_color(tokens.brand_hover()))
                    .rounded(px(d::NAME_FIELD_RADIUS))
                    .bg(tokens.surface_lowest())
                    .text_size(px(d::NAME_FIELD_FONT))
                    .text_center()
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(self.project_name_value())),
            );

        let card = div()
            .flex()
            .flex_col()
            .items_center()
            .max_w(px(d::CARD_MAX_WIDTH))
            .m(px(d::CARD_MARGIN))
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .p(px(d::CARD_ICON_PAD))
                    .rounded_full()
                    .bg(rgb(CARD_LIME))
                    .child(crate::icons::icon(
                        "exportToFileIcon",
                        d::CARD_ICON,
                        rgb(0xffffff),
                    )),
            )
            .child(
                div()
                    .my(px(d::CARD_TITLE_MARGIN))
                    .text_size(px(d::CARD_TITLE_FONT))
                    .font_bold()
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(t("exportDialog.disk_title"))),
            )
            // `.Card-details`: centred prose with the name field under it. Its
            // `min-height` is what keeps the cards' buttons on one line upstream;
            // here it keeps the button from riding up as the name changes height.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .w_full()
                    .min_h(px(d::CARD_DETAILS_MIN_H))
                    .px(px(d::CARD_DETAILS_PAD))
                    .text_size(px(d::CARD_DETAILS_FONT))
                    .text_center()
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(t("exportDialog.disk_details")))
                    .child(project_name),
            )
            .child(
                div()
                    .id("save-card-button")
                    .test_support()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(d::CARD_BUTTON_H))
                    .mt(px(d::CARD_BUTTON_TOP))
                    .mb(px(d::CARD_BUTTON_BOTTOM))
                    .px(px(d::CARD_BUTTON_PAD))
                    .rounded(px(d::CARD_BUTTON_RADIUS))
                    .bg(rgb(CARD_LIME))
                    .hover(move |s| s.bg(rgb(CARD_LIME_DARKER)))
                    .text_size(px(d::CARD_BUTTON_FONT))
                    .text_color(rgb(0xffffff))
                    .child(SharedString::from(t("exportDialog.disk_button")))
                    // `actionSaveFileToDisk` needs the platform's save panel,
                    // which needs a `Context`; `handle_key` has none, so the
                    // request is queued and `render` starts it next frame.
                    .on_click(Self::act(cx, |this, _, _, _| {
                        this.pending_scene_save = true;
                    })),
            );

        // `Dialog`: a title with an underline, then the card grid.
        let content = div()
            .flex()
            .flex_col()
            .w(px(d::WIDTH))
            .max_w_full()
            .p(px(d::PADDING))
            .bg(tokens.island())
            .border_1()
            .border_color(tokens.dialog_border())
            .rounded(px(d::RADIUS))
            .shadow(crate::design::modal_shadow())
            .occlude()
            .child(
                div()
                    .pb(px(d::TITLE_PAD_BOTTOM))
                    .text_size(px(d::TITLE_FONT))
                    .text_color(tokens.surface_text())
                    .child(SharedString::from(t("menu.save"))),
            )
            .child(div().h(px(1.0)).w_full().bg(tokens.dialog_border()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .w_full()
                    .mt(px(d::TITLE_GAP))
                    .child(card),
            );

        // Upstream's modal is a full-bleed `.Modal__background` with the content
        // centred over it as a *sibling*, so a click on the card cannot reach the
        // backdrop's dismiss handler.
        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .child(
                div()
                    .id("save-dialog-backdrop")
                    .test_support()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .occlude()
                    .bg(tokens.overlay_backdrop())
                    // `Modal`'s `closeOnClickOutside` defaults to true.
                    .on_click(Self::act(cx, |this, _, _, _| this.show_save_dialog = false)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .p(px(d::MODAL_PADDING))
                    .child(content),
            )
            .into_any_element()
    }
}
