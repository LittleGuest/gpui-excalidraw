//! The Excalidraw icon set, embedded and renderable.
//!
//! The native editor used to draw its toolbar as letter placeholders (`V`, `R`,
//! `Snap`, `<-|`), which never reads as Excalidraw. These are the *real* glyphs,
//! extracted from upstream `icons.tsx` by `tools/extract_icons.py`.
//!
//! How rendering works: GPUI rasterises an SVG into an **alpha mask** and tints
//! it with the element's text colour (`Svg` uses `style.text.color`). So the
//! `currentColor` in these files resolves to black upstream and is repainted by
//! the caller — which is exactly how Excalidraw itself switches icons between
//! light and dark themes. [`icon`] pins that behaviour in one place.

use std::borrow::Cow;

use gpui_kit::*;

include!(concat!(env!("OUT_DIR"), "/icons_generated.rs"));

/// Prefix every icon path carries inside the asset source.
const PREFIX: &str = "icons/";

/// The asset source that serves [`ICON_FILES`].
///
/// Register it once at startup:
/// `gpui_kit::application().with_assets(IconAssets)`.
///
/// Unknown paths fall through to GPUI Kit's bundled Lucide catalog so the
/// styling components keep working.
#[derive(Clone, Copy, Debug, Default)]
pub struct IconAssets;

impl AssetSource for IconAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let name = path.strip_prefix(PREFIX).unwrap_or(path);
        if name.is_empty() {
            return Ok(None);
        }
        if let Some((_, svg)) = ICON_FILES.iter().find(|(file, _)| *file == name) {
            return Ok(Some(Cow::Borrowed(svg.as_bytes())));
        }
        // Fall through to GPUI Kit's bundled catalog so the styling components
        // keep their own icons. `AllAssets` *errors* on a miss; we deliberately
        // flatten that into `None` so an unknown path — most likely a typo in
        // an icon name — is a quiet no-op instead of an error surfaced from
        // deep inside the paint pass.
        Ok(gpui_kit::assets::AllAssets.load(path).ok().flatten())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut names: Vec<SharedString> = ICON_FILES
            .iter()
            .filter_map(|(file, _)| {
                let full = format!("{PREFIX}{file}");
                // Asset paths are listed as served, i.e. including the prefix.
                full.starts_with(path).then(|| full.into())
            })
            .collect();
        names.extend(gpui_kit::assets::AllAssets.list(path).unwrap_or_default());
        Ok(names)
    }
}

/// The asset path for an upstream icon name, e.g. `SelectionIcon`.
pub fn icon_path(name: &str) -> SharedString {
    format!("{PREFIX}{name}.svg").into()
}

/// Whether `name` ships with the editor. Used by tests and by debug assertions
/// in the toolbar so a typo cannot silently render a blank button.
pub fn has_icon(name: &str) -> bool {
    ICON_FILES
        .iter()
        .any(|(file, _)| *file == format!("{name}.svg"))
}

/// Every bundled icon name, without the `.svg` suffix.
pub fn icon_names() -> impl Iterator<Item = &'static str> {
    ICON_FILES
        .iter()
        .map(|(file, _)| file.strip_suffix(".svg").unwrap_or(file))
}

/// The embedded SVG source for an upstream icon name, if it ships with the
/// editor. Exposed so tests can assert the shipped markup is well-formed
/// without duplicating the icon table.
pub fn icon_source(name: &str) -> Option<&'static str> {
    ICON_FILES
        .iter()
        .find(|(file, _)| *file == format!("{name}.svg"))
        .map(|(_, svg)| *svg)
}

/// A tinted icon at a fixed size.
///
/// `size` is the square box the SVG is fitted into; icons such as the arrowhead
/// previews are wider than they are tall and letterbox inside it.
///
/// Every icon in the chrome funnels through here, which makes this the one
/// place a mistyped name can be caught. A miss renders as *nothing* — not an
/// error — so an unchecked typo would ship as an invisible button; the debug
/// assertion turns that into a loud failure during development instead.
pub fn icon(name: &str, size: f32, color: Rgba) -> impl IntoElement + use<> {
    debug_assert!(
        has_icon(name),
        "{name} is not in the bundled icon set — check the spelling against assets/icons/"
    );
    svg()
        .path(icon_path(name))
        .flex_shrink_0()
        .size(px(size))
        .text_color(color)
}
