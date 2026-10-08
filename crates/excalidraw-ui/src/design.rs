//! Excalidraw's design tokens, ported from upstream `theme.scss`.
//!
//! These are **copied, not invented**. Every value below traces to a CSS custom
//! property in `packages/excalidraw/css/variables.scss` / `theme.scss` in the
//! Excalidraw repository, so the native editor's chrome reads as the same
//! product rather than an approximation of it.
//!
//! Two rules keep it honest:
//!
//! * **Islands.** Almost all chrome sits on an "island" — a rounded surface with
//!   the `--shadow-island` elevation. Isolating that here means a new button
//!   gets the right surface for free instead of re-deriving colours per widget.
//! * **Sizes come in rem-derived pixels.** Upstream sizes everything in `rem`
//!   against a 16px root, so `2.25rem` is `36.0` here. Keep the arithmetic
//!   visible; don't round to taste.

use gpui_kit::*;

/// Which palette a [`Tokens`] instance carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Palette {
    #[default]
    Light,
    Dark,
}

impl Palette {
    pub fn toggled(self) -> Self {
        match self {
            Palette::Light => Palette::Dark,
            Palette::Dark => Palette::Light,
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Palette::Dark)
    }
}

/// A complete set of resolved colours for one theme.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub palette: Palette,
    /// `--default-bg-color` — the canvas backdrop behind the grid.
    pub canvas_bg: u32,
    /// `--island-bg-color` — toolbar, menus, panels.
    pub island_bg: u32,
    /// `--color-on-surface` — primary text and icon colour.
    pub on_surface: u32,
    /// `--color-primary`.
    pub primary: u32,
    /// `--color-primary-hover`.
    pub primary_hover: u32,
    /// `--color-primary-light` — the fill behind a checked tool button.
    pub primary_container: u32,
    /// `--color-on-primary-container` — text/icon on `primary_container`.
    pub on_primary_container: u32,
    /// `--button-hover-bg`.
    pub button_hover: u32,
    /// `--keybinding-color` — the shortcut letter in a tool button's corner.
    pub keybinding: u32,
    /// `--color-surface-high`.
    ///
    /// Upstream uses this one colour for every chrome hairline: the island
    /// border, the `.App-toolbar__divider` and the menu separator all measure
    /// `rgb(241, 240, 255)` on excalidraw.com. It is *not* a mid grey — a
    /// darker line reads as a heavy rule next to upstream's near-invisible one.
    pub surface_high: u32,
    /// `--color-surface-low` — the fill of a chrome chip (hamburger, zoom,
    /// undo/redo, help). Light `#ececf4`, dark `#232329`.
    pub surface_low: u32,
    /// `--color-surface-lowest` — the 1px *ring* drawn around a chrome chip in
    /// place of a drop shadow. Light `#ffffff`, dark `#121212`.
    pub surface_lowest: u32,
    /// `--color-primary-darkest` — the ring around the selected colour swatch.
    /// Light `#4a47b1`, dark `#beb9ff`.
    pub primary_darkest: u32,
    /// `--color-primary-light-darker` — the file-drop card's border and the
    /// receding "back" sheet of its illustration. Light `#d7d5ff`, dark
    /// `#43415e`.
    pub primary_light_darker: u32,
    /// `--color-outline` (aliased to `--color-gray-20` / `--color-gray-80`) —
    /// the hairline around a colour-picker swatch that would otherwise vanish
    /// against the popup, e.g. white or a near-white pastel.
    pub swatch_outline: u32,
    /// `--input-label-color` — the `#` prefix of the hex field. Light gray-7
    /// `#495057`, dark gray-2 `#e9ecef`.
    pub input_label: u32,
    /// The canvas grid line colour.
    pub grid: u32,
}

/// Light theme. Mirrors the `:root` block of upstream `theme.scss`.
pub const LIGHT: Tokens = Tokens {
    palette: Palette::Light,
    canvas_bg: 0xffffff,
    island_bg: 0xffffff,
    surface_high: 0xf1f0ff, // --color-surface-high
    on_surface: 0x1b1b1f,
    primary: 0x6965db,
    primary_hover: 0x5753d0,
    primary_container: 0xe3e2fe, // --color-primary-light
    on_primary_container: 0x030064,
    button_hover: 0xf1f0ff,   // --button-hover-bg -> --color-surface-high
    keybinding: 0xb8b8b8,     // --color-gray-40
    surface_low: 0xececf4,    // --color-surface-low
    surface_lowest: 0xffffff, // --color-surface-lowest
    primary_darkest: 0x4a47b1,
    primary_light_darker: 0xd7d5ff,
    swatch_outline: 0xebebeb, // --color-gray-20
    input_label: 0x495057,    // --color-gray-7
    grid: 0xe9ecef,
};

/// Dark theme. Mirrors the `.theme--dark` block of upstream `theme.scss`.
pub const DARK: Tokens = Tokens {
    palette: Palette::Dark,
    canvas_bg: 0x121212,
    island_bg: 0x232329,
    surface_high: 0x2e2d39, // --color-surface-high
    on_surface: 0xe3e3e8,
    primary: 0xa8a5ff,
    primary_hover: 0xbbb8ff,
    primary_container: 0x4f4d6f, // --color-primary-light
    on_primary_container: 0xdedcff,
    button_hover: 0x2e2d39,
    keybinding: 0x7a7a7a, // --color-gray-60
    surface_low: 0x232329,
    surface_lowest: 0x121212,
    primary_darkest: 0xbeb9ff,
    primary_light_darker: 0x43415e,
    swatch_outline: 0x3d3d3d, // --color-gray-80
    input_label: 0xe9ecef,    // --color-gray-2
    grid: 0x232329,
};

impl Tokens {
    pub fn for_palette(palette: Palette) -> Self {
        match palette {
            Palette::Light => LIGHT,
            Palette::Dark => DARK,
        }
    }

    /// `--island-bg-color` as a GPUI colour.
    pub fn island(&self) -> Rgba {
        rgb(self.island_bg)
    }

    /// `--color-surface-high` — the chrome hairline and island border colour.
    pub fn surface_high(&self) -> Rgba {
        rgb(self.surface_high)
    }

    /// `--color-surface-high`, used as the island border colour.
    pub fn island_border(&self) -> Rgba {
        rgb(self.surface_high)
    }

    /// `--color-surface-low` — the fill behind a chrome chip.
    pub fn surface_low(&self) -> Rgba {
        rgb(self.surface_low)
    }

    /// `--color-gray-60` — the unlock bubble's glyph tint.
    ///
    /// Upstream declares the gray ramp once, outside both theme blocks, so this
    /// is the same `#7a7a7a` in light and dark; the *keybinding* colour is the
    /// one that flips per theme.
    pub fn gray_60(&self) -> Rgba {
        rgb(0x7a7a7a)
    }

    /// `--color-primary-darkest` — the ring around the selected colour swatch.
    pub fn primary_darkest(&self) -> Rgba {
        rgb(self.primary_darkest)
    }

    /// `--color-primary-light-darker` — the file-drop card's border.
    pub fn primary_light_darker(&self) -> Rgba {
        rgb(self.primary_light_darker)
    }

    /// `--color-outline` — the ring around a pale colour-picker swatch.
    pub fn swatch_outline(&self) -> Rgba {
        rgb(self.swatch_outline)
    }

    /// `--input-label-color` — the hex field's `#`.
    pub fn input_label(&self) -> Rgba {
        rgb(self.input_label)
    }

    /// `--input-border-color` — the hairline around a text input. Light
    /// `#ced4da` (`$color-gray-4`), dark `#2e2e2e`.
    ///
    /// Not a design token of its own: `styles.scss` gives every
    /// `input[type="text"]` this border, so the file-name field in the export
    /// dialog carries it too.
    pub fn input_border(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgb(0xced4da),
            Palette::Dark => rgb(0x2e2e2e),
        }
    }

    /// `--color-brand-hover` — the border of a hovered or focused text input.
    /// Aliased to `--color-primary-hover`, so it is the same purple as
    /// [`Tokens::hover`]'s sibling.
    pub fn brand_hover(&self) -> Rgba {
        rgb(self.primary_hover)
    }

    /// `--dialog-border-color` — the modal's outline and the `Dialog__title`'s
    /// underline. Aliased to `--color-gray-20` light / `--color-gray-80` dark,
    /// which is why it coincides with [`Tokens::swatch_outline`].
    pub fn dialog_border(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgb(0xebebeb),
            Palette::Dark => rgb(0x3d3d3d),
        }
    }

    /// `--overlay-bg-color` — the dimming wash behind the file-drop card.
    ///
    /// The only transparent token in the set, so it is written as
    /// `0xRRGGBBAA` rather than reusing the opaque `rgb` helper.
    pub fn overlay_backdrop(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgba(0x0c0c0e59), // rgba(12, 12, 14, .35)
            Palette::Dark => rgba(0x0c0c0ea6),  // rgba(12, 12, 14, .65)
        }
    }

    /// `.CursorHint`'s fill. Upstream pins this to the *dark-theme* island
    /// colour in both themes so the hint keeps high contrast over the mostly
    /// light canvas.
    pub fn cursor_hint_bg(&self) -> Rgba {
        rgb(0x232329)
    }

    /// `.CursorHint`'s glyph and text colour — dark-theme `--color-on-surface`.
    pub fn cursor_hint_fg(&self) -> Rgba {
        rgb(0xe3e3e8)
    }

    /// `--color-surface-lowest` — the ring drawn around a chrome chip.
    pub fn surface_lowest(&self) -> Rgba {
        rgb(self.surface_lowest)
    }

    pub fn surface_text(&self) -> Rgba {
        rgb(self.on_surface)
    }

    /// `--default-bg-color`'s border — the same `--color-surface-high` hairline.
    pub fn border(&self) -> Rgba {
        self.island_border()
    }

    pub fn accent(&self) -> Rgba {
        rgb(self.primary)
    }

    pub fn hover(&self) -> Rgba {
        rgb(self.button_hover)
    }

    /// Background of a *selected* tool button (`--color-primary-light`).
    pub fn selected_bg(&self) -> Rgba {
        rgb(self.primary_container)
    }

    /// Icon colour on a selected tool button.
    pub fn selected_fg(&self) -> Rgba {
        rgb(self.on_primary_container)
    }

    pub fn shortcut_text(&self) -> Rgba {
        rgb(self.keybinding)
    }

    pub fn separator(&self) -> Rgba {
        rgb(self.surface_high)
    }

    pub fn canvas(&self) -> Rgba {
        rgb(self.canvas_bg)
    }

    pub fn grid_line(&self) -> Rgba {
        rgb(self.grid)
    }
}

/// Spacing and sizing, in pixels, converted from upstream `rem` values.
pub mod size {
    /// `--editor-container-padding: 1rem`.
    pub const CONTAINER_PADDING: f32 = 16.0;
    /// `--lg-button-size: 2.25rem` — toolbar and chrome buttons.
    pub const BUTTON_LG: f32 = 36.0;
    /// `--default-button-size: 2rem` — compact buttons, footer controls.
    pub const BUTTON: f32 = 32.0;
    /// The glyph inside any 36px button — tool, hamburger, zoom, undo/redo,
    /// library, help.
    ///
    /// All of them measure exactly 16×16 on excalidraw.com, so there is one
    /// constant rather than three that have to be kept in step. Upstream's
    /// icons use a `0 0 24 24` viewBox with the stroke inset, so painting them
    /// larger (the earlier 20px) makes the chrome visibly heavier than the real
    /// thing.
    pub const ICON: f32 = 16.0;
    /// `.App-toolbar { padding: .25rem }` — measured 4px on excalidraw.com,
    /// which is what makes the island 44px tall (4 + 36 + 4).
    pub const TOOLBAR_PADDING: f32 = 4.0;
    /// `.App-toolbar .Stack { gap: .25rem }` — measured 4px between buttons.
    pub const TOOLBAR_GAP: f32 = 4.0;
    /// `--border-radius-lg: .5rem` — islands and tool buttons.
    pub const RADIUS_LG: f32 = 8.0;
    /// `--border-radius-md: .375rem` — menu rows, small swatches.
    pub const RADIUS_MD: f32 = 6.0;
    /// Width of the floating style panel (`.panel { width: 15.5rem }`).
    pub const PANEL_WIDTH: f32 = 248.0;
    /// Width of the properties panel (`.Island.App-menu__left`).
    ///
    /// Measured 195×411 with a rectangle selected — it is *not* the 15.5rem of
    /// the library sidebar, and using that width makes the style panel visibly
    /// wider than upstream's.
    pub const PROPERTIES_PANEL_WIDTH: f32 = 195.0;
    /// A tool button's shortcut letter: `.ToolIcon__keybinding { font-size:
    /// 10px }`, pinned at `top: 21px; right: 3px` inside the 36px button.
    pub const KEYBINDING_FONT: f32 = 10.0;
    pub const KEYBINDING_TOP: f32 = 21.0;
    pub const KEYBINDING_RIGHT: f32 = 3.0;
    /// Height of a menu row (`.dropdown-item`) — measured 32px upstream.
    pub const MENU_ROW: f32 = 32.0;
    /// `.dropdown-item { gap: 10px }` — between the icon column and the label.
    pub const MENU_ROW_GAP: f32 = 10.0;
    /// The hamburger menu island's width (`.dropdown-menu-container`).
    ///
    /// Upstream caps the menu at `max-width: 20rem` and lets the widest row win;
    /// here the rows measure 286px, so it is pinned rather than left to depend on
    /// whichever label happens to be longest. The second-level panels beside it
    /// anchor off this constant, so the two stay in step when a row changes.
    pub const MENU_WIDTH: f32 = 286.0;
    /// The gap between the menu and a second-level panel docked to its right.
    pub const MENU_PANEL_GAP: f32 = 8.0;
    /// The shape-switch popup over a selected element
    /// (`.ConvertElementTypePopup`): `padding: .5rem`, `gap: .2rem`,
    /// `border-radius: .5rem`, holding 32px toggle buttons.
    pub const SHAPE_SWITCH_PADDING: f32 = 8.0;
    pub const SHAPE_SWITCH_GAP: f32 = 3.2;
    /// Upstream anchors the popup `GAP_HORIZONTAL: 8` left of the element's
    /// bottom-left corner and `GAP_VERTICAL: 10` plus the 8px handle inset
    /// below it, so the selection handles stay visible.
    pub const SHAPE_SWITCH_OFFSET_X: f32 = 8.0;
    pub const SHAPE_SWITCH_OFFSET_Y: f32 = 18.0;
    /// The island of buttons floating off a selected element's top-right corner
    /// (`.excalidraw-canvas-buttons`): `padding: 5px` (upstream's inline
    /// `CONTAINER_PADDING`), `gap: .375rem`, `border-radius: .5rem`.
    pub const CANVAS_BUTTONS_PADDING: f32 = 5.0;
    pub const CANVAS_BUTTONS_GAP: f32 = 6.0;
    /// `getContainerCoords` puts the island `10px` right of the element's
    /// top-right corner, level with its top edge.
    pub const CANVAS_BUTTONS_OFFSET_X: f32 = 10.0;
    /// `.UnlockPopup { padding: .8rem; gap: .5rem }` holding a `1.25rem` glyph.
    pub const UNLOCK_PADDING: f32 = 12.8;
    pub const UNLOCK_GAP: f32 = 8.0;
    pub const UNLOCK_ICON: f32 = 20.0;
    /// Upstream sits the bubble's *bottom* edge 12px above the locked element's
    /// top edge (`bottom: height + 12 - viewY` with a zero container offset).
    pub const UNLOCK_OFFSET_Y: f32 = 12.0;
    /// `.HintViewer kbd { font-size: 10px }` — the key chips in the hint line.
    pub const KBD_FONT: f32 = 10.0;
    /// `.HintViewer kbd { padding: 1px 3px }`.
    pub const KBD_PAD_X: f32 = 3.0;
    pub const KBD_PAD_Y: f32 = 1.0;
    /// `.HintViewer kbd { border-radius: 4px }`.
    pub const KBD_RADIUS: f32 = 4.0;
    /// `.file-drop-overlay__card`: `width: 30rem`, `padding: 2rem 1.5rem
    /// 1.5rem`, `border-radius: 1.5rem`.
    pub mod file_drop {
        pub const CARD_WIDTH: f32 = 480.0;
        pub const CARD_PAD_TOP: f32 = 32.0;
        pub const CARD_PAD_X: f32 = 24.0;
        pub const CARD_PAD_BOTTOM: f32 = 24.0;
        pub const CARD_RADIUS: f32 = 24.0;
        /// `.file-drop-overlay { padding: 1rem }`.
        pub const OVERLAY_PADDING: f32 = 16.0;
        /// `.file-drop-overlay__title { font-size: 1.5rem }`.
        pub const TITLE_FONT: f32 = 24.0;
        /// `.file-drop-overlay__hint { font-size: .875rem }`.
        pub const HINT_FONT: f32 = 14.0;
        pub const HINT_GAP: f32 = 8.0;
        /// `.file-drop-overlay__hint--secondary { font-size: .75rem }`.
        pub const SECONDARY_FONT: f32 = 12.0;
        /// `.file-drop-overlay__illustration { height: 10rem; margin-top:
        /// 1.75rem }`, holding two `6rem × 8rem` sheets.
        pub const ILLUSTRATION_HEIGHT: f32 = 160.0;
        pub const ILLUSTRATION_GAP: f32 = 28.0;
        pub const SHEET_W: f32 = 96.0;
        pub const SHEET_H: f32 = 128.0;
        /// The sheets sit `top: .5rem`, but the *back* one is nudged down to
        /// `1rem` so the pair fans out rather than stacking.
        pub const SHEET_TOP: f32 = 8.0;
        pub const SHEET_TOP_BACK: f32 = 16.0;
        /// `.file-drop-overlay__hint kbd { padding: .125rem .375rem }`.
        pub const KBD_PAD_X: f32 = 6.0;
        pub const KBD_PAD_Y: f32 = 2.0;
        pub const KBD_RADIUS: f32 = 4.0;
    }

    /// `.Toast`: `min-width: 220px`, `max-width: min(360px, 100vw - 32px)`,
    /// `padding: .5rem .75rem`, `border-radius: var(--border-radius-lg)`, and
    /// a `--color-surface-lowest` 1px ring.
    pub mod toast {
        pub const MIN_WIDTH: f32 = 220.0;
        pub const MAX_WIDTH: f32 = 360.0;
        pub const PAD_X: f32 = 12.0;
        pub const PAD_Y: f32 = 8.0;
        /// `.Toast__message { font-size: .75rem; line-height: 1.25rem }`.
        pub const FONT: f32 = 12.0;
        /// `.close { padding: .4rem }` holding a `1.2rem` glyph.
        pub const CLOSE_PAD: f32 = 6.4;
        pub const CLOSE_ICON: f32 = 19.2;
        /// `.floating-status-stack { bottom: .5rem }`.
        pub const BOTTOM: f32 = 8.0;
    }

    /// `.excalidraw-eye-dropper-preview`: a `2rem` circle with a 1px ring and a
    /// `2px 2px 0 0 #4949497c` offset shadow.
    pub mod eye_dropper {
        pub const PREVIEW: f32 = 32.0;
        /// `positionElementBesideCursor({ gap: 7 })`.
        pub const PREVIEW_GAP: f32 = 7.0;
        /// `.excalidraw-eye-dropper-trigger { width/height: 1.25rem }`.
        pub const TRIGGER: f32 = 20.0;
        pub const TRIGGER_RADIUS: f32 = 8.0;
        /// The trigger's `padding: 4px`, which leaves a 12px glyph.
        pub const TRIGGER_ICON: f32 = 12.0;
        /// The trigger tucks into the swatch row with `margin-right: -4px;
        /// margin-left: -2px` so its padding does not open a gap.
        pub const TRIGGER_MARGIN_RIGHT: f32 = -4.0;
        pub const TRIGGER_MARGIN_LEFT: f32 = -2.0;
    }

    /// `.CursorHint`: `padding: .375rem`, `border-radius: var(--border-radius-md)`,
    /// holding a `1rem` glyph. `CURSOR_HINT_GAP` is upstream's
    /// `positionElementBesideCursor({ gap: 16 })`.
    pub mod cursor_hint {
        pub const PADDING: f32 = 6.0;
        pub const GAP: f32 = 16.0;
        pub const ICON: f32 = 16.0;
    }

    /// `.color-picker__button { width: 1.375rem }` — measured 22×22 on
    /// excalidraw.com, at a 27px pitch (5px gap).
    pub const SWATCH: f32 = 22.0;
    /// The swatch corner radius.
    pub const SWATCH_RADIUS: f32 = 4.0;

    /// The colour picker popup.
    ///
    /// `.color-picker__button--large` is `1.875rem`, laid out by
    /// `.color-picker-content--default` as `repeat(5, 1.875rem)` at a `.25rem`
    /// gap and `.5rem` padding; the popup itself is capped at `13rem`.
    pub const SWATCH_LARGE: f32 = 30.0;
    pub const SWATCH_LARGE_RADIUS: f32 = 6.0;
    /// `.color-picker__button.active-color` — the panel's trigger swatch.
    pub const SWATCH_TRIGGER: f32 = 26.0;
    pub const PICKER_GRID_GAP: f32 = 4.0;
    pub const PICKER_GRID_PAD: f32 = 8.0;
    pub const PICKER_WIDTH: f32 = 208.0;
    pub const PICKER_GAP: f32 = 12.0;
    /// `.color-picker__input-label { border-radius: 8px; padding: 0 12px }`
    /// holding a `--default-button-size` field.
    pub const PICKER_INPUT_RADIUS: f32 = 8.0;
    pub const PICKER_INPUT_PAD: f32 = 12.0;

    /// `.ElementLinkDialog`: `padding: 1.5rem`, `border-radius: 10px`, pinned to
    /// the container's top-left corner.
    pub mod link_dialog {
        pub const PADDING: f32 = 24.0;
        pub const RADIUS: f32 = 10.0;
        /// Upstream leaves the dialog width to its content — the field is a
        /// `flex: 1` child — so it is pinned to a comfortable reading width here
        /// rather than growing with the link inside it.
        pub const WIDTH: f32 = 320.0;
    }

    /// The font picker popup (`.FontPicker__container` / `PropertiesPopover`
    /// styled `width: 15rem`).
    pub mod font_picker {
        pub const WIDTH: f32 = 240.0;
        /// `.dropdown-menu .dropdown-item` — the same 32px row the hamburger
        /// uses, plus its icon gutter.
        pub const ROW_HEIGHT: f32 = 32.0;
        pub const PAD: f32 = 8.0;
        pub const LIST_PAD: f32 = 4.0;
    }

    /// The "Save to…" modal (`Modal.scss`, `Dialog.scss`, `Card.scss`).
    ///
    /// Upstream's `JSONExportDialog` renders one `Card` per export route; the
    /// share-link card needs excalidraw.com's backend and the Excalidraw+ card
    /// is injected by the hosted app, so only the local-file card is drawn here.
    pub mod save_dialog {
        /// `Dialog`'s `regular` width, which `JSONExportDialog` takes by default.
        pub const WIDTH: f32 = 800.0;
        /// `Modal.scss`: the modal's `Island` sits at `padding: 2.5rem`.
        pub const PADDING: f32 = 40.0;
        /// `Modal.scss`: `.Modal { padding: calc(var(--space-factor) * 10) }`,
        /// so the dialog keeps 40px clear of the window edges.
        pub const MODAL_PADDING: f32 = 40.0;
        /// `Modal.scss`: `border-radius: 0.75rem`.
        pub const RADIUS: f32 = 12.0;
        /// `Dialog.scss`: `.Dialog__title { font-size: 1.25rem }`.
        pub const TITLE_FONT: f32 = 20.0;
        /// `Dialog.scss`: the title's `padding: 0 0 0.75rem`.
        pub const TITLE_PAD_BOTTOM: f32 = 12.0;
        /// `Dialog.scss`: the title's `margin-bottom: 1.5rem`.
        pub const TITLE_GAP: f32 = 24.0;
        /// `Card.scss`: `max-width: 290px`. The card is centred in the modal.
        pub const CARD_MAX_WIDTH: f32 = 290.0;
        /// `Card.scss`: `.Card { margin: 1em }`.
        pub const CARD_MARGIN: f32 = 16.0;
        /// `Card.scss`: `svg { width: 2.8rem }`.
        pub const CARD_ICON: f32 = 44.8;
        /// `Card.scss`: `padding: 1.4rem`, which is what makes the lime disc.
        pub const CARD_ICON_PAD: f32 = 22.4;
        /// A card's `<h2>` at the browser default `1.5em`.
        pub const CARD_TITLE_FONT: f32 = 24.0;
        /// A card `<h2>`'s default `margin: 0.83em 0`.
        pub const CARD_TITLE_MARGIN: f32 = 20.0;
        /// `Card.scss`: `.Card-details { font-size: 0.96em }`.
        pub const CARD_DETAILS_FONT: f32 = 15.4;
        /// `Card.scss`: `.Card-details { padding: 0 1em }`.
        pub const CARD_DETAILS_PAD: f32 = 16.0;
        /// `Card.scss`: `.Card-details { min-height: 90px }`, which is what
        /// keeps the cards' buttons on one line.
        pub const CARD_DETAILS_MIN_H: f32 = 90.0;
        /// `Card.scss`: `.Card-button { height: 2.5rem; margin-top: 1em;
        /// margin-bottom: 0.3em }`.
        pub const CARD_BUTTON_H: f32 = 40.0;
        pub const CARD_BUTTON_TOP: f32 = 16.0;
        pub const CARD_BUTTON_BOTTOM: f32 = 4.8;
        /// `ToolIcon.scss`: `.ToolIcon__label { margin: 0 0.8em }` on the
        /// button's `1em` label — the button is only as wide as its text.
        pub const CARD_BUTTON_PAD: f32 = 12.8;
        /// `ToolIcon.scss`: `.ToolIcon { border-radius: var(--border-radius-lg) }`.
        pub const CARD_BUTTON_RADIUS: f32 = 8.0;
        /// `.Card-button` inherits `.Card`'s `1em`.
        pub const CARD_BUTTON_FONT: f32 = 16.0;
        /// `ExportDialog.scss`: `.ProjectName { margin: 1em auto }`.
        pub const NAME_MARGIN: f32 = 16.0;
        /// `ProjectName.scss`: `.ProjectName-label { margin: 0.625em 0 }`.
        pub const NAME_LABEL_MARGIN: f32 = 10.0;
        /// The filename field. `ProjectName.scss` leaves the width to a browser
        /// `<input>`'s intrinsic size (`width: auto` in this dialog), which
        /// measures 179px for an `Untitled-<date>` default; a native app has no
        /// such default, so it is pinned to the value the reference shows.
        pub const NAME_FIELD_W: f32 = 179.0;
        /// `height: calc(1rem - 3px)` plus `input[type="text"]`'s
        /// `padding: 0.75rem` and `border: 1.5px solid` — a content box, which
        /// is why the 13px declaration still renders 40px tall.
        pub const NAME_FIELD_H: f32 = 40.0;
        /// `styles.scss`: `input[type="text"] { padding: 0.75rem }`.
        pub const NAME_FIELD_PAD: f32 = 12.0;
        /// `styles.scss`: `input[type="text"] { border: 1.5px solid }`.
        pub const NAME_FIELD_BORDER: f32 = 1.5;
        /// `styles.scss`: `border-radius: var(--space-factor)` = `0.25rem`.
        pub const NAME_FIELD_RADIUS: f32 = 4.0;
        /// `ProjectName.scss`: `.TextInput` sits inside `.Card-details`, so it
        /// inherits that block's `0.96em`.
        pub const NAME_FIELD_FONT: f32 = 15.4;
    }

    /// The language `<select>` (`.dropdown-select__language`).
    ///
    /// Measured off upstream `styles.scss` / `theme.scss`: `height: 2rem`,
    /// `padding-inline: 0.5rem 1.5rem`, `border-radius: var(--space-factor)`,
    /// `font-size: 0.8rem`, and the `--dropdown-icon` drawn at
    /// `background-size: 0.65em` from `right 0.7rem`, vertically centred.
    pub mod language_select {
        pub const HEIGHT: f32 = 32.0;
        pub const PAD_START: f32 = 8.0;
        pub const PAD_END: f32 = 24.0;
        pub const RADIUS: f32 = 4.0;
        pub const FONT: f32 = 12.8;
        pub const CARET: f32 = 8.32;
        pub const CARET_RIGHT: f32 = 11.2;
        pub const CARET_TOP: f32 = (HEIGHT - CARET) / 2.0;
    }
}

/// `--shadow-island`, expressed as GPUI box shadows.
///
/// Upstream: `0 0 1px rgba(0,0,0,.17), 0 0 3px rgba(0,0,0,.08), 0 7px 14px rgba(0,0,0,.05)`.
/// The ring a chrome chip wears instead of a drop shadow.
///
/// Upstream: `box-shadow: 0 0 0 1px var(--color-surface-lowest)` — a flat 1px
/// outline in the *canvas* colour, which is what makes the zoom / undo chips
/// read as cut-outs from the page rather than as raised islands.
pub fn chip_ring_shadow(tokens: Tokens) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: tokens.surface_lowest().into(),
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(1.0),
        inset: false,
    }]
}

/// The shadow under a selected element's canvas buttons.
///
/// Upstream hard-codes `0 2px 4px 0 rgb(0 0 0 / 30%)` here rather than reusing
/// `--shadow-island`, so the island reads as sitting *on* the element instead of
/// floating above the page — a single, heavier, downward-offset blur.
pub fn canvas_buttons_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0.0, 0.0, 0.0, 0.3),
        offset: point(px(0.0), px(2.0)),
        blur_radius: px(4.0),
        spread_radius: px(0.0),
        inset: false,
    }]
}

/// The shadow under the file-drop overlay's card.
///
/// Upstream hard-codes `0 16px 48px rgba(0,0,0,.08), 0 2px 8px rgba(0,0,0,.04)`
/// so the card reads as lifted well clear of the dimmed canvas.
pub fn file_drop_card_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.08),
            offset: point(px(0.0), px(16.0)),
            blur_radius: px(48.0),
            spread_radius: px(0.0),
            inset: false,
        },
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.04),
            offset: point(px(0.0), px(2.0)),
            blur_radius: px(8.0),
            spread_radius: px(0.0),
            inset: false,
        },
    ]
}

/// The offset drop shadow under the eye-dropper preview swatch.
///
/// Upstream: `box-shadow: 2px 2px 0 0 #4949497c` — a hard-edged shadow, no blur,
/// which is what keeps the small circle legible over any canvas colour.
pub fn eye_dropper_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x4949497c).into(),
        offset: point(px(2.0), px(2.0)),
        blur_radius: px(0.0),
        spread_radius: px(0.0),
        inset: false,
    }]
}

/// `--shadow-island`, expressed as GPUI box shadows.
///
/// Upstream: `0 0 1px rgba(0,0,0,.17), 0 0 3px rgba(0,0,0,.08), 0 7px 14px rgba(0,0,0,.05)`.
pub fn island_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.17),
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(1.0),
            spread_radius: px(0.0),
            inset: false,
        },
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.08),
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(3.0),
            spread_radius: px(0.0),
            inset: false,
        },
        BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.05),
            offset: point(px(0.0), px(7.0)),
            blur_radius: px(14.0),
            spread_radius: px(0.0),
            inset: false,
        },
    ]
}

/// `--modal-shadow`, expressed as GPUI box shadows.
///
/// Upstream: a six-stop elevation ramp from `0 100px 80px rgba(0,0,0,.07)` down
/// to `0 2.77px 2.21px rgba(0,0,0,.0197)`. Deeper than [`island_shadow`], which
/// is what makes a dialog read as floating above the chrome rather than sitting
/// in it.
pub fn modal_shadow() -> Vec<BoxShadow> {
    // (offset-y, blur, alpha) per CSS stop, in upstream's declared order.
    const STOPS: [(f32, f32, f32); 6] = [
        (100.0, 80.0, 0.07),
        (41.7776, 33.4221, 0.0503198),
        (22.3363, 17.869, 0.0417275),
        (12.5216, 10.0172, 0.035),
        (6.6501, 5.32008, 0.0282725),
        (2.76726, 2.21381, 0.0196802),
    ];
    STOPS
        .iter()
        .map(|&(offset_y, blur, alpha)| BoxShadow {
            color: hsla(0.0, 0.0, 0.0, alpha),
            offset: point(px(0.0), px(offset_y)),
            blur_radius: px(blur),
            spread_radius: px(0.0),
            inset: false,
        })
        .collect()
}
