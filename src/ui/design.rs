use gpui_kit::*;

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

#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub palette: Palette,

    pub canvas_bg: u32,

    pub island_bg: u32,

    pub on_surface: u32,

    pub primary: u32,

    pub primary_hover: u32,

    pub primary_container: u32,

    pub on_primary_container: u32,

    pub button_hover: u32,

    pub keybinding: u32,

    pub surface_high: u32,

    pub surface_low: u32,

    pub surface_lowest: u32,

    pub primary_darkest: u32,

    pub primary_light_darker: u32,

    pub swatch_outline: u32,

    pub input_label: u32,

    pub grid: u32,
}

pub const LIGHT: Tokens = Tokens {
    palette: Palette::Light,
    canvas_bg: 0xffffff,
    island_bg: 0xffffff,
    surface_high: 0xf1f0ff,
    on_surface: 0x1b1b1f,
    primary: 0x6965db,
    primary_hover: 0x5753d0,
    primary_container: 0xe3e2fe,
    on_primary_container: 0x030064,
    button_hover: 0xf1f0ff,
    keybinding: 0xb8b8b8,
    surface_low: 0xececf4,
    surface_lowest: 0xffffff,
    primary_darkest: 0x4a47b1,
    primary_light_darker: 0xd7d5ff,
    swatch_outline: 0xebebeb,
    input_label: 0x495057,
    grid: 0xe9ecef,
};

pub const DARK: Tokens = Tokens {
    palette: Palette::Dark,
    canvas_bg: 0x121212,
    island_bg: 0x232329,
    surface_high: 0x2e2d39,
    on_surface: 0xe3e3e8,
    primary: 0xa8a5ff,
    primary_hover: 0xbbb8ff,
    primary_container: 0x4f4d6f,
    on_primary_container: 0xdedcff,
    button_hover: 0x2e2d39,
    keybinding: 0x7a7a7a,
    surface_low: 0x232329,
    surface_lowest: 0x121212,
    primary_darkest: 0xbeb9ff,
    primary_light_darker: 0x43415e,
    swatch_outline: 0x3d3d3d,
    input_label: 0xe9ecef,
    grid: 0x232329,
};

impl Tokens {
    pub fn for_palette(palette: Palette) -> Self {
        match palette {
            Palette::Light => LIGHT,
            Palette::Dark => DARK,
        }
    }

    pub fn island(&self) -> Rgba {
        rgb(self.island_bg)
    }

    pub fn surface_high(&self) -> Rgba {
        rgb(self.surface_high)
    }

    pub fn island_border(&self) -> Rgba {
        rgb(self.surface_high)
    }

    pub fn surface_low(&self) -> Rgba {
        rgb(self.surface_low)
    }

    pub fn gray_60(&self) -> Rgba {
        rgb(0x7a7a7a)
    }

    pub fn primary_darkest(&self) -> Rgba {
        rgb(self.primary_darkest)
    }

    pub fn primary_light_darker(&self) -> Rgba {
        rgb(self.primary_light_darker)
    }

    pub fn swatch_outline(&self) -> Rgba {
        rgb(self.swatch_outline)
    }

    pub fn input_label(&self) -> Rgba {
        rgb(self.input_label)
    }

    pub fn input_border(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgb(0xced4da),
            Palette::Dark => rgb(0x2e2e2e),
        }
    }

    pub fn brand_hover(&self) -> Rgba {
        rgb(self.primary_hover)
    }

    pub fn dialog_border(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgb(0xebebeb),
            Palette::Dark => rgb(0x3d3d3d),
        }
    }

    pub fn overlay_backdrop(&self) -> Rgba {
        match self.palette {
            Palette::Light => rgba(0x0c0c0e59),
            Palette::Dark => rgba(0x0c0c0ea6),
        }
    }

    pub fn cursor_hint_bg(&self) -> Rgba {
        rgb(0x232329)
    }

    pub fn cursor_hint_fg(&self) -> Rgba {
        rgb(0xe3e3e8)
    }

    pub fn surface_lowest(&self) -> Rgba {
        rgb(self.surface_lowest)
    }

    pub fn surface_text(&self) -> Rgba {
        rgb(self.on_surface)
    }

    pub fn border(&self) -> Rgba {
        self.island_border()
    }

    pub fn accent(&self) -> Rgba {
        rgb(self.primary)
    }

    pub fn hover(&self) -> Rgba {
        rgb(self.button_hover)
    }

    pub fn selected_bg(&self) -> Rgba {
        rgb(self.primary_container)
    }

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

pub mod size {

    pub const CONTAINER_PADDING: f32 = 16.0;

    pub const BUTTON_LG: f32 = 36.0;

    pub const BUTTON: f32 = 32.0;

    pub const ICON: f32 = 16.0;

    pub const TOOLBAR_PADDING: f32 = 4.0;

    pub const TOOLBAR_GAP: f32 = 4.0;

    pub const RADIUS_LG: f32 = 8.0;

    pub const RADIUS_MD: f32 = 6.0;

    pub const PANEL_WIDTH: f32 = 248.0;

    pub const PROPERTIES_PANEL_WIDTH: f32 = 195.0;

    pub const KEYBINDING_FONT: f32 = 10.0;
    pub const KEYBINDING_TOP: f32 = 21.0;
    pub const KEYBINDING_RIGHT: f32 = 3.0;

    pub const MENU_ROW: f32 = 32.0;

    pub const MENU_ROW_GAP: f32 = 10.0;

    pub const MENU_WIDTH: f32 = 286.0;

    pub const MENU_PANEL_GAP: f32 = 8.0;

    pub const SHAPE_SWITCH_PADDING: f32 = 8.0;
    pub const SHAPE_SWITCH_GAP: f32 = 3.2;

    pub const SHAPE_SWITCH_OFFSET_X: f32 = 8.0;
    pub const SHAPE_SWITCH_OFFSET_Y: f32 = 18.0;

    pub const CANVAS_BUTTONS_PADDING: f32 = 5.0;
    pub const CANVAS_BUTTONS_GAP: f32 = 6.0;

    pub const CANVAS_BUTTONS_OFFSET_X: f32 = 10.0;

    pub const UNLOCK_PADDING: f32 = 12.8;
    pub const UNLOCK_GAP: f32 = 8.0;
    pub const UNLOCK_ICON: f32 = 20.0;

    pub const UNLOCK_OFFSET_Y: f32 = 12.0;

    pub const KBD_FONT: f32 = 10.0;

    pub const KBD_PAD_X: f32 = 3.0;
    pub const KBD_PAD_Y: f32 = 1.0;

    pub const KBD_RADIUS: f32 = 4.0;

    pub mod file_drop {
        pub const CARD_WIDTH: f32 = 480.0;
        pub const CARD_PAD_TOP: f32 = 32.0;
        pub const CARD_PAD_X: f32 = 24.0;
        pub const CARD_PAD_BOTTOM: f32 = 24.0;
        pub const CARD_RADIUS: f32 = 24.0;

        pub const OVERLAY_PADDING: f32 = 16.0;

        pub const TITLE_FONT: f32 = 24.0;

        pub const HINT_FONT: f32 = 14.0;
        pub const HINT_GAP: f32 = 8.0;

        pub const SECONDARY_FONT: f32 = 12.0;

        pub const ILLUSTRATION_HEIGHT: f32 = 160.0;
        pub const ILLUSTRATION_GAP: f32 = 28.0;
        pub const SHEET_W: f32 = 96.0;
        pub const SHEET_H: f32 = 128.0;

        pub const SHEET_TOP: f32 = 8.0;
        pub const SHEET_TOP_BACK: f32 = 16.0;

        pub const KBD_PAD_X: f32 = 6.0;
        pub const KBD_PAD_Y: f32 = 2.0;
        pub const KBD_RADIUS: f32 = 4.0;
    }

    pub mod toast {
        pub const MIN_WIDTH: f32 = 220.0;
        pub const MAX_WIDTH: f32 = 360.0;
        pub const PAD_X: f32 = 12.0;
        pub const PAD_Y: f32 = 8.0;

        pub const FONT: f32 = 12.0;

        pub const CLOSE_PAD: f32 = 6.4;
        pub const CLOSE_ICON: f32 = 19.2;

        pub const BOTTOM: f32 = 8.0;
    }

    pub mod eye_dropper {
        pub const PREVIEW: f32 = 32.0;

        pub const PREVIEW_GAP: f32 = 7.0;

        pub const TRIGGER: f32 = 20.0;
        pub const TRIGGER_RADIUS: f32 = 8.0;

        pub const TRIGGER_ICON: f32 = 12.0;

        pub const TRIGGER_MARGIN_RIGHT: f32 = -4.0;
        pub const TRIGGER_MARGIN_LEFT: f32 = -2.0;
    }

    pub mod cursor_hint {
        pub const PADDING: f32 = 6.0;
        pub const GAP: f32 = 16.0;
        pub const ICON: f32 = 16.0;
    }

    pub const SWATCH: f32 = 22.0;

    pub const SWATCH_RADIUS: f32 = 4.0;

    pub const SWATCH_LARGE: f32 = 30.0;
    pub const SWATCH_LARGE_RADIUS: f32 = 6.0;

    pub const SWATCH_TRIGGER: f32 = 26.0;
    pub const PICKER_GRID_GAP: f32 = 4.0;
    pub const PICKER_GRID_PAD: f32 = 8.0;
    pub const PICKER_WIDTH: f32 = 208.0;
    pub const PICKER_GAP: f32 = 12.0;

    pub const PICKER_INPUT_RADIUS: f32 = 8.0;
    pub const PICKER_INPUT_PAD: f32 = 12.0;

    pub mod link_dialog {
        pub const PADDING: f32 = 24.0;
        pub const RADIUS: f32 = 10.0;

        pub const WIDTH: f32 = 320.0;
    }

    pub mod font_picker {
        pub const WIDTH: f32 = 240.0;

        pub const ROW_HEIGHT: f32 = 32.0;
        pub const PAD: f32 = 8.0;
        pub const LIST_PAD: f32 = 4.0;
    }

    pub mod save_dialog {

        pub const WIDTH: f32 = 800.0;

        pub const PADDING: f32 = 40.0;

        pub const MODAL_PADDING: f32 = 40.0;

        pub const RADIUS: f32 = 12.0;

        pub const TITLE_FONT: f32 = 20.0;

        pub const TITLE_PAD_BOTTOM: f32 = 12.0;

        pub const TITLE_GAP: f32 = 24.0;

        pub const CARD_MAX_WIDTH: f32 = 290.0;

        pub const CARD_MARGIN: f32 = 16.0;

        pub const CARD_ICON: f32 = 44.8;

        pub const CARD_ICON_PAD: f32 = 22.4;

        pub const CARD_TITLE_FONT: f32 = 24.0;

        pub const CARD_TITLE_MARGIN: f32 = 20.0;

        pub const CARD_DETAILS_FONT: f32 = 15.4;

        pub const CARD_DETAILS_PAD: f32 = 16.0;

        pub const CARD_DETAILS_MIN_H: f32 = 90.0;

        pub const CARD_BUTTON_H: f32 = 40.0;
        pub const CARD_BUTTON_TOP: f32 = 16.0;
        pub const CARD_BUTTON_BOTTOM: f32 = 4.8;

        pub const CARD_BUTTON_PAD: f32 = 12.8;

        pub const CARD_BUTTON_RADIUS: f32 = 8.0;

        pub const CARD_BUTTON_FONT: f32 = 16.0;

        pub const NAME_MARGIN: f32 = 16.0;

        pub const NAME_LABEL_MARGIN: f32 = 10.0;

        pub const NAME_FIELD_W: f32 = 179.0;

        pub const NAME_FIELD_H: f32 = 40.0;

        pub const NAME_FIELD_PAD: f32 = 12.0;

        pub const NAME_FIELD_BORDER: f32 = 1.5;

        pub const NAME_FIELD_RADIUS: f32 = 4.0;

        pub const NAME_FIELD_FONT: f32 = 15.4;
    }

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

pub fn chip_ring_shadow(tokens: Tokens) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: tokens.surface_lowest().into(),
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(1.0),
        inset: false,
    }]
}

pub fn canvas_buttons_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0.0, 0.0, 0.0, 0.3),
        offset: point(px(0.0), px(2.0)),
        blur_radius: px(4.0),
        spread_radius: px(0.0),
        inset: false,
    }]
}

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

pub fn eye_dropper_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x4949497c).into(),
        offset: point(px(2.0), px(2.0)),
        blur_radius: px(0.0),
        spread_radius: px(0.0),
        inset: false,
    }]
}

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

pub fn modal_shadow() -> Vec<BoxShadow> {
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
