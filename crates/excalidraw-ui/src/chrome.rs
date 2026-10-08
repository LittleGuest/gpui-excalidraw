//! The floating chrome: islands, tool buttons, dividers, dropdown rows.
//!
//! Excalidraw draws almost no full-width bars. The canvas runs edge to edge and
//! every control floats above it as an **island** — a rounded surface lifted by
//! `--shadow-island`. This module owns those primitives so [`crate::editor`] can
//! describe *what goes where* without re-deriving surfaces, sizes and icon
//! tinting at every call site.
//!
//! Every size and colour comes from [`crate::design`], which is a direct port of
//! upstream `theme.scss`. Nothing here invents a value; if a number looks
//! arbitrary it is because it is a `rem` from the original stylesheet.

use gpui_kit::{
    base::{ObservedElement, TestSupportExt},
    *,
};

use crate::{
    design::{Tokens, chip_ring_shadow, island_shadow, size},
    icons::icon,
};

/// The type a chrome button has after [`TestSupportExt::test_support`].
///
/// With the `test-support` feature this is GPUI Base's observation wrapper, so
/// integration tests can find the button by id and click it for real — which is
/// the only way to prove the islands do not swallow their own clicks. Without
/// the feature the alias collapses to the plain element, so production pays
/// nothing.
pub(crate) type Button = ObservedElement<Stateful<Div>>;

/// An island: the rounded, elevated surface everything else sits on.
pub fn island(tokens: Tokens) -> Div {
    div()
        .bg(tokens.island())
        .rounded(px(size::RADIUS_LG))
        .shadow(island_shadow())
}

/// The inner row of the toolbar island (`.App-toolbar`).
pub fn toolbar_island(tokens: Tokens) -> Div {
    island(tokens)
        .flex()
        .flex_row()
        .items_center()
        .p(px(size::TOOLBAR_PADDING))
        .gap(px(size::TOOLBAR_GAP))
}

/// A compact chrome chip — the hamburger, the zoom controls, undo/redo, the
/// help button.
///
/// These are *not* toolbar islands, and getting that wrong is very visible:
/// measured on excalidraw.com, every one of them is a flat
/// `--color-surface-low` fill wearing a `0 0 0 1px --color-surface-lowest`
/// ring, with **no padding at all** — the 36px buttons inside are the full
/// height of the chip. The toolbar is the only white, drop-shadowed island.
pub fn rail_island(tokens: Tokens) -> Div {
    div()
        .bg(tokens.surface_low())
        .rounded(px(size::RADIUS_LG))
        .shadow(chip_ring_shadow(tokens))
        .flex()
        .flex_row()
        .items_center()
}

/// A vertical hairline between tool groups (`.App-toolbar__divider`).
///
/// Upstream gives this 4px of margin on one side only — the row's own 4px gap
/// supplies the other side — so the rule sits 4px from the group it follows and
/// 8px from the group it precedes. Measured off excalidraw.com: the first
/// divider line lands at x+40 from the padlock, the second 8px before the
/// dots. `margin_right` picks which side carries the extra margin.
pub fn toolbar_divider(tokens: Tokens, margin_right: bool) -> Div {
    let rule = div()
        .flex_shrink_0()
        .w(px(1.0))
        .h(px(24.0))
        .bg(tokens.separator());
    if margin_right {
        rule.mr(px(4.0))
    } else {
        rule.ml(px(4.0))
    }
}

/// A `<kbd>` chip — the little outlined key name Excalidraw draws inside the
/// "to move canvas…" hint (`.HintViewer kbd`). Transparent fill, 1px border in
/// the hint colour, 4px radius, monospace at 10px.
pub fn kbd_chip(text: impl Into<SharedString>, tokens: Tokens) -> Div {
    div()
        .flex_shrink_0()
        .mx(px(1.0))
        .px(px(size::KBD_PAD_X))
        .py(px(size::KBD_PAD_Y))
        .rounded(px(size::KBD_RADIUS))
        .border_1()
        .border_color(tokens.shortcut_text())
        .font_family("monospace")
        .text_size(px(size::KBD_FONT))
        .text_color(tokens.shortcut_text())
        .child(text.into())
}

/// The shortcut letter drawn into a tool button's lower-right corner
/// (`.ToolIcon__keybinding`).
///
/// Upstream pins it with `top: 21px; right: 3px` rather than to the bottom
/// edge, and paints the selected state in `--color-on-primary-container`
/// (`rgb(3, 0, 100)`), not a faded version of it.
fn keybinding_badge(text: &str, checked: bool, tokens: Tokens) -> Div {
    let color = if checked {
        tokens.selected_fg()
    } else {
        tokens.shortcut_text()
    };
    div()
        .absolute()
        .top(px(size::KEYBINDING_TOP))
        .right(px(size::KEYBINDING_RIGHT))
        .text_size(px(size::KEYBINDING_FONT))
        .text_color(color)
        .child(SharedString::from(text.to_owned()))
}

/// A 36px square button holding a tinted icon (`.ToolIcon`).
///
/// `badge` is the shortcut letter shown in the corner; pass `None` for chrome
/// that has no key binding (the hamburger, the theme toggle). `checked` paints
/// the `--color-primary-light` container, which is how Excalidraw marks the
/// active tool.
pub fn tool_button(
    id: impl Into<ElementId>,
    icon_name: &str,
    badge: Option<&str>,
    checked: bool,
    tokens: Tokens,
) -> Button {
    let fg = if checked {
        tokens.selected_fg()
    } else {
        tokens.surface_text()
    };
    let base = div()
        .id(id)
        .test_support()
        .relative()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size::BUTTON_LG))
        .rounded(px(size::RADIUS_LG))
        .text_color(fg)
        .child(icon(icon_name, size::ICON, fg));
    let base = if checked {
        base.bg(tokens.selected_bg())
    } else {
        base.hover(move |s| s.bg(tokens.hover()))
    };
    match badge {
        Some(text) => base.child(keybinding_badge(text, checked, tokens)),
        None => base,
    }
}

/// A 32px icon-only button for the footer and the corner rails, where upstream
/// swaps `--lg-button-size` for `--default-button-size`.
///
/// `enabled` dims the glyph rather than removing the button, mirroring how
/// Excalidraw fades undo/redo when a history stack is empty.
pub fn chrome_button(
    id: impl Into<ElementId>,
    icon_name: &str,
    enabled: bool,
    tokens: Tokens,
) -> Button {
    let fg = if enabled {
        tokens.surface_text()
    } else {
        tokens.surface_text().alpha(0.35)
    };
    div()
        .id(id)
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size::BUTTON_LG))
        .rounded(px(size::RADIUS_LG))
        .text_color(fg)
        .child(icon(icon_name, size::ICON, fg))
        .hover(move |s| s.bg(tokens.hover()))
}

/// The zoom readout between the two zoom buttons (`.zoom-value`).
///
/// Upstream: a 60×36 flex child with `padding: 0 10px` and 16px text. It has no
/// fill of its own — the chip behind it supplies that.
pub fn zoom_readout(tokens: Tokens) -> Div {
    div()
        .flex_shrink_0()
        .h(px(size::BUTTON_LG))
        .min_w(px(60.0))
        .px(px(10.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(16.0))
        .text_color(tokens.surface_text())
}

/// A row inside a dropdown (`.dropdown-item`): icon, label, optional shortcut.
///
/// Rows without an icon still reserve the icon column, so labels keep their
/// left edge down the whole menu — that ragged edge is the tell of a menu that
/// was assembled rather than laid out.
pub fn menu_row(
    id: impl Into<ElementId>,
    icon_name: Option<&str>,
    label: &str,
    tokens: Tokens,
) -> Button {
    menu_row_with(id, icon_name, label, None, false, tokens)
}

/// A dropdown row with the full Excalidraw anatomy: glyph, label, a key hint
/// pinned to the right edge, and a check mark for toggles.
pub fn menu_row_with(
    id: impl Into<ElementId>,
    icon_name: Option<&str>,
    label: &str,
    shortcut: Option<&str>,
    checked: bool,
    tokens: Tokens,
) -> Button {
    menu_row_colored(
        id,
        icon_name,
        label,
        shortcut,
        checked,
        tokens,
        tokens.surface_text(),
    )
}

/// Same anatomy, but the label/leading glyph take an explicit colour — the
/// promo rows (Command palette, Sign up) print in the primary purple.
pub fn menu_row_colored(
    id: impl Into<ElementId>,
    icon_name: Option<&str>,
    label: &str,
    shortcut: Option<&str>,
    checked: bool,
    tokens: Tokens,
    fg: Rgba,
) -> Button {
    let row = div()
        .id(id)
        .test_support()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .w_full()
        .min_h(px(size::MENU_ROW))
        .px(px(8.0))
        .rounded(px(size::RADIUS_MD))
        .text_size(px(14.0))
        .text_color(fg)
        .hover(move |s| s.bg(tokens.hover()));
    // The leading slot holds the row's glyph — and for a toggle that glyph *is*
    // the check. Upstream's `DropdownMenuItemCheckbox` renders
    // `icon={checked ? checkIcon : emptyIcon}` in this same slot, so the label
    // starts at the same x whether or not the row is a toggle, and the check
    // never drifts towards the shortcut on the right.
    let row = match icon_name {
        Some(name) => row.child(icon(name, size::ICON, fg)),
        None if checked => row.child(icon("checkIcon", size::ICON, fg)),
        None => row.child(div().flex_shrink_0().w(px(size::ICON))),
    };
    let row = row.child(div().flex_1().child(SharedString::from(label.to_owned())));
    match shortcut {
        Some(keys) => row.child(
            div()
                .flex_shrink_0()
                .text_size(px(12.0))
                .text_color(tokens.shortcut_text())
                .child(SharedString::from(keys.to_owned())),
        ),
        None => row,
    }
}

/// A dropdown surface (`.dropdown`): island background plus menu shadow.
pub fn dropdown(tokens: Tokens) -> Div {
    div()
        .absolute()
        .flex()
        .flex_col()
        .p(px(8.0))
        .gap(px(2.0))
        .min_w(px(264.0))
        .occlude()
        .bg(tokens.island())
        .rounded(px(size::RADIUS_LG))
        .shadow(island_shadow())
}
