// The render tree for the editor is deeply nested element chains, and the
// `#[test]` expansion over them exceeds the default macro recursion budget.
#![recursion_limit = "512"]

pub mod bitmap;
pub mod canvas;
pub mod chrome;
pub mod design;
pub mod editor;
pub mod export;
pub mod fonts;
pub mod i18n;
pub mod icons;
pub mod layers;
pub mod library;
pub mod properties;
pub mod state;
pub mod theme;

pub use canvas::*;
pub use chrome::*;
pub use design::*;
pub use editor::*;
pub use export::*;
pub use fonts::{EXCALIFONT_FAMILY, VIRGIL_FAMILY, register_hand_drawn_fonts};
pub use i18n::*;
pub use icons::*;
pub use layers::*;
pub use library::*;
pub use properties::*;
pub use state::*;
pub use theme::*;
