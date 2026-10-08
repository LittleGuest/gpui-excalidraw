use excalidraw_core::types::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }
}

impl From<Theme> for ThemeMode {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::Light,
            Theme::Dark => Self::Dark,
        }
    }
}

impl From<ThemeMode> for Theme {
    fn from(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Theme::Dark,
            _ => Theme::Light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
}

impl AppTheme {
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasColors {
    pub background: &'static str,
    pub grid: &'static str,
    pub stroke: &'static str,
}

pub fn canvas_colors(theme: AppTheme) -> CanvasColors {
    match theme {
        AppTheme::Light => CanvasColors {
            background: "#ffffff",
            grid: "#e5e5e5",
            stroke: "#1e1e1e",
        },
        AppTheme::Dark => CanvasColors {
            background: "#121212",
            grid: "#2c2c2c",
            stroke: "#e0e0e0",
        },
    }
}

/// One named entry of upstream `COLOR_PALETTE` (`packages/common/src/colors.ts`).
///
/// A palette entry is either a *single* colour (transparent / white / black) or a
/// five-step ramp — open-color's weights 50, 200, 400, 600, 800. The picker draws
/// every entry at one active shade, which is what lets a whole row of swatches
/// step through light-to-dark together.
pub struct PaletteEntry {
    pub name: &'static str,
    pub shades: &'static [&'static str],
}

impl PaletteEntry {
    /// The colour shown while `shade` is the active ramp step.
    ///
    /// Single-colour entries repeat their one value, so callers never have to
    /// branch on the entry's shape.
    pub fn at(&self, shade: usize) -> &'static str {
        if self.shades.is_empty() {
            return "transparent";
        }
        self.shades[shade.min(self.shades.len() - 1)]
    }

    /// Whether this entry has a ramp to step through (`ShadeList` only shows
    /// shades for these).
    pub fn is_ramp(&self) -> bool {
        self.shades.len() > 1
    }
}

const TRANSPARENT: &[&str] = &["transparent"];
const BLACK: &[&str] = &["#1e1e1e"];
const WHITE: &[&str] = &["#ffffff"];
const GRAY: &[&str] = &["#f8f9fa", "#e9ecef", "#ced4da", "#868e96", "#343a40"];
const RED: &[&str] = &["#fff5f5", "#ffc9c9", "#ff8787", "#fa5252", "#e03131"];
const PINK: &[&str] = &["#fff0f6", "#fcc2d7", "#f783ac", "#e64980", "#c2255c"];
const GRAPE: &[&str] = &["#f8f0fc", "#eebefa", "#da77f2", "#be4bdb", "#9c36b5"];
const VIOLET: &[&str] = &["#f3f0ff", "#d0bfff", "#9775fa", "#7950f2", "#6741d9"];
const BLUE: &[&str] = &["#e7f5ff", "#a5d8ff", "#4dabf7", "#228be6", "#1971c2"];
const CYAN: &[&str] = &["#e3fafc", "#99e9f2", "#3bc9db", "#15aabf", "#0c8599"];
const TEAL: &[&str] = &["#e6fcf5", "#96f2d7", "#38d9a9", "#12b886", "#099268"];
const GREEN: &[&str] = &["#ebfbee", "#b2f2bb", "#69db7c", "#40c057", "#2f9e44"];
const YELLOW: &[&str] = &["#fff9db", "#ffec99", "#ffd43b", "#fab005", "#f08c00"];
const ORANGE: &[&str] = &["#fff4e6", "#ffd8a8", "#ffa94d", "#fd7e14", "#e8590c"];
const BRONZE: &[&str] = &["#f8f1ee", "#eaddd7", "#d2bab0", "#a18072", "#846358"];

/// `DEFAULT_ELEMENT_STROKE_COLOR_PALETTE` — the 5×3 grid of the stroke picker.
///
/// The order is the grid: the first five fill the top row, so it is part of the
/// layout rather than a detail that can be sorted.
pub fn element_palette() -> &'static [PaletteEntry] {
    &[
        PaletteEntry {
            name: "transparent",
            shades: TRANSPARENT,
        },
        PaletteEntry {
            name: "white",
            shades: WHITE,
        },
        PaletteEntry {
            name: "gray",
            shades: GRAY,
        },
        PaletteEntry {
            name: "black",
            shades: BLACK,
        },
        PaletteEntry {
            name: "bronze",
            shades: BRONZE,
        },
        PaletteEntry {
            name: "cyan",
            shades: CYAN,
        },
        PaletteEntry {
            name: "blue",
            shades: BLUE,
        },
        PaletteEntry {
            name: "violet",
            shades: VIOLET,
        },
        PaletteEntry {
            name: "grape",
            shades: GRAPE,
        },
        PaletteEntry {
            name: "pink",
            shades: PINK,
        },
        PaletteEntry {
            name: "green",
            shades: GREEN,
        },
        PaletteEntry {
            name: "teal",
            shades: TEAL,
        },
        PaletteEntry {
            name: "yellow",
            shades: YELLOW,
        },
        PaletteEntry {
            name: "orange",
            shades: ORANGE,
        },
        PaletteEntry {
            name: "red",
            shades: RED,
        },
    ]
}

/// `DEFAULT_ELEMENT_STROKE_COLOR_INDEX` — the ramp step the stroke picker opens on.
pub const DEFAULT_STROKE_SHADE: usize = 4;
/// `DEFAULT_ELEMENT_BACKGROUND_COLOR_INDEX`.
pub const DEFAULT_BACKGROUND_SHADE: usize = 1;

/// `DEFAULT_ELEMENT_STROKE_PICKS` — the five swatches pinned above the trigger.
pub const STROKE_TOP_PICKS: [&str; 5] = ["#1e1e1e", "#e03131", "#2f9e44", "#1971c2", "#f08c00"];
/// `DEFAULT_ELEMENT_BACKGROUND_PICKS`.
pub const BACKGROUND_TOP_PICKS: [&str; 5] =
    ["transparent", "#ffc9c9", "#b2f2bb", "#a5d8ff", "#ffec99"];

/// Whether `color` sits in the palette at all, i.e. whether it is one a user
/// picked themselves (upstream `isCustomColor`).
pub fn is_custom_color(color: &str) -> bool {
    !element_palette()
        .iter()
        .any(|entry| entry.shades.contains(&color))
}

/// Where a colour sits in the palette: its entry's name and ramp step, with
/// `None` for the step of a single-colour entry (upstream
/// `getColorNameAndShadeFromColor`).
pub fn color_position(color: &str) -> Option<(&'static str, Option<usize>)> {
    for entry in element_palette() {
        if let Some(shade) = entry.shades.iter().position(|c| *c == color) {
            return Some((entry.name, if entry.is_ramp() { Some(shade) } else { None }));
        }
    }
    None
}
