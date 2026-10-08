//! The hand-drawn typefaces canvas text is drawn with.
//!
//! Excalidraw does *not* roughen text geometrically the way it roughens shape
//! outlines — the hand-drawn look is baked into the glyphs themselves. Upstream
//! ships those glyphs as `Excalifont` (its default family) and `Virgil` (the
//! family Excalifont replaced), both under the SIL Open Font License.
//!
//! Upstream serves them as `woff2` and lets the browser decompress. GPUI's font
//! stack parses faces with `ttf-parser`, which only reads `TrueType`/`OpenType`
//! containers, so the files here are the upstream `woff2` subsets decompressed
//! back to `ttf` (`woff2_decompress`) — same glyphs, readable container.
//!
//! Excalifont is split upstream into seven `unicode-range` subsets. Registering
//! all seven would not help: GPUI resolves a family to a single face, so the
//! extra subsets would be shadowed. The Latin subset (U+20-7E, U+00A0-00FF and
//! the punctuation upstream groups with it) is the one that covers canvas text.

use std::borrow::Cow;

use gpui_kit::*;

/// Family name GPUI resolves for Excalidraw's default hand-drawn face.
pub const EXCALIFONT_FAMILY: &str = "Excalifont";

/// Family name GPUI resolves for `Virgil`, the family Excalifont replaced.
pub const VIRGIL_FAMILY: &str = "Virgil";

/// Latin subset of upstream `fonts/Excalifont/Excalifont-Regular-*.woff2`.
const EXCALIFONT_TTF: &[u8] = include_bytes!("../assets/fonts/Excalifont-Regular.ttf");

/// Upstream `fonts/Virgil/Virgil-Regular.woff2`.
const VIRGIL_TTF: &[u8] = include_bytes!("../assets/fonts/Virgil-Regular.ttf");

/// The embedded hand-drawn faces, in registration order.
pub fn hand_drawn_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![Cow::Borrowed(EXCALIFONT_TTF), Cow::Borrowed(VIRGIL_TTF)]
}

/// Install the hand-drawn faces into the app's text system.
///
/// Call once after `gpui_kit::init`, before any window is opened: the text
/// system invalidates cached layouts when a font arrives, but a face that is
/// registered after the first shape pass still costs one frame of fallback
/// rendering.
pub fn register_hand_drawn_fonts(cx: &App) {
    // A failed registration is not fatal — the canvas simply keeps rendering
    // with the system fallback, which is exactly what it did before. There is
    // nothing useful to report to the user from inside a paint setup call.
    let _ = cx.text_system().add_fonts(hand_drawn_fonts());
}

#[cfg(test)]
mod tests {
    // Deliberately not `use super::*`: this crate sits at its `recursion_limit`
    // for macro expansion, and re-importing the `gpui_kit` glob into a nested
    // module is enough to push this file's test build over the compiler's stack.
    use super::{EXCALIFONT_FAMILY, EXCALIFONT_TTF, VIRGIL_FAMILY, VIRGIL_TTF, hand_drawn_fonts};

    /// `0x00010000` is the TrueType sfnt version. A `woff2` file starts with
    /// `wOF2` instead, and `ttf-parser` refuses it — so this catches a future
    /// refresh that copies the upstream `woff2` in by mistake.
    fn is_truetype(bytes: &[u8]) -> bool {
        bytes.len() > 12 && bytes[0..4] == [0x00, 0x01, 0x00, 0x00]
    }

    #[test]
    fn embedded_faces_are_truetype_not_woff2() {
        for font in hand_drawn_fonts() {
            assert!(
                is_truetype(&font),
                "embedded font is not a TrueType sfnt container",
            );
        }
    }

    #[test]
    fn both_hand_drawn_families_are_embedded() {
        let fonts = hand_drawn_fonts();
        assert_eq!(fonts.len(), 2);
        assert!(fonts.iter().any(|f| f.len() == EXCALIFONT_TTF.len()));
        assert!(fonts.iter().any(|f| f.len() == VIRGIL_TTF.len()));
    }

    fn be_u16(b: &[u8], at: usize) -> u16 {
        u16::from_be_bytes([b[at], b[at + 1]])
    }

    fn be_u32(b: &[u8], at: usize) -> u32 {
        u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    /// The family names a face declares, read straight out of its `name` table.
    ///
    /// GPUI resolves `font("...")` by the family stored *inside* the file, so
    /// the name the canvas asks for and the name the file carries have to
    /// agree. A mismatch is silent — text just falls back to a system font.
    fn declared_family_names(bytes: &[u8]) -> Vec<String> {
        let num_tables = be_u16(bytes, 4) as usize;
        let name_table = (0..num_tables)
            .map(|i| 12 + i * 16)
            .find(|rec| &bytes[*rec..*rec + 4] == b"name")
            .map(|rec| be_u32(bytes, rec + 8) as usize)
            .expect("face has no name table");
        let count = be_u16(bytes, name_table + 2) as usize;
        let storage = name_table + be_u16(bytes, name_table + 4) as usize;
        let mut names = Vec::new();
        for i in 0..count {
            let rec = name_table + 6 + i * 12;
            // Platform 3 / encoding 1 is the Windows Unicode table: UTF-16BE.
            // nameID 1 is the family, 16 the typographic family.
            if be_u16(bytes, rec) != 3 || be_u16(bytes, rec + 2) != 1 {
                continue;
            }
            let name_id = be_u16(bytes, rec + 6);
            if name_id != 1 && name_id != 16 {
                continue;
            }
            let len = be_u16(bytes, rec + 8) as usize;
            let off = storage + be_u16(bytes, rec + 10) as usize;
            let units: Vec<u16> = bytes[off..off + len]
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            names.push(String::from_utf16_lossy(&units));
        }
        names
    }

    #[test]
    fn embedded_faces_declare_the_family_the_canvas_asks_for() {
        let names = declared_family_names(EXCALIFONT_TTF);
        assert!(
            names.iter().any(|n| n == EXCALIFONT_FAMILY),
            "Excalifont face declares {names:?}",
        );
        let names = declared_family_names(VIRGIL_TTF);
        assert!(
            names.iter().any(|n| n == VIRGIL_FAMILY),
            "Virgil face declares {names:?}",
        );
    }
}
