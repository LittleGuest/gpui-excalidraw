use excalidraw_ui::Editor;
use gpui_kit::{
    AppContext, Bounds, Entity, Point, TestAppContext, WindowBounds, WindowHandle, WindowOptions,
    px, size, test::TestWindowExt,
};

fn open_editor(cx: &mut TestAppContext) -> (WindowHandle<gpui_kit::base::Root>, Entity<Editor>) {
    open_editor_sized(cx, 800.0, 600.0)
}

/// The same editor in a window of a given size.
///
/// A 600px-tall window is the short case: the hamburger menu outgrows it and
/// scrolls, so its last rows sit below the fold until the user scrolls. Tests
/// that care about those rows ask for a window tall enough to show them, and
/// the ones that care about the cap keep the short window.
fn open_editor_sized(
    cx: &mut TestAppContext,
    width: f32,
    height: f32,
) -> (WindowHandle<gpui_kit::base::Root>, Entity<Editor>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        let bounds = Bounds {
            origin: Point::default(),
            size: size(px(width), px(height)),
        };
        let (window, content) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |_, cx| {
                let mut editor = Editor::new();
                editor.add_demo_scene();
                cx.new(|_| editor)
            },
        )
        .expect("open test window");
        (
            window.downcast::<gpui_kit::base::Root>().expect("root"),
            content,
        )
    })
}

#[gpui_kit::test]
fn editor_renders_toolbar_and_canvas(cx: &mut TestAppContext) {
    let (window, _content) = open_editor(cx);
    cx.update_window(window.into(), |_, _window, cx| {
        _window.render_frame(cx);
    })
    .unwrap();
}

// Tool switching is covered end-to-end by
// `chrome_islands_are_laid_out_and_clickable`, which clicks the real buttons
// rather than calling `set_tool` and claiming it was a click.

/// The chrome rebuild's end-to-end guard.
///
/// The islands are absolutely positioned siblings of the canvas, and every one
/// of them calls `.occlude()`. That is a lot of ways for a click to go missing —
/// a button that paints correctly but never receives its event looks identical
/// in a screenshot. So this test drives the real hit-testing pipeline: find the
/// button by id, click it, assert the view state changed *and* that the canvas
/// underneath did not also react.
#[gpui_kit::test]
fn chrome_islands_are_laid_out_and_clickable(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        // 1. Every control exists, is visible, and has real area. A zero-area
        //    button still "renders" — it is simply unclickable.
        for id in [
            "menu",
            "tool-lock",
            "tool-handIcon",
            "tool-SelectionIcon",
            "tool-RectangleIcon",
            "tool-DiamondIcon",
            "tool-EllipseIcon",
            "tool-ArrowIcon",
            "tool-LineIcon",
            "tool-FreedrawIcon",
            "tool-TextIcon",
            "tool-stickyNoteToolIcon",
            "tool-EraserIcon",
            "more-tools",
            "library",
            "theme",
            "zoom-out",
            "zoom-in",
            "undo",
            "redo",
        ] {
            let snapshot = window.find(id);
            assert!(snapshot.visible(), "{id} is not visible");
            let bounds = snapshot.bounds();
            assert!(
                bounds.size.width >= px(20.0) && bounds.size.height >= px(20.0),
                "{id} is too small to hit: {bounds:?}"
            );
        }

        // 2. The padlock is the head of the toolbar, so it is the cheapest proof
        //    that a click reaches the island rather than the canvas below it.
        window.click("tool-lock", cx);
        content.update(cx, |editor, _| {
            assert!(editor.document.tool_locked, "the lock click never arrived");
        });
        window.click("tool-lock", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.document.tool_locked,
                "the lock click did not toggle"
            );
        });

        // 3. Picking a tool through the UI must not leak into the canvas: if the
        //    canvas had received the same click it would have started a marquee,
        //    and the demo scene would have gained an element.
        let before = content.read(cx).document.scene.len();
        window.click("tool-RectangleIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Rectangle);
            assert_eq!(
                editor.document.scene.len(),
                before,
                "the canvas reacted to a toolbar click"
            );
        });

        // 4. Image / Frame / Laser live one level down. Opening the popover and
        //    clicking a row has to work for them to be reachable at all.
        window.click("more-tools", cx);
        window.render_frame(cx);
        window.click("more-laserPointerToolIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Laser);
            assert!(!editor.is_more_tools_open(), "the popover should close");
        });

        // 5. Drawing after picking a tool must still work, i.e. the occluding
        //    islands only cover themselves.
        window.click("tool-EllipseIcon", cx);
        let before = content.read(cx).document.scene.len();
        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(520.0), px(400.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                before + 1,
                "dragging on the canvas should add an ellipse"
            );
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "an unlocked tool hands back the selection tool"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn hamburger_menu_rows_are_clickable(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));

        // Preferences opens a second-level panel to the right; the menu stays open.
        window.render_frame(cx);
        window.click("fm-prefs", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));

        // The grid toggle lives in that submenu now, and flips in place
        // without dismissing either panel (mirrors upstream).
        window.render_frame(cx);
        window.click("prefs-grid", cx);
        content.update(cx, |editor, _| {
            assert!(editor.document.show_grid, "the grid row did nothing");
            assert!(editor.is_menu_open(), "the submenu should stay open");
        });
    })
    .unwrap();
}

/// The menu is capped and scrolls rather than running off a short window.
///
/// `.dropdown-menu-container` is `overflow-y: auto` with a
/// `calc(100svh - 32px - 36px)` cap. Without it the menu's lower half — the
/// language select and the canvas-background swatches — is painted outside the
/// window and can never be reached.
///
/// The window is deliberately shorter than the default 600px: the menu's
/// natural height is 462.5px, so at 600px the cap (532px) never bites and there
/// is nothing to scroll. Only a window short enough to clip the content shows
/// whether the cap is doing its job.
#[gpui_kit::test]
fn hamburger_menu_is_capped_and_scrolls(cx: &mut TestAppContext) {
    let height = 380.0;
    let (window, _content) = open_editor_sized(cx, 800.0, height);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        let menu = window.find("file-menu").bounds();
        // `100svh - editor-container-padding * 2 - lg-button-size`.
        let cap = height - 32.0 - 36.0;
        assert!(
            menu.size.height <= px(cap),
            "the menu should stop short of the window edge, got {menu:?}"
        );
        // The content is taller than that, so the cap has to be scrolling
        // something rather than simply clipping it away.
        assert!(
            !window.find("fm-lang").visible(),
            "with the cap in place the language select starts below the fold"
        );
    })
    .unwrap();
}

/// The language row is a `<select>`, not a cycling button.
///
/// Upstream's hamburger hosts a native `<select>` listing every locale, so a
/// click on the control opens the options instead of advancing to the next
/// language — which is what the previous `⌄` row did, and what the screenshot
/// review flagged.
#[gpui_kit::test]
fn language_select_is_a_listbox_not_a_cycle(cx: &mut TestAppContext) {
    use excalidraw_ui::Language;

    let (window, content) = open_editor_sized(cx, 800.0, 900.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        let before = content.read(cx).i18n.language();
        window.click("fm-lang", cx);
        content.update(cx, |editor, _| {
            assert!(editor.is_language_open(), "the select never expanded");
            assert_eq!(
                editor.i18n.language(),
                before,
                "opening the list changed the language"
            );
        });

        // Every locale upstream ships is offered, and the panel is capped so the
        // tail of the list scrolls instead of falling off a short window.
        window.render_frame(cx);
        for language in Language::all() {
            let id = gpui_kit::SharedString::from(format!("lang-{}", language.code()));
            assert!(window.try_find(id.clone()).is_some(), "{id} is missing");
        }
        let panel = window.find("lang-list").bounds();
        assert!(
            panel.size.height <= px(900.0 - 32.0 - 36.0),
            "the list should stop short of the window edge, got {panel:?}"
        );
        assert!(
            window.find("lang-en").visible(),
            "the list should open on its first row"
        );

        // Picking one applies it, collapses the list, and leaves the menu up.
        window.click("lang-de-DE", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.i18n.language(), Language::De);
            assert!(!editor.is_language_open(), "the list should collapse");
            assert!(editor.is_menu_open(), "the menu should stay open");
        });

        // Re-opening shows the new selection as the control's label.
        window.render_frame(cx);
        window.click("fm-lang", cx);
        window.render_frame(cx);
        window.click("lang-en", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.i18n.language(), Language::En);
        });
    })
    .unwrap();
}

/// The canvas-background row ends with the custom-colour trigger, not a preset.
///
/// Upstream renders this one `ColorPicker` with `palette={null}` and
/// `type="canvasBackground"`: five tints, a hairline, then the active-colour
/// trigger, whose popup is the hex field and nothing else. The screenshot review
/// flagged our sixth swatch, a hard-coded "transparent" preset that upstream
/// does not offer, in place of that trigger.
#[gpui_kit::test]
fn canvas_background_row_ends_with_the_custom_colour_trigger(cx: &mut TestAppContext) {
    let (window, content) = open_editor_sized(cx, 800.0, 900.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        assert!(
            window.try_find("viewbg-transparent").is_none(),
            "the canvas row must not offer a transparent preset"
        );
        assert!(
            window.try_find("viewbg-custom").is_some(),
            "the canvas row should end with the custom-colour trigger"
        );

        // The tints still paint the canvas.
        window.click("viewbg-f8f9fa", cx);
        assert_eq!(
            content.read(cx).document.view_background_color,
            "#f8f9fa",
            "picking a tint should set the canvas background"
        );

        // The trigger opens upstream's hex-only popup — no palette grid, no
        // shade ramp, no most-used custom colours.
        window.render_frame(cx);
        window.click("viewbg-custom", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("color-hex-field").is_some(),
            "the trigger should open the hex field"
        );
        assert!(
            window.try_find("color-black").is_none(),
            "the canvas-background popup must not show the palette grid"
        );
        assert!(
            window.try_find("shade-0").is_none(),
            "the canvas-background popup must not show the shade ramp"
        );
    })
    .unwrap();
}

/// The shape switcher over a selected element, driven through real clicks.
///
/// Upstream's `ConvertElementTypePopup` floats just below a single selected
/// shape and re-types it. Two things are easy to get wrong and both are covered
/// here: it speaks for exactly one element, and re-picking the type the element
/// already has is a no-op rather than a mutation that quietly bumps its version.
#[gpui_kit::test]
fn shape_switch_retypes_a_lone_selected_element(cx: &mut TestAppContext) {
    use excalidraw_core::element::ElementType;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        // The demo's diamond sits clear of the properties panel, which is where
        // the switcher has to be clickable rather than merely painted.
        let (id, center) = {
            let editor = content.read(cx);
            let base = editor.document.scene.elements[2].base();
            (
                base.id.clone(),
                Point::new(
                    px((base.x + base.width / 2.0) as f32),
                    px((base.y + base.height / 2.0) as f32),
                ),
            )
        };
        window.drag(center, center, cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.selected_count(), 1, "the canvas click missed");
        });

        // The island offers the three generic types and nothing from the linear
        // family — a diamond is not a line.
        window.render_frame(cx);
        for control in [
            "shape-switch",
            "shape-rectangle",
            "shape-diamond",
            "shape-ellipse",
        ] {
            assert!(window.find(control).visible(), "{control} did not paint");
        }
        assert!(
            window.try_find("shape-line").is_none(),
            "a diamond must not be offered a line button"
        );

        // Re-picking the current type must not touch the element at all.
        let version = content
            .read(cx)
            .document
            .scene
            .get(&id)
            .unwrap()
            .base()
            .version;
        window.click("shape-diamond", cx);
        content.update(cx, |editor, _| {
            let element = editor.document.scene.get(&id).unwrap();
            assert_eq!(element.kind(), ElementType::Diamond);
            assert_eq!(element.base().version, version, "a no-op still bumped");
        });

        // Rectangle converts and is undoable.
        window.render_frame(cx);
        window.click("shape-rectangle", cx);
        content.update(cx, |editor, _| {
            let element = editor.document.scene.get(&id).unwrap();
            assert_eq!(
                element.kind(),
                ElementType::Rectangle,
                "the click missed the button"
            );
            assert!(
                element.base().version > version,
                "the conversion was not recorded"
            );
        });
        window.press("ctrl-z", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.get(&id).unwrap().kind(),
                ElementType::Diamond,
                "undo did not restore the shape"
            );
        });

        // Multi-selection hides it, and an element with no shape family (text)
        // never gets one.
        let ids: Vec<String> = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .map(|e| e.id().to_string())
            .collect();
        content.update(cx, |editor, _| {
            editor.document.select_many(ids[..2].to_vec())
        });
        window.render_frame(cx);
        assert!(
            window.try_find("shape-switch").is_none(),
            "multi-select kept the switcher"
        );

        let text_id = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .find(|e| e.kind() == ElementType::Text)
            .map(|e| e.id().to_string())
            .expect("the demo scene holds a text element");
        content.update(cx, |editor, _| editor.document.select(&text_id));
        window.render_frame(cx);
        assert!(
            window.try_find("shape-switch").is_none(),
            "text got a shape switcher"
        );
    })
    .unwrap();
}

/// A click that can only land on locked geometry offers to unlock it.
///
/// Nothing in the selection tool reacts to a locked element — `element_at`
/// deliberately skips them — so without the bubble a locked shape would be
/// unreachable with the mouse. The rule is upstream's: the padlock appears only
/// when the click hit nothing already selected, and a single click on it both
/// unlocks and selects the element it names.
#[gpui_kit::test]
fn canvas_click_on_a_locked_element_offers_an_unlock_bubble(cx: &mut TestAppContext) {
    use excalidraw_core::element::{Element, ElementBase, ElementType};

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        // A rectangle parked in the clear middle of the canvas: the demo scene's
        // first row sits under the properties panel, where a click would be
        // swallowed by the panel rather than reaching the canvas.
        let mut base = ElementBase::new("unlock-target".to_string(), ElementType::Rectangle);
        base.x = 300.0;
        base.y = 250.0;
        base.width = 150.0;
        base.height = 100.0;
        content.update(cx, |editor, _| {
            editor.document.scene.add(Element::Rectangle { base });
            // Locking lives on the multi-selection row, so a second element has
            // to be along for the ride.
            let other = editor.document.scene.elements[1].id().to_string();
            editor
                .document
                .select_many(vec!["unlock-target".to_string(), other]);
        });

        window.render_frame(cx);
        window.click("arrange-lock", cx);
        content.update(cx, |editor, _| {
            assert!(
                editor
                    .document
                    .scene
                    .get("unlock-target")
                    .unwrap()
                    .base()
                    .locked,
                "the panel's padlock never locked the selection"
            );
        });

        // Clicking the locked shape selects nothing, which is exactly the state
        // that arms the bubble.
        window.render_frame(cx);
        let centre = Point::new(px(375.0), px(300.0));
        window.drag(centre, centre, cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.selected_count(),
                0,
                "a locked element should not become the selection directly"
            );
        });

        window.render_frame(cx);
        let popup = window.find("unlock-popup").bounds();
        let bottom_edge = popup.origin.y + popup.size.height;
        assert!(
            bottom_edge <= px(250.0) && bottom_edge >= px(230.0),
            "the padlock should sit just above the element's top edge, got {popup:?}"
        );

        // One click unlocks and selects.
        window.click("unlock-popup", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor
                    .document
                    .scene
                    .get("unlock-target")
                    .unwrap()
                    .base()
                    .locked,
                "the bubble did not unlock the element"
            );
            assert_eq!(editor.selected_count(), 1);
            assert!(editor.document.selected.contains("unlock-target"));
        });
        window.render_frame(cx);
        assert!(
            window.try_find("unlock-popup").is_none(),
            "the bubble should clear once it has done its job"
        );
    })
    .unwrap();
}

/// The canvas-buttons island only exists for the two server-backed element kinds.
///
/// Upstream hangs it off a magic frame (convert the sketch to code) and off a
/// *finished* generated iframe (copy the source, go fullscreen) — a plain
/// embeddable or a still-generating iframe gets nothing. The trigger conditions
/// are the whole point of the control, so both the positive and the negative
/// cases are driven through the real hit pipeline here.
#[gpui_kit::test]
fn canvas_buttons_track_a_generated_iframe(cx: &mut TestAppContext) {
    use excalidraw_core::element::{
        Element, ElementBase, ElementType, FrameElement, IframeElement,
    };

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let mut base = ElementBase::new("gen-iframe".to_string(), ElementType::Iframe);
        base.x = 300.0;
        base.y = 250.0;
        base.width = 200.0;
        base.height = 150.0;
        base.link = Some("https://example.com/embed".to_string());
        base.custom_data = Some(serde_json::json!({ "generationData": { "status": "done" } }));
        content.update(cx, |editor, _| {
            editor
                .document
                .scene
                .add(Element::Iframe(IframeElement { base }));
            editor.document.select("gen-iframe");
        });

        window.render_frame(cx);
        assert!(
            window.find("canvas-copyIcon").visible(),
            "a finished generated iframe should offer a copy button"
        );
        assert!(window.find("canvas-fullscreenIcon").visible());
        let island = window.find("canvas-buttons").bounds();
        // Upstream anchors the island 10px off the element's top-right corner.
        assert!(
            (island.origin.x.to_f64() - 510.0).abs() < 1.0,
            "the island should hang off the right edge, got {island:?}"
        );
        assert!(
            (island.origin.y.to_f64() - 250.0).abs() < 1.0,
            "the island should start at the element's top edge, got {island:?}"
        );

        window.click("canvas-copyIcon", cx);

        // A still-generating iframe is left alone.
        content.update(cx, |editor, _| {
            editor
                .document
                .scene
                .get_mut("gen-iframe")
                .unwrap()
                .base_mut()
                .custom_data =
                Some(serde_json::json!({ "generationData": { "status": "pending" } }));
        });
        window.render_frame(cx);
        assert!(
            window.try_find("canvas-buttons").is_none(),
            "an unfinished generation must not get the island"
        );

        // A magic frame offers exactly the one action upstream gives it.
        content.update(cx, |editor, _| {
            editor.document.scene.remove("gen-iframe");
            let mut base = ElementBase::new("magic".to_string(), ElementType::MagicFrame);
            base.x = 300.0;
            base.y = 250.0;
            base.width = 200.0;
            base.height = 150.0;
            editor
                .document
                .scene
                .add(Element::MagicFrame(FrameElement { base, name: None }));
            editor.document.select("magic");
        });
        window.render_frame(cx);
        assert!(window.find("canvas-MagicIcon").visible());
        assert!(
            window.try_find("canvas-copyIcon").is_none(),
            "a magic frame is not a copy-source element"
        );
    })
    .unwrap();

    // Copying the source is a real clipboard write, so assert on the clipboard
    // rather than on the transient status line.
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://example.com/embed"));
}

/// Keyboard input has to survive the real dispatch path.
///
/// Nothing in this view is focusable, so GPUI falls back to dispatching at the
/// *root* node and bubbles upwards from there. The key handlers used to hang
/// off the full-bleed canvas child, where that path never reaches them — every
/// shortcut, Escape included, was silently dead. Calling `handle_key` directly
/// would not catch this: only a real dispatch does.
#[gpui_kit::test]
fn keyboard_shortcuts_survive_goroot_level_dispatch(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        window.press("r", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.tool,
                Tool::Rectangle,
                "the r shortcut never reached the editor"
            );
        });

        window.press("e", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Eraser, "the e shortcut is dead");
        });

        window.press("v", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Selection, "v is dead");
        });

        // Escape is the one shortcut that has to work while a popover is open.
        window.click("menu", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));
        window.press("escape", cx);
        content.update(cx, |editor, _| {
            assert!(!editor.is_menu_open(), "escape did not reach the editor");
        });
    })
    .unwrap();
}

/// Undo through the keyboard, end to end.
#[gpui_kit::test]
fn ctrl_z_undoes_through_the_real_dispatch_path(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        let before = content.read(cx).document.scene.len();

        window.click("tool-RectangleIcon", cx);
        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(500.0), px(380.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.scene.len(), before + 1);
            assert_eq!(editor.document.tool, Tool::Selection);
        });

        window.press("ctrl-z", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                before,
                "ctrl-z never reached the editor"
            );
        });
    })
    .unwrap();
}

/// The command palette has to open on Ctrl+/, filter on what is typed, and
/// actually run the highlighted command. Driving it through key dispatch rather
/// than calling `open_palette` is the point: the overlay owns the keyboard while
/// it is up, so a filter keystroke must land in the query box instead of
/// switching the tool underneath it.
#[gpui_kit::test]
fn ctrl_slash_filters_and_runs_the_command_palette(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        window.press("ctrl-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        // Typing while the palette is up must filter it, not pick the rectangle
        // tool via the bare "r" shortcut.
        window.input("rect", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.palette_query(), "rect");
            let results: Vec<&str> = editor
                .palette_results()
                .into_iter()
                .map(|(id, ..)| id)
                .collect();
            assert_eq!(
                results,
                vec!["tool-rectangle"],
                "the filter is not matching"
            );
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_palette_open(),
                "running a command should dismiss it"
            );
            assert_eq!(editor.document.tool, Tool::Rectangle);
        });
    })
    .unwrap();
}

/// Clicking a palette row must reach the row, not the backdrop that sits behind
/// it — the backdrop is a sibling precisely so it cannot shadow the panel.
#[gpui_kit::test]
fn palette_rows_are_clickable_over_the_backdrop(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("tool-RectangleIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Rectangle)
        });

        window.click("menu", cx);
        window.render_frame(cx);
        window.click("fm-palette", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        // An empty filter puts the Selection tool first; clicking that row must run it.
        window.render_frame(cx);
        window.click("palette-row-0", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "the row click missed"
            );
            assert!(!editor.is_palette_open());
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn palette_escape_dismisses_without_running(cx: &mut TestAppContext) {
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        window.press("escape", cx);
        content.update(cx, |editor, _| {
            assert!(!editor.is_palette_open());
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "escape ran a command"
            );
        });
    })
    .unwrap();
}

/// "Save to…" raises upstream's export dialog, through the real frame loop.
///
/// The menu row and Ctrl+S both ran `save_to("scene.excalidraw")` and dropped
/// the result, so the dialog upstream shows never appeared and the file landed
/// in the process working directory. Upstream raises `JSONExportDialog` from the
/// menu row and binds the shortcut to `actionSaveFileToDisk`, which is the same
/// dialog's button — so both paths have to end up here.
#[gpui_kit::test]
fn save_raises_the_export_dialog(cx: &mut TestAppContext) {
    let (window, content) = open_editor_sized(cx, 1000.0, 800.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("save-dialog-backdrop").is_none(),
            "the dialog must not be up before it is asked for"
        );

        window.click("menu", cx);
        window.render_frame(cx);
        window.click("fm-save", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            assert!(editor.save_dialog_open(), "the menu row raised nothing");
            let untitled = editor.i18n.t("labels.untitled");
            // Upstream's `App.getName()`: only a scene that has never been named
            // falls back to `Untitled-<date>`.
            assert!(
                editor.project_name_value().starts_with(&untitled),
                "the name field should show App.getName()'s fallback, got {:?}",
                editor.project_name_value()
            );
        });

        // The modal paints, and the card hugs its own content instead of
        // stretching across the 800px dialog — the name field is pinned at
        // `ProjectName.scss`'s intrinsic width, the button at its label's.
        assert!(window.find("save-dialog-backdrop").visible());
        let field = window.find("save-filename").bounds();
        assert!(
            field.size.width > px(120.0) && field.size.width < px(240.0),
            "the name field is the wrong width: {field:?}"
        );
        // `.Modal` centres the content both ways, and the card is a single
        // centred column inside it — so the name field sits on the window's
        // vertical axis.
        let field_centre = field.origin.x + field.size.width / 2.0;
        assert!(
            (field_centre - px(500.0)).abs() < px(12.0),
            "the dialog is not centred: {field:?}"
        );
        let button = window.find("save-card-button").bounds();
        assert!(
            button.size.width > px(60.0) && button.size.width < px(240.0),
            "the card's button should hug its label: {button:?}"
        );
        assert!(
            window.find("save-card-button").visible(),
            "the card's button is off-screen"
        );

        // Typing has to reach the field *and* repaint it.
        let seeded = content.update(cx, |editor, _| editor.project_name_value());
        window.input("x", cx);
        content.update(cx, |editor, _| {
            assert!(
                editor.project_name_value().ends_with('x'),
                "typing did not reach the name field: {:?}",
                editor.project_name_value()
            );
        });
        assert_eq!(
            window.find("save-filename").value(),
            Some(format!("{seeded}x").as_str()),
            "the field did not repaint after a keystroke"
        );

        // Escape dismisses the modal, and the shortcut raises it again.
        window.press("escape", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.save_dialog_open(),
                "escape did not dismiss the modal"
            );
        });

        window.press("ctrl-s", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(editor.save_dialog_open(), "Ctrl+S raised nothing");
        });
    })
    .unwrap();
}

/// Find on canvas: Ctrl+F, type a needle, and the match count plus the stepper
/// have to come from the real scene rather than a stub.
#[gpui_kit::test]
fn ctrl_f_finds_and_cycles_text_matches(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-f", cx);
        content.update(cx, |editor, _| assert!(editor.is_find_open()));

        // The demo scene holds two text elements, "Excalidraw" and "Note".
        window.input("e", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_query(), "e");
            assert_eq!(editor.find_match_count(), 2);
            assert_eq!(editor.find_index(), 0);
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_index(), 1, "enter did not step")
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_index(), 0, "the stepper should wrap");
        });

        window.press("escape", cx);
        content.update(cx, |editor, _| assert!(!editor.is_find_open()));
    })
    .unwrap();
}

/// The Help dialog opens on Shift+/ (the "?" key) and Escape closes it.
#[gpui_kit::test]
fn help_dialog_opens_on_question_mark(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("shift-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_help_open()));

        window.render_frame(cx);
        assert!(
            window.find("help-close").visible(),
            "the help dialog did not paint"
        );

        window.press("escape", cx);
        content.update(cx, |editor, _| assert!(!editor.is_help_open()));
    })
    .unwrap();
}

/// The UI's PNG path must emit a real PNG buffer, not an SVG renamed.
#[gpui_kit::test]
fn png_export_from_the_editor_is_a_real_png(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let bytes = content
            .update(cx, |editor, _| editor.export_png_bytes(2.0))
            .expect("export should succeed");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a PNG signature");

        let path = std::env::temp_dir().join("excalidraw-ui-test-export.png");
        let path_string = path.to_string_lossy().into_owned();
        content
            .update(cx, |editor, _| editor.export_png_to(&path_string))
            .expect("writing the PNG should succeed");
        let written = std::fs::read(&path).expect("the exported file should exist");
        assert_eq!(&written[..8], b"\x89PNG\r\n\x1a\n");
        let _ = std::fs::remove_file(&path);
    })
    .unwrap();
}

/// The stroke row's eye-dropper trigger starts a real pick, and the backdrop
/// commits whatever colour sits under the pointer.
///
/// The trigger has to appear without any prior interaction and the backdrop has
/// to out-rank the canvas underneath it — otherwise the click that should commit
/// a sample would instead start a marquee on the canvas.
#[gpui_kit::test]
fn eye_dropper_picks_the_colour_under_the_pointer(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        let before = content
            .read(cx)
            .document
            .scene
            .get(&target)
            .unwrap()
            .base()
            .stroke_color
            .clone();
        let expected = content.read(cx).export_background().to_ascii_lowercase();

        // The eye dropper lives inside the picker popup (upstream puts it on the
        // hex row), so the trigger has to be revealed before it can be clicked.
        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        window.click("eye-dropper-stroke", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                editor.is_eye_dropper_active(),
                "the trigger never armed the picker"
            );
        });
        assert!(
            window.find("eye-dropper").visible(),
            "the backdrop did not paint"
        );

        // The demo's shapes sit along the top of the canvas, so this point is
        // bare canvas: the sample is the background, not a shape's fill.
        window.click_at("eye-dropper", Point::new(px(400.0), px(275.0)), cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_eye_dropper_active(),
                "clicking should end the pick"
            );
            let stroke = &editor
                .document
                .scene
                .get(&target)
                .unwrap()
                .base()
                .stroke_color;
            assert_ne!(stroke, &before, "the picked colour never landed");
            assert_eq!(
                stroke.to_ascii_lowercase(),
                expected,
                "the pick should have sampled the canvas background"
            );
        });
    })
    .unwrap();
}

/// Escape abandons a pick without writing a colour.
#[gpui_kit::test]
fn escape_abandons_an_eye_dropper_pick(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);
        let before = content
            .read(cx)
            .document
            .scene
            .get(&target)
            .unwrap()
            .base()
            .stroke_color
            .clone();

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        window.click("eye-dropper-stroke", cx);
        content.update(cx, |editor, _| assert!(editor.is_eye_dropper_active()));

        window.press("escape", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_eye_dropper_active(),
                "escape should cancel the pick"
            );
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                before,
                "a cancelled pick must not write a colour"
            );
        });
        assert!(
            window.try_find("eye-dropper").is_none(),
            "the backdrop should be torn down"
        );
    })
    .unwrap();
}

/// The arrow shortcut shows the hint beside the pointer, a press hides it, and
/// the timer clears it on its own.
///
/// The hint only exists once the pointer has a position (upstream bails on the
/// untouched `viewport.lastPosition`), so this drives a real pointer move first
/// rather than asserting on a stub.
#[gpui_kit::test]
fn arrow_shortcut_shows_a_hint_beside_the_pointer(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        // A zero-length drag is the cheapest way to park the pointer on the
        // canvas; it also proves the hint is not shown before that happens.
        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(400.0), px(300.0)),
            cx,
        );

        window.press("a", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.cursor_hint_icon(),
                Some("sharpArrowIcon"),
                "the letter shortcut should hint the arrow tool"
            );
        });
        assert!(
            window.find("cursor-hint").visible(),
            "the hint did not paint"
        );

        // Starting to interact hides it at once rather than waiting for the fade.
        window.drag(
            Point::new(px(420.0), px(320.0)),
            Point::new(px(420.0), px(320.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.cursor_hint_icon(),
                None,
                "a press should hide the hint"
            );
        });

        // A digit bypasses the cooldown, so the timer case gets a fresh hint.
        window.press("5", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.cursor_hint_icon(), Some("sharpArrowIcon"));
        });
    })
    .unwrap();

    // Past the 700ms show + 100ms fade window the hint must clear itself.
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(900));
    cx.run_until_parked();
    let icon = cx
        .update_window(window.into(), |_, _, cx| {
            content.read(cx).cursor_hint_icon()
        })
        .unwrap();
    assert_eq!(icon, None, "the hint should auto-hide");
}

/// An OS file drag paints the overlay, and a scene dropped without shift
/// replaces the canvas and reports itself.
#[gpui_kit::test]
fn external_file_drag_shows_the_overlay_and_a_scene_drop_replaces(cx: &mut TestAppContext) {
    use std::path::PathBuf;

    use gpui_kit::{ExternalPaths, FileDropEvent, PlatformInput};

    let (window, content) = open_editor(cx);

    let path = std::env::temp_dir().join("gpui-excalidraw-ui-drop-test.excalidraw");
    let path_string = path.to_string_lossy().into_owned();
    cx.update(|cx| {
        content
            .update(cx, |editor, _| editor.save_to(&path_string))
            .expect("exporting the demo scene should succeed");
    });
    let saved_len = cx.update(|cx| content.read(cx).document.scene.len());

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        // `Entered` is the only event the platform sends when a drag arrives; it
        // parks the paths in the active drag and synthesises the pointer move the
        // overlay tracks.
        let paths = ExternalPaths([PathBuf::from(&path_string)].into_iter().collect());
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Entered {
                position: Point::new(px(400.0), px(300.0)),
                paths,
            }),
            cx,
        );
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.file_drag_kind(),
                Some("scene"),
                "a .excalidraw file is a scene"
            );
        });
        assert!(window.find("file-drop-overlay").visible());

        // Leaving tears it down again.
        window.dispatch_event(PlatformInput::FileDrop(FileDropEvent::Exited), cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| assert_eq!(editor.file_drag_kind(), None));
        assert!(window.try_find("file-drop-overlay").is_none());

        // Empty the canvas so the drop's restore is unmistakable.
        content.update(cx, |editor, _| editor.document.scene.elements.clear());
        window.render_frame(cx);

        let paths = ExternalPaths([PathBuf::from(&path_string)].into_iter().collect());
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Entered {
                position: Point::new(px(400.0), px(300.0)),
                paths,
            }),
            cx,
        );
        window.render_frame(cx);
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Submit {
                position: Point::new(px(400.0), px(300.0)),
            }),
            cx,
        );
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                saved_len,
                "the drop did not load the file"
            );
            let message = editor
                .toast_message()
                .expect("the replace should report itself");
            assert!(
                message.contains("Content replaced"),
                "unexpected toast text: {message}"
            );
        });
        assert!(window.find("toast").visible(), "the toast did not paint");

        window.click("toast-close", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.toast_message(), None, "the close button did nothing");
        });
    })
    .unwrap();

    let _ = std::fs::remove_file(&path);
}

/// The stroke row's colour trigger opens a real popup, and both a grid swatch
/// and a ramp step write through to the selected element.
///
/// The palette used to sit flat in the panel; it now hangs off the trigger, so
/// this drives the popup end-to-end rather than trusting the new render path.
#[gpui_kit::test]
fn colour_picker_popup_writes_the_clicked_swatch(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        // Closed until asked for: the grid is not in the tree at all.
        assert!(
            window.try_find("color-red").is_none(),
            "the palette should not be painted before the trigger is clicked"
        );

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        assert!(
            window.find("color-red").visible(),
            "the trigger never opened the palette"
        );
        assert!(
            window.find("color-hex-field").visible(),
            "the hex row is missing"
        );
        assert!(
            window.find("shade-0").visible(),
            "a ramped colour should offer its ramp"
        );

        // The green ellipse (index 1) starts on a blue stroke; picking red at the
        // active step has to land on red's darkest shade (#e03131).
        window.click("color-red", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                "#e03131",
                "clicking a swatch should write that colour"
            );
        });

        // Stepping the ramp keeps the popup up and re-picks from the same entry.
        window.click("shade-0", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                "#fff5f5",
                "stepping the ramp should move the whole grid to that shade"
            );
        });

        // The trigger is a toggle.
        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("color-red").is_none(),
            "clicking the trigger again should close the popup"
        );
    })
    .unwrap();
}

/// The typography row's trigger opens the font picker, its quick search filters
/// the list, and clicking a row changes the text element's family.
#[gpui_kit::test]
fn font_picker_popup_searches_and_selects(cx: &mut TestAppContext) {
    use excalidraw_core::element::Element;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let text_id = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .find_map(|element| match element {
                Element::Text(_) => Some(element.id().to_string()),
                _ => None,
            })
            .expect("the demo scene has a text element");
        content.update(cx, |editor, _| editor.document.select(&text_id));
        window.render_frame(cx);

        assert!(
            window.try_find("font-search").is_none(),
            "the list starts closed"
        );
        window.click("font-trigger", cx);
        window.render_frame(cx);
        assert!(
            window.find("font-search").visible(),
            "the trigger did not open the list"
        );
        assert!(
            window.find("font-item-Comic Shanns").visible(),
            "every family should be listed"
        );

        // The quick search is a real filter: "comic" leaves only the one family.
        window.input("comic", cx);
        window.render_frame(cx);
        assert!(window.find("font-item-Comic Shanns").visible());
        assert!(
            window.try_find("font-item-Nunito").is_none(),
            "a needle should hide the families it does not match"
        );

        window.click("font-item-Comic Shanns", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            let family = match editor.document.scene.get(&text_id).unwrap() {
                Element::Text(text) => text.font_family,
                _ => panic!("the text element changed type"),
            };
            assert_eq!(
                family,
                excalidraw_core::types::FontFamily::ComicShanns,
                "clicking a row should set the family"
            );
        });
    })
    .unwrap();
}

/// The link control opens the dialog, a typed URL is written on confirm, and the
/// remove button plus confirm clears it again.
#[gpui_kit::test]
fn element_link_dialog_writes_and_clears_the_link(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        assert!(
            window.try_find("link-input").is_none(),
            "the dialog starts closed"
        );
        window.click("single-link", cx);
        window.render_frame(cx);
        assert!(
            window.find("link-input").visible(),
            "the control did not open the dialog"
        );

        // No link yet, so upstream hides the remove button.
        assert!(window.try_find("link-remove").is_none());

        window.input("example.com", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.link_input(),
                "example.com",
                "typing should reach the field"
            );
        });

        window.click("link-confirm", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.selected_link().as_deref(), Some("example.com"));
        });
        assert!(
            window.try_find("link-input").is_none(),
            "confirming should dismiss the dialog"
        );

        // Reopening seeds the field and now offers the remove button, which
        // empties the field so the confirm clears the element's link.
        window.click("single-link", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.link_input(),
                "example.com",
                "the field should be seeded"
            );
        });
        window.click("link-remove", cx);
        window.render_frame(cx);
        window.click("link-confirm", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.selected_link(),
                None,
                "confirming an empty field clears the link"
            );
        });
    })
    .unwrap();
}

/// Ctrl+V with text on the system clipboard lays that text onto the canvas.
///
/// Upstream splits a multi-line paste into one text element per line and turns
/// a blank line into a paragraph gap rather than an empty element, so "first",
/// a blank line and "second" have to arrive as exactly two elements. Driving
/// this through the real dispatch path is the point: the key handler used to
/// hand Ctrl+V to the text branch, which discards every modified keystroke, so
/// a canvas paste never happened at all.
#[gpui_kit::test]
fn ctrl_v_pastes_clipboard_text_as_text_elements(cx: &mut TestAppContext) {
    use excalidraw_core::element::Element;

    let (window, content) = open_editor(cx);

    cx.update(|cx| {
        cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(
            "first\n\nsecond".to_string(),
        ));
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-v", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            let bodies: Vec<String> = editor
                .document
                .selected_elements()
                .iter()
                .filter_map(|element| match element {
                    Element::Text(text) => Some(text.text.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(
                bodies,
                vec!["first".to_string(), "second".to_string()],
                "the paste should make one text element per non-blank line"
            );
        });
    })
    .unwrap();
}

/// Ctrl+V while a text element is being edited joins that element instead of
/// creating new ones.
///
/// A freshly placed text element starts in replace-on-type mode, so the paste
/// has to clear the "Text" placeholder first — the same thing typing a
/// character does.
#[gpui_kit::test]
fn ctrl_v_while_editing_pastes_into_the_text_element(cx: &mut TestAppContext) {
    use excalidraw_core::element::Element;
    use excalidraw_ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update(|cx| {
        cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string("pasteme".to_string()));
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        // A click with the text tool places a fresh element and opens it for
        // editing.
        let id = content.update(cx, |editor, _| {
            editor.set_tool(Tool::Text);
            editor.on_canvas_mouse_down(400.0, 300.0, gpui_kit::Modifiers::default(), 1);
            editor.editing_text_id().map(str::to_string)
        });
        let id = id.expect("the text tool should open a new element for editing");

        window.press("ctrl-v", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            let element = editor
                .document
                .scene
                .get(&id)
                .expect("the text element vanished");
            match element {
                Element::Text(text) => assert_eq!(text.text, "pasteme"),
                _ => panic!("the paste changed the element type"),
            }
        });
    })
    .unwrap();
}
