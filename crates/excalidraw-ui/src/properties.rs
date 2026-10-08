//! The shape-properties island (upstream: `.panel` + `PanelComponent`).
//!
//! Excalidraw's panel is **icon-only**: rows of same-sized controls separated by
//! hairlines, with no column of text labels. That is deliberate on their side —
//! the icons are the labels — and it is what makes the panel read as Excalidraw
//! rather than as a form. Every control here is a 32px `.ctrl` square tinted
//! from [`crate::design::Tokens`].

use excalidraw_core::{
    arrowhead::Arrowhead,
    types::{FillStyle, FontFamily, Roundness, RoundnessType, StrokeStyle, TextAlign},
};
use gpui_kit::*;

use crate::{
    design::{Tokens, island_shadow, size},
    editor::{ColorTarget, Editor},
    icons::icon,
};

/// Resolve the island palette from the document theme.
fn tokens_of(editor: &Editor) -> Tokens {
    let palette = match editor.document.theme {
        excalidraw_core::types::Theme::Dark => crate::design::Palette::Dark,
        excalidraw_core::types::Theme::Light => crate::design::Palette::Light,
    };
    Tokens::for_palette(palette)
}

pub(crate) fn parse_hex(hex: &str) -> Rgba {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255) as u32;
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255) as u32;
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255) as u32;
        rgb((r << 16) | (g << 8) | b)
    } else {
        rgb(0xffffff)
    }
}

/// A `.ctrl` — the square icon button every enumeration in the panel uses.
///
/// A checked control keeps its container fill and, matching upstream's CSS
/// order, does **not** light up on hover: `.ctrl:hover` is declared before
/// `.ctrl.checked`, so `.checked` wins.
fn ctrl(
    id: impl Into<ElementId>,
    icon_name: &str,
    checked: bool,
    tokens: Tokens,
) -> crate::chrome::Button {
    let fg = if checked {
        tokens.selected_fg()
    } else {
        tokens.surface_text()
    };
    let base = div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size::BUTTON))
        .rounded(px(size::RADIUS_MD))
        .border_1()
        .border_color(if checked {
            tokens.selected_bg()
        } else {
            tokens.border()
        })
        .text_color(fg)
        .child(icon(icon_name, size::ICON, fg));
    if checked {
        base.bg(tokens.selected_bg())
    } else {
        base.hover(move |s| s.bg(tokens.hover()))
    }
}

/// `.ctrl` with a text glyph instead of an icon — used by the opacity steps,
/// which upstream renders as a slider rather than an icon set.
fn ctrl_text(
    id: impl Into<ElementId>,
    label: &str,
    checked: bool,
    tokens: Tokens,
) -> Stateful<Div> {
    let fg = if checked {
        tokens.selected_fg()
    } else {
        tokens.surface_text()
    };
    let base = div()
        .id(id)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .h(px(size::BUTTON))
        .min_w(px(size::BUTTON))
        .px(px(6.0))
        .rounded(px(size::RADIUS_MD))
        .border_1()
        .border_color(if checked {
            tokens.selected_bg()
        } else {
            tokens.border()
        })
        .text_size(px(11.0))
        .text_color(fg)
        .child(SharedString::from(label.to_owned()));
    if checked {
        base.bg(tokens.selected_bg())
    } else {
        base.hover(move |s| s.bg(tokens.hover()))
    }
}

/// A colour swatch — upstream `.color-picker__button`: a flat 22×22 square,
/// radius 4, painted in the colour itself (27px pitch ⇒ 5px gap). The only
/// treatment is the selected state: a 1px inset ring in
/// `--color-primary-darkest` (measured `rgb(74,71,177) 0 0 0 1px` on the
/// outline layer). Pastels and the transparent slot stay completely flat —
/// upstream gives them no hairline at all.
fn swatch(id: String, color: Rgba, checked: bool, tokens: Tokens) -> Stateful<Div> {
    let swatch = div()
        .id(id)
        .flex_shrink_0()
        .size(px(size::SWATCH))
        .rounded(px(size::SWATCH_RADIUS))
        .bg(color);
    if checked {
        swatch.border_1().border_color(tokens.primary_darkest())
    } else {
        swatch
    }
}

/// A hairline between groups of rows.
fn rule(tokens: Tokens) -> Div {
    div().w_full().h(px(1.0)).bg(tokens.separator())
}

/// A row of controls that wraps when the panel is too narrow.
fn row(children: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(8.0))
        .children(children)
}

/// The eye-dropper trigger that closes a colour row (upstream
/// `.excalidraw-eye-dropper-trigger`).
///
/// It is deliberately shorter than the row's swatches: upstream's trigger is a
/// 20px chip, so squeezing it to 22 would read as a fifth swatch.
fn eye_dropper_trigger(
    id: &'static str,
    target: ColorTarget,
    active: bool,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> crate::chrome::Button {
    // `.excalidraw-eye-dropper-trigger.selected` takes `--color-primary` for
    // its glyph, not the darker `--color-on-primary-container` a `.ctrl` uses.
    let fg = if active {
        tokens.accent()
    } else {
        tokens.surface_text()
    };
    let base = div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .mr(px(size::eye_dropper::TRIGGER_MARGIN_RIGHT))
        .ml(px(size::eye_dropper::TRIGGER_MARGIN_LEFT))
        .size(px(size::eye_dropper::TRIGGER))
        .rounded(px(size::eye_dropper::TRIGGER_RADIUS))
        .text_color(fg)
        .child(icon("eyeDropperIcon", size::eye_dropper::TRIGGER_ICON, fg));
    let base = if active {
        base.bg(tokens.selected_bg())
    } else {
        base.hover(move |s| s.bg(tokens.hover()))
    };
    base.on_click(Editor::act(cx, move |this, _, _, _| {
        this.toggle_eye_dropper(target)
    }))
}

/// A row of colour swatches — upstream wraps them at a 27px pitch, i.e. a 5px
/// gap between 22px swatches (tighter than the 8px control-row gap).
fn swatch_row(children: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(5.0))
        .children(children)
}

/// The paintable fill for a palette colour.
///
/// `transparent` is the one entry that cannot be painted as itself: a
/// fully-transparent square would read as a hole punched in the panel, so it
/// falls back to the surface behind it.
fn palette_fill(color: &str, tokens: Tokens) -> Rgba {
    if color == "transparent" {
        tokens.island()
    } else {
        parse_hex(color)
    }
}

/// A `.color-picker__button--large` swatch — the 30×30 tile the popup's grid and
/// shade rows are built from.
///
/// `has_outline` is upstream's `isColorDark(color, COLOR_OUTLINE_CONTRAST_THRESHOLD)`:
/// pale swatches get a hairline so they do not dissolve into a white popup.
fn large_swatch(
    id: String,
    color: &str,
    active: bool,
    has_outline: bool,
    tokens: Tokens,
) -> crate::chrome::Button {
    let tile = div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .size(px(size::SWATCH_LARGE))
        .rounded(px(size::SWATCH_LARGE_RADIUS))
        .bg(palette_fill(color, tokens));
    if active {
        tile.border_1().border_color(tokens.primary_darkest())
    } else if has_outline {
        tile.border_1().border_color(tokens.swatch_outline())
    } else {
        tile
    }
}

/// Whether a swatch needs upstream's `has-outline` hairline to stay visible.
///
/// Upstream compares the colour's luminance against
/// `COLOR_OUTLINE_CONTRAST_THRESHOLD` (240); a value above it is too pale to
/// read without a border.
pub(crate) fn needs_outline(color: &str) -> bool {
    if color == "transparent" {
        return true;
    }
    let c = parse_hex(color);
    let luminance = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
    luminance > 0.82
}

/// A `.color-picker__heading` — the small caption above each section.
fn picker_heading(text: &str, tokens: Tokens) -> Div {
    div()
        .px(px(size::PICKER_GRID_PAD))
        .text_size(px(12.0))
        .text_color(tokens.surface_text())
        .child(SharedString::from(text.to_owned()))
}

/// `.color-picker-content--default` — a five-column grid of large swatches.
///
/// The width is pinned rather than left to wrap: `repeat(5, 1.875rem)` plus a
/// `.25rem` gap and `.5rem` padding, which is what fixes the popup at 13rem.
fn swatch_grid(children: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(size::PICKER_GRID_GAP))
        .p(px(size::PICKER_GRID_PAD))
        .w(px(size::PICKER_GRID_PAD * 2.0
            + size::SWATCH_LARGE * 5.0
            + size::PICKER_GRID_GAP * 4.0))
        .children(children)
}

/// Upstream's `ButtonSeparator`: a 1px, 1rem hairline centred in its column.
pub(crate) fn button_separator(tokens: Tokens) -> Div {
    div()
        .flex_shrink_0()
        .w(px(15.0))
        .flex()
        .justify_center()
        .child(div().w(px(1.0)).h(px(16.0)).bg(tokens.border()))
}

/// Where a colour target's top picks and default ramp step come from.
fn stroke_target(target: ColorTarget) -> bool {
    matches!(target, ColorTarget::Stroke)
}

/// One `.color-picker-container` row of the panel: the top-picks strip, a
/// hairline, then the active-colour trigger that opens the picker.
fn color_picker_row(
    editor: &Editor,
    target: ColorTarget,
    color: &str,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> Div {
    let picks: &[&'static str] = if stroke_target(target) {
        &crate::theme::STROKE_TOP_PICKS
    } else {
        &crate::theme::BACKGROUND_TOP_PICKS
    };
    let prefix = if stroke_target(target) {
        "top-stroke"
    } else {
        "top-bg"
    };

    let children: Vec<AnyElement> = picks
        .iter()
        .map(|c| {
            let c = *c;
            swatch(
                format!("{prefix}-{c}"),
                palette_fill(c, tokens),
                c == color,
                tokens,
            )
            .on_click(Editor::act(cx, move |this, _, _, _| match target {
                ColorTarget::Stroke => this.document.set_selected_stroke_color(c),
                ColorTarget::Background => this.document.set_selected_background_color(c),
                ColorTarget::CanvasBackground => {
                    this.document.view_background_color = c.to_string()
                }
            }))
            .into_any_element()
        })
        .collect();

    let id = if stroke_target(target) {
        "color-trigger-stroke"
    } else {
        "color-trigger-bg"
    };
    let open = editor.color_popup() == Some(target);

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .py(px(4.0))
        .child(div().flex_1().child(swatch_row(children)))
        .child(button_separator(tokens))
        .child(color_trigger(id, target, color, open, tokens, cx))
}

/// `.color-picker__button.active-color` — the trigger showing the element's
/// current colour, and the only way into the picker popup.
pub(crate) fn color_trigger(
    id: &'static str,
    target: ColorTarget,
    color: &str,
    open: bool,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> crate::chrome::Button {
    div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .size(px(size::SWATCH_TRIGGER))
        .rounded(px(size::SWATCH_LARGE_RADIUS))
        .border_1()
        .border_color(if open {
            tokens.primary_darkest()
        } else {
            tokens.swatch_outline()
        })
        .bg(palette_fill(color, tokens))
        .on_click(Editor::act(cx, move |this, _, _, _| {
            this.toggle_color_popup(target)
        }))
}

/// The font-family icon upstream's `getFontFamilyIcon` pairs with each family.
fn font_icon(family: FontFamily) -> &'static str {
    match family {
        FontFamily::Excalifont | FontFamily::Virgil => "FreedrawIcon",
        FontFamily::Lilita => "FontFamilyHeadingIcon",
        FontFamily::ComicShanns | FontFamily::Cascadia => "FontFamilyCodeIcon",
        FontFamily::Nunito | FontFamily::Helvetica => "FontFamilyNormalIcon",
        FontFamily::Assistant | FontFamily::Liberation | FontFamily::Xiaolai => {
            "FontFamilyNormalIcon"
        }
    }
}

/// Upstream's `getFontFamilyLabel`: the `FONT_FAMILY` key itself, so these names
/// are deliberately not localised — they are family names, not UI copy.
pub(crate) fn font_label(family: FontFamily) -> &'static str {
    match family {
        FontFamily::Excalifont => "Excalifont",
        FontFamily::Nunito => "Nunito",
        FontFamily::ComicShanns => "Comic Shanns",
        FontFamily::Virgil => "Virgil",
        FontFamily::Helvetica => "Helvetica",
        FontFamily::Cascadia => "Cascadia",
        FontFamily::Assistant => "Assistant",
        FontFamily::Liberation => "Liberation",
        FontFamily::Lilita => "Lilita One",
        FontFamily::Xiaolai => "Xiaolai",
    }
}

/// `DEFAULT_FONTS` — the three families pinned above the panel's font trigger.
pub(crate) const DEFAULT_FONTS: [FontFamily; 3] = [
    FontFamily::Excalifont,
    FontFamily::Nunito,
    FontFamily::ComicShanns,
];

/// Every family the picker can offer, sorted the way upstream sorts its list.
pub(crate) fn all_fonts() -> Vec<FontFamily> {
    let mut fonts = vec![
        FontFamily::Excalifont,
        FontFamily::Nunito,
        FontFamily::ComicShanns,
        FontFamily::Virgil,
        FontFamily::Helvetica,
        FontFamily::Cascadia,
        FontFamily::Assistant,
        FontFamily::Liberation,
        FontFamily::Lilita,
        FontFamily::Xiaolai,
    ];
    fonts.sort_by_key(|f| font_label(*f).to_lowercase());
    fonts
}

/// The three favourite families plus the trigger that opens the full list.
fn font_picker_row(
    editor: &Editor,
    current: FontFamily,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> Div {
    let children: Vec<AnyElement> = DEFAULT_FONTS
        .iter()
        .map(|family| {
            let family = *family;
            let checked = family == current;
            let fg = if checked {
                tokens.selected_fg()
            } else {
                tokens.surface_text()
            };
            let base = div()
                .id(format!("font-top-{}", font_label(family)))
                .test_support()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .size(px(size::BUTTON))
                .rounded(px(size::RADIUS_MD))
                .border_1()
                .border_color(if checked {
                    tokens.selected_bg()
                } else {
                    tokens.border()
                })
                .child(icon(font_icon(family), size::ICON, fg));
            let base = if checked {
                base.bg(tokens.selected_bg())
            } else {
                base.hover(move |s| s.bg(tokens.hover()))
            };
            base.on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_font_family(family);
            }))
            .into_any_element()
        })
        .collect();

    let open = editor.is_font_popup_open();
    let fg = if open {
        tokens.selected_fg()
    } else {
        tokens.surface_text()
    };
    let trigger = div()
        .id("font-trigger")
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size::SWATCH_TRIGGER))
        .rounded(px(size::RADIUS_MD))
        .border_1()
        .border_color(if open {
            tokens.selected_bg()
        } else {
            tokens.border()
        })
        .child(icon("TextIcon", size::ICON, fg))
        .on_click(Editor::act(cx, |this, _, _, _| this.toggle_font_popup()));

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .py(px(4.0))
        .child(div().flex_1().child(row(children)))
        .child(button_separator(tokens))
        .child(trigger)
}

pub fn render_properties_panel(editor: &mut Editor, cx: &mut Context<Editor>) -> gpui_kit::Div {
    let cx = &*cx;
    let tokens = tokens_of(editor);
    let is_empty = editor.document.selected.is_empty();

    let mut panel = div()
        .w(px(size::PROPERTIES_PANEL_WIDTH))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .p(px(12.0))
        .overflow_hidden()
        .bg(tokens.island())
        .rounded(px(size::RADIUS_LG))
        .shadow(island_shadow());

    if is_empty {
        return panel;
    }

    let selection_count = editor.document.selected.len();
    let Some(element) = editor
        .document
        .selected_elements()
        .first()
        .map(|e| (*e).clone())
    else {
        return panel;
    };
    let base = element.base().clone();

    let stroke_color = base.stroke_color.clone();
    let background_color = base.background_color.clone();

    // ----- arrange: only meaningful with two or more elements selected ------
    if selection_count >= 2 {
        let aligns: [(excalidraw_core::operations::Align, &str, &str); 6] = [
            (
                excalidraw_core::operations::Align::Left,
                "AlignLeftIcon",
                "align-left",
            ),
            (
                excalidraw_core::operations::Align::CenterH,
                "CenterHorizontallyIcon",
                "align-center-h",
            ),
            (
                excalidraw_core::operations::Align::Right,
                "AlignRightIcon",
                "align-right",
            ),
            (
                excalidraw_core::operations::Align::Top,
                "AlignTopIcon",
                "align-top",
            ),
            (
                excalidraw_core::operations::Align::CenterV,
                "CenterVerticallyIcon",
                "align-center-v",
            ),
            (
                excalidraw_core::operations::Align::Bottom,
                "AlignBottomIcon",
                "align-bottom",
            ),
        ];
        let align_controls: Vec<AnyElement> = aligns
            .into_iter()
            .map(|(align, icon_name, id)| {
                ctrl(id, icon_name, false, tokens)
                    .on_click(Editor::act(cx, move |this, _, _, _| {
                        this.align_selected(align)
                    }))
                    .into_any_element()
            })
            .collect();

        let distribute_controls: Vec<AnyElement> = [
            (
                excalidraw_core::operations::Distribute::Horizontal,
                "DistributeHorizontallyIcon",
                "distribute-h",
            ),
            (
                excalidraw_core::operations::Distribute::Vertical,
                "DistributeVerticallyIcon",
                "distribute-v",
            ),
        ]
        .into_iter()
        .map(|(dir, icon_name, id)| {
            ctrl(id, icon_name, false, tokens)
                .on_click(Editor::act(cx, move |this, _, _, _| {
                    this.distribute_selected(dir)
                }))
                .into_any_element()
        })
        .collect();

        let group_control = ctrl("arrange-group", "GroupIcon", false, tokens)
            .on_click(Editor::act(cx, |this, _, _, _| this.group_selected()));
        let lock_control = ctrl("arrange-lock", "LockedIcon", false, tokens)
            .on_click(Editor::act(cx, |this, _, _, _| this.lock_selected(true)));

        panel = panel
            .child(row(align_controls))
            .child(row(distribute_controls))
            .child(row(vec![
                group_control.into_any_element(),
                lock_control.into_any_element(),
            ]))
            .child(rule(tokens));
    }

    // ----- colours ---------------------------------------------------------
    //
    // Upstream keeps the panel to two rows: five "top picks", a hairline, and the
    // active-colour trigger that opens the picker. Everything else — the 5×3
    // palette grid, the shade ramp and the hex field — lives in the popup, which
    // is what keeps the panel one screen tall whatever the palette holds.
    panel = panel
        .child(color_picker_row(
            editor,
            ColorTarget::Stroke,
            &stroke_color,
            tokens,
            cx,
        ))
        .child(color_picker_row(
            editor,
            ColorTarget::Background,
            &background_color,
            tokens,
            cx,
        ))
        .child(rule(tokens));

    // ----- fill style ------------------------------------------------------
    let fill_style = base.fill_style;
    let fill_controls: Vec<AnyElement> = [
        (FillStyle::Hachure, "FillHachureIcon", "fill-hachure"),
        (FillStyle::CrossHatch, "FillCrossHatchIcon", "fill-cross"),
        (FillStyle::Solid, "FillSolidIcon", "fill-solid"),
        (FillStyle::Zigzag, "FillZigZagIcon", "fill-zigzag"),
    ]
    .into_iter()
    .map(|(style, icon_name, id)| {
        ctrl(id, icon_name, fill_style == style, tokens)
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_fill_style(style);
            }))
            .into_any_element()
    })
    .collect();

    // ----- stroke width ----------------------------------------------------
    // Upstream offers three: thin, bold, extra bold.
    let stroke_width = base.stroke_width;
    let width_controls: Vec<AnyElement> = [
        (1.0f64, "StrokeWidthBaseIcon", "width-thin"),
        (2.0, "StrokeWidthBoldIcon", "width-bold"),
        (4.0, "StrokeWidthExtraBoldIcon", "width-extra"),
    ]
    .into_iter()
    .map(|(w, icon_name, id)| {
        ctrl(id, icon_name, (stroke_width - w).abs() < 0.01, tokens)
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_stroke_width(w);
            }))
            .into_any_element()
    })
    .collect();

    // ----- stroke style ----------------------------------------------------
    let stroke_style = base.stroke_style;
    let stroke_controls: Vec<AnyElement> = [
        (StrokeStyle::Solid, "StrokeStyleSolidIcon", "style-solid"),
        (StrokeStyle::Dashed, "StrokeStyleDashedIcon", "style-dashed"),
        (StrokeStyle::Dotted, "StrokeStyleDottedIcon", "style-dotted"),
    ]
    .into_iter()
    .map(|(style, icon_name, id)| {
        ctrl(id, icon_name, stroke_style == style, tokens)
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_stroke_style(style);
            }))
            .into_any_element()
    })
    .collect();

    // ----- sloppiness ------------------------------------------------------
    // 0 = architect (perfectly straight), 1 = artist, 2 = cartoonist.
    let roughness = base.roughness;
    let sloppiness_controls: Vec<AnyElement> = [
        (0.0f64, "SloppinessArchitectIcon", "sloppy-architect"),
        (1.0, "SloppinessArtistIcon", "sloppy-artist"),
        (2.0, "SloppinessCartoonistIcon", "sloppy-cartoonist"),
    ]
    .into_iter()
    .map(|(r, icon_name, id)| {
        ctrl(id, icon_name, (roughness - r).abs() < 0.01, tokens)
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_roughness(r);
            }))
            .into_any_element()
    })
    .collect();

    panel = panel
        .child(row(fill_controls))
        .child(row(width_controls))
        .child(row(stroke_controls))
        .child(row(sloppiness_controls));

    let is_text = matches!(element, excalidraw_core::element::Element::Text(_));
    let is_linear = matches!(
        element,
        excalidraw_core::element::Element::Line(_) | excalidraw_core::element::Element::Arrow(_)
    );

    // ----- edges (shapes only) ---------------------------------------------
    if !is_text && !is_linear {
        let roundness = base.roundness.and_then(|r| r.value);
        let sharp = ctrl("edge-sharp", "EdgeSharpIcon", roundness.is_none(), tokens).on_click(
            Editor::act(cx, |this, _, _, _| {
                this.document.set_selected_roundness(None);
            }),
        );
        let rounded = ctrl("edge-round", "EdgeRoundIcon", roundness.is_some(), tokens).on_click(
            Editor::act(cx, |this, _, _, _| {
                this.document.set_selected_roundness(Some(Roundness {
                    kind: RoundnessType::ProportionalRadius,
                    value: Some(0.25),
                }));
            }),
        );
        panel = panel
            .child(row(vec![
                sharp.into_any_element(),
                rounded.into_any_element(),
            ]))
            .child(rule(tokens));
    }

    // ----- arrowheads (linear only) ----------------------------------------
    if is_linear {
        let (cur_start, cur_end) = match &element {
            excalidraw_core::element::Element::Line(l) => {
                (l.linear.start_arrowhead, l.linear.end_arrowhead)
            }
            excalidraw_core::element::Element::Arrow(a) => {
                (a.linear.start_arrowhead, a.linear.end_arrowhead)
            }
            _ => (None, None),
        };
        let heads: [(Option<Arrowhead>, &str, &str); 6] = [
            (None, "ArrowheadNoneIcon", "ah-none"),
            (Some(Arrowhead::Arrow), "ArrowheadArrowIcon", "ah-arrow"),
            (Some(Arrowhead::Bar), "ArrowheadBarIcon", "ah-bar"),
            (Some(Arrowhead::Dot), "ArrowheadCircleIcon", "ah-dot"),
            (
                Some(Arrowhead::Triangle),
                "ArrowheadTriangleIcon",
                "ah-triangle",
            ),
            (
                Some(Arrowhead::Diamond),
                "ArrowheadDiamondIcon",
                "ah-diamond",
            ),
        ];

        let start_controls: Vec<AnyElement> = heads
            .iter()
            .map(|&(head, icon_name, id)| {
                ctrl(format!("start-{id}"), icon_name, cur_start == head, tokens)
                    .on_click(Editor::act(cx, move |this, _, _, _| {
                        this.push_checkpoint();
                        this.document.set_selected_arrowhead(false, head);
                    }))
                    .into_any_element()
            })
            .collect();
        let end_controls: Vec<AnyElement> = heads
            .iter()
            .map(|&(head, icon_name, id)| {
                ctrl(format!("end-{id}"), icon_name, cur_end == head, tokens)
                    .on_click(Editor::act(cx, move |this, _, _, _| {
                        this.push_checkpoint();
                        this.document.set_selected_arrowhead(true, head);
                    }))
                    .into_any_element()
            })
            .collect();

        panel = panel
            .child(row(start_controls))
            .child(row(end_controls))
            .child(rule(tokens));
    }

    // ----- typography (text only) ------------------------------------------
    if is_text {
        let (current_family, current_size, current_align) =
            if let excalidraw_core::element::Element::Text(t) = &element {
                (t.font_family, t.font_size, t.text_align)
            } else {
                (FontFamily::Virgil, 20.0, TextAlign::Left)
            };

        // Upstream's font row is the same shape as its colour rows: three
        // favourite families, then the trigger that opens the full list.
        let family_row = font_picker_row(editor, current_family, tokens, cx);

        let size_controls: Vec<AnyElement> = [
            (16.0f64, "FontSizeSmallIcon", "size-s"),
            (20.0, "FontSizeMediumIcon", "size-m"),
            (28.0, "FontSizeLargeIcon", "size-l"),
            (36.0, "FontSizeExtraLargeIcon", "size-xl"),
        ]
        .into_iter()
        .map(|(s, icon_name, id)| {
            ctrl(id, icon_name, (current_size - s).abs() < 0.01, tokens)
                .on_click(Editor::act(cx, move |this, _, _, _| {
                    this.document.set_selected_font_size(s);
                }))
                .into_any_element()
        })
        .collect();

        let align_controls: Vec<AnyElement> = [
            (TextAlign::Left, "TextAlignLeftIcon", "ta-left"),
            (TextAlign::Center, "TextAlignCenterIcon", "ta-center"),
            (TextAlign::Right, "TextAlignRightIcon", "ta-right"),
        ]
        .into_iter()
        .map(|(align, icon_name, id)| {
            ctrl(id, icon_name, current_align == align, tokens)
                .on_click(Editor::act(cx, move |this, _, _, _| {
                    this.document.set_selected_text_align(align);
                }))
                .into_any_element()
        })
        .collect();

        panel = panel
            .child(family_row)
            .child(row(size_controls))
            .child(row(align_controls))
            .child(rule(tokens));
    }

    // ----- opacity ---------------------------------------------------------
    let opacity = base.opacity;
    let opacity_controls: Vec<AnyElement> = [25.0f64, 50.0, 75.0, 100.0]
        .into_iter()
        .map(|o| {
            ctrl_text(
                format!("opacity-{}", o as i32),
                &format!("{}%", o as i32),
                (opacity - o).abs() < 0.01,
                tokens,
            )
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.document.set_selected_opacity(o);
            }))
            .into_any_element()
        })
        .collect();
    panel = panel.child(row(opacity_controls)).child(rule(tokens));

    // ----- layers ----------------------------------------------------------
    let layer_controls: Vec<AnyElement> = [
        ("layers-front", "BringToFrontIcon", LayerAction::Front),
        ("layers-forward", "BringForwardIcon", LayerAction::Forward),
        ("layers-backward", "SendBackwardIcon", LayerAction::Backward),
        ("layers-back", "SendToBackIcon", LayerAction::Back),
    ]
    .into_iter()
    .map(|(id, icon_name, action)| {
        ctrl(id, icon_name, false, tokens)
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.push_checkpoint();
                match action {
                    LayerAction::Front => this.document.bring_to_front(),
                    LayerAction::Forward => this.document.bring_forward(),
                    LayerAction::Backward => this.document.send_backward(),
                    LayerAction::Back => this.document.send_to_back(),
                }
            }))
            .into_any_element()
    })
    .collect();

    // ----- lock / duplicate / delete ---------------------------------------
    let locked = base.locked;
    let lock_control = ctrl(
        "single-lock",
        if locked { "LockedIcon" } else { "UnlockedIcon" },
        locked,
        tokens,
    )
    .on_click(Editor::act(cx, |this, _, _, _| {
        let locked = this
            .document
            .selected_elements()
            .first()
            .map(|e| e.base().locked)
            .unwrap_or(false);
        this.lock_selected(!locked);
    }));
    let duplicate_control = ctrl("single-dup", "DuplicateIcon", false, tokens)
        .on_click(Editor::act(cx, |this, _, _, _| this.duplicate_selected()));
    let delete_control = ctrl("single-del", "TrashIcon", false, tokens)
        .on_click(Editor::act(cx, |this, _, _, _| this.delete_selected()));
    // Upstream's `actionLink` is a toggle whose checked state is "this element
    // already has a link", and it is what opens the link dialog.
    let has_link = base.link.is_some();
    let link_control = ctrl("single-link", "LinkIcon", has_link, tokens)
        .on_click(Editor::act(cx, |this, _, _, _| this.open_link_dialog()));

    panel.child(row(layer_controls)).child(row(vec![
        link_control.into_any_element(),
        lock_control.into_any_element(),
        duplicate_control.into_any_element(),
        delete_control.into_any_element(),
    ]))
}

/// The `.color-picker` / `.FontPicker__container` popup surface: island fill,
/// island shadow, with the sections separated by upstream's `.75rem` gap.
fn picker_surface(tokens: Tokens, width: f32) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(size::PICKER_GAP))
        .py(px(4.0))
        .w(px(width))
        .bg(tokens.island())
        .rounded(px(4.0))
        .shadow(island_shadow())
        .occlude()
}

/// One popup section: a heading over its list, with no gap between them — the
/// popup's own gap separates sections, not a heading from its grid.
fn picker_section(heading: Option<Div>, body: Div) -> Div {
    let section = div().flex().flex_col();
    match heading {
        Some(heading) => section.child(heading).child(body),
        None => section.child(body),
    }
}

/// The colour-picker popup (upstream `Picker`): the most used custom colours,
/// the 5×3 palette grid, the shade ramp and the hex field.
///
/// The panel is deliberately left one screen tall by moving all of this here —
/// the palette cannot grow the panel no matter what the scene holds.
pub(crate) fn render_color_picker(editor: &Editor, cx: &Context<Editor>) -> Div {
    let Some(target) = editor.color_popup() else {
        return div();
    };
    let tokens = tokens_of(editor);
    let t = |key: &str| editor.i18n.t(key);

    // Upstream's canvas-background picker is the one `ColorPicker` call that
    // passes `palette={null}`, so its popup is nothing but the hex field — no
    // palette grid, no shade ramp, no custom colours. The field's own heading is
    // rendered here because the other popups rely on the section headings above.
    if target == ColorTarget::CanvasBackground {
        return picker_surface(tokens, size::PICKER_WIDTH)
            .child(picker_heading(&t("colorPicker.hexCode"), tokens))
            .child(hex_row(editor, target, tokens, cx));
    }

    let color = editor.color_popup_color(target);
    let shade = editor.active_shade(target);
    let palette = crate::theme::element_palette();
    let active_name = crate::theme::color_position(&color).map(|(name, _)| name);

    let mut surface = picker_surface(tokens, size::PICKER_WIDTH);

    // ----- most used custom colours ----------------------------------------
    let custom = editor.custom_colors(target);
    if !custom.is_empty() {
        let tiles: Vec<AnyElement> = custom
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let value = value.clone();
                let active = value == color;
                let pick = value.clone();
                large_swatch(
                    format!("color-custom-{index}"),
                    &value,
                    active,
                    true,
                    tokens,
                )
                .on_click(Editor::act(cx, move |this, _, _, _| {
                    this.apply_color(target, &pick);
                }))
                .into_any_element()
            })
            .collect();
        surface = surface.child(picker_section(
            Some(picker_heading(
                &t("colorPicker.mostUsedCustomColors"),
                tokens,
            )),
            swatch_grid(tiles),
        ));
    }

    // ----- the palette grid -------------------------------------------------
    let tiles: Vec<AnyElement> = palette
        .iter()
        .map(|entry| {
            let value = entry.at(shade);
            let active = active_name == Some(entry.name);
            large_swatch(
                format!("color-{}", entry.name),
                value,
                active,
                needs_outline(value),
                tokens,
            )
            .on_click(Editor::act(cx, move |this, _, _, _| {
                this.apply_color(target, value);
            }))
            .into_any_element()
        })
        .collect();
    surface = surface.child(picker_section(
        Some(picker_heading(&t("colorPicker.colors"), tokens)),
        swatch_grid(tiles),
    ));

    // ----- the shade ramp ---------------------------------------------------
    //
    // Only a ramped entry has shades to show; a single-colour entry or a custom
    // colour gets upstream's centred "no shades" line instead of an empty grid.
    let ramp = active_name
        .and_then(|name| palette.iter().find(|entry| entry.name == name))
        .filter(|entry| entry.is_ramp());
    let shade_body = match ramp {
        Some(entry) => {
            let tiles: Vec<AnyElement> = entry
                .shades
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let value = *value;
                    large_swatch(
                        format!("shade-{index}"),
                        value,
                        index == shade,
                        needs_outline(value),
                        tokens,
                    )
                    .on_click(Editor::act(cx, move |this, _, _, _| {
                        this.set_shade(target, index);
                        this.apply_color(target, value);
                    }))
                    .into_any_element()
                })
                .collect();
            swatch_grid(tiles)
        }
        None => div()
            .flex()
            .items_center()
            .justify_center()
            .px(px(size::PICKER_GRID_PAD))
            .py(px(16.0))
            .text_size(px(12.0))
            .text_color(tokens.surface_text())
            .child(SharedString::from(t("colorPicker.noShades"))),
    };
    surface = surface.child(picker_section(
        Some(picker_heading(&t("colorPicker.shades"), tokens)),
        shade_body,
    ));

    // ----- the hex field, with the eye dropper inside it --------------------
    surface = surface.child(hex_row(editor, target, tokens, cx));

    surface
}

/// `.color-picker__input-label` — `#`, the hex buffer, a hairline, and the eye
/// dropper. Upstream puts the picker's eye dropper *here*, not in the panel row.
fn hex_row(editor: &Editor, target: ColorTarget, tokens: Tokens, cx: &Context<Editor>) -> Div {
    let t = |key: &str| editor.i18n.t(key);
    let buffer = editor.hex_buffer().to_string();
    let placeholder = buffer.is_empty();
    let text = if placeholder {
        t("colorPicker.hexCode")
    } else {
        buffer
    };
    let field_color = if placeholder {
        tokens.shortcut_text()
    } else {
        tokens.surface_text()
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .mx(px(size::PICKER_GRID_PAD))
        .px(px(size::PICKER_INPUT_PAD))
        .h(px(size::BUTTON))
        .border_1()
        .border_color(if editor.is_hex_focused() {
            tokens.primary_darkest()
        } else {
            tokens.border()
        })
        .rounded(px(size::PICKER_INPUT_RADIUS))
        .text_size(px(14.0))
        .child(
            div()
                .flex_shrink_0()
                .text_color(tokens.input_label())
                .child(SharedString::from("#")),
        )
        .child(
            div()
                .id("color-hex-field")
                .test_support()
                .flex_1()
                .text_color(field_color)
                .child(SharedString::from(text))
                .on_click(Editor::act(cx, move |this, _, _, _| {
                    this.set_hex_focused(true)
                })),
        )
        .child(
            div()
                .flex_shrink_0()
                .w(px(1.0))
                .h(px(20.0))
                .bg(tokens.border()),
        )
        .child(eye_dropper_trigger(
            if stroke_target(target) {
                "eye-dropper-stroke"
            } else {
                "eye-dropper-bg"
            },
            target,
            editor.eye_dropper_target() == Some(target),
            tokens,
            cx,
        ))
}

/// One family row of the font picker (`.dropdown-item`).
fn font_row(
    family: FontFamily,
    current: Option<FontFamily>,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> AnyElement {
    let selected = current == Some(family);
    let fg = if selected {
        tokens.selected_fg()
    } else {
        tokens.surface_text()
    };
    let row = div()
        .id(format!("font-item-{}", font_label(family)))
        .test_support()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(size::MENU_ROW_GAP))
        .w_full()
        .min_h(px(size::font_picker::ROW_HEIGHT))
        .px(px(8.0))
        .rounded(px(size::RADIUS_MD))
        .text_size(px(14.0))
        .text_color(fg)
        .child(icon(font_icon(family), size::ICON, fg))
        .child(div().flex_1().child(SharedString::from(font_label(family))));
    let row = if selected {
        row.bg(tokens.selected_bg())
    } else {
        row.hover(move |s| s.bg(tokens.hover()))
    };
    row.on_click(Editor::act(cx, move |this, _, _, _| {
        this.document.set_selected_font_family(family);
    }))
    .into_any_element()
}

/// The font-picker popup: quick search over the scene's families and the rest
/// (upstream `FontPickerList`).
pub(crate) fn render_font_picker(editor: &Editor, cx: &Context<Editor>) -> Div {
    let tokens = tokens_of(editor);
    let t = |key: &str| editor.i18n.t(key);
    let current = editor
        .document
        .selected_elements()
        .first()
        .map(|element| match element {
            excalidraw_core::element::Element::Text(text) => text.font_family,
            _ => FontFamily::Virgil,
        });
    let (scene, available) = editor.picker_font_groups();

    let mut surface = picker_surface(tokens, size::font_picker::WIDTH);

    // ----- quick search -----------------------------------------------------
    let query = editor.font_search().to_string();
    let placeholder = query.is_empty();
    surface = surface.child(
        div()
            .id("font-search")
            .test_support()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .mx(px(size::font_picker::PAD))
            .px(px(8.0))
            .h(px(size::BUTTON))
            .border_1()
            .border_color(tokens.border())
            .rounded(px(size::RADIUS_MD))
            .text_size(px(14.0))
            .text_color(if placeholder {
                tokens.shortcut_text()
            } else {
                tokens.surface_text()
            })
            .child(icon("searchIcon", size::ICON, tokens.shortcut_text()))
            .child(div().flex_1().child(SharedString::from(if placeholder {
                t("quickSearch.placeholder")
            } else {
                query
            }))),
    );

    // ----- the two groups ---------------------------------------------------
    if scene.is_empty() && available.is_empty() {
        surface = surface.child(
            div()
                .px(px(size::font_picker::PAD))
                .py(px(8.0))
                .text_size(px(13.0))
                .text_color(tokens.shortcut_text())
                .child(SharedString::from(t("fontList.empty"))),
        );
    } else {
        let groups: [(&str, &Vec<FontFamily>); 2] = [
            ("fontList.sceneFonts", &scene),
            ("fontList.availableFonts", &available),
        ];
        for (title, fonts) in groups {
            if fonts.is_empty() {
                continue;
            }
            let rows: Vec<AnyElement> = fonts
                .iter()
                .map(|family| font_row(*family, current, tokens, cx))
                .collect();
            surface = surface.child(picker_heading(&t(title), tokens)).child(
                div()
                    .flex()
                    .flex_col()
                    .p(px(size::font_picker::LIST_PAD))
                    .children(rows),
            );
        }
    }

    surface
}

/// One `.DialogActionButton`.
fn dialog_button(
    id: &'static str,
    label: &str,
    primary: bool,
    tokens: Tokens,
    cx: &Context<Editor>,
) -> crate::chrome::Button {
    let fill = if primary {
        tokens.accent()
    } else {
        tokens.island()
    };
    let fg = if primary {
        rgb(0xffffff)
    } else {
        tokens.surface_text()
    };
    div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .h(px(size::BUTTON))
        .px(px(12.0))
        .rounded(px(size::RADIUS_LG))
        .border_1()
        .border_color(if primary { fill } else { tokens.border() })
        .bg(fill)
        .text_size(px(14.0))
        .text_color(fg)
        .child(SharedString::from(label.to_owned()))
        .on_click(Editor::act(cx, move |this, _, _, _| {
            if primary {
                this.confirm_link();
            } else {
                this.cancel_link_dialog();
            }
        }))
}

/// The element-link dialog (upstream `ElementLinkDialog`): a header, the link
/// field with its remove button, and the cancel/confirm pair.
pub(crate) fn render_link_dialog(editor: &Editor, cx: &Context<Editor>) -> Div {
    let tokens = tokens_of(editor);
    let t = |key: &str| editor.i18n.t(key);
    let has_link = editor.selected_link().is_some();
    let field = editor.link_input().to_string();

    let mut input_row = div().flex().flex_row().items_center().w_full().child(
        div()
            .id("link-input")
            .test_support()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .h(px(size::BUTTON))
            .px(px(8.0))
            .border_1()
            .border_color(tokens.border())
            .rounded(px(size::RADIUS_LG))
            .text_size(px(14.0))
            .text_color(tokens.surface_text())
            .child(SharedString::from(field.clone())),
    );
    // Upstream only offers the remove button while the element *has* a link —
    // it empties the field, and the confirm is what clears the element.
    if has_link && !field.is_empty() {
        input_row = input_row.child(
            div()
                .id("link-remove")
                .test_support()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .ml(px(16.0))
                .size(px(size::BUTTON))
                .rounded(px(size::RADIUS_MD))
                .hover(move |s| s.bg(tokens.hover()))
                .child(icon("TrashIcon", size::ICON, rgb(0xff6b6b)))
                .on_click(Editor::act(cx, |this, _, _, _| this.clear_link_input())),
        );
    }

    div()
        .flex()
        .flex_col()
        .w(px(size::link_dialog::WIDTH))
        .p(px(size::link_dialog::PADDING))
        .bg(tokens.island())
        .rounded(px(size::link_dialog::RADIUS))
        .shadow(island_shadow())
        .occlude()
        .child(
            div()
                .flex()
                .flex_col()
                .mb(px(size::link_dialog::PADDING))
                .child(
                    div()
                        .text_size(px(20.0))
                        .text_color(tokens.surface_text())
                        .child(SharedString::from(t("elementLink.title"))),
                )
                .child(
                    div()
                        .mt(px(8.0))
                        .text_size(px(14.0))
                        .text_color(tokens.surface_text())
                        .child(SharedString::from(t("elementLink.desc"))),
                ),
        )
        .child(input_row)
        .child(
            div()
                .flex()
                .flex_row()
                .justify_end()
                .gap(px(10.0))
                .mt(px(size::link_dialog::PADDING))
                .child(dialog_button(
                    "link-cancel",
                    &t("buttons.cancel"),
                    false,
                    tokens,
                    cx,
                ))
                .child(dialog_button(
                    "link-confirm",
                    &t("buttons.confirm"),
                    true,
                    tokens,
                    cx,
                )),
        )
}

/// Which end of the stacking order a layer button moves the selection to.
#[derive(Clone, Copy)]
enum LayerAction {
    Front,
    Forward,
    Backward,
    Back,
}
