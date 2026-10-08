//! Guards for the Excalidraw design tokens.
//!
//! The whole point of `design.rs` is that its values are *copied from upstream,
//! not invented*. These tests pin the handful most likely to drift, so a
//! well-meaning "tidy up the colour" edit can't quietly turn Excalidraw purple
//! into generic SaaS purple.

use excalidraw_ui::design::{DARK, LIGHT, Palette, Tokens, size};

#[test]
fn primary_colours_match_upstream() {
    assert_eq!(LIGHT.primary, 0x6965db, "light --color-primary");
    assert_eq!(DARK.primary, 0xa8a5ff, "dark --color-primary");
    assert_eq!(LIGHT.canvas_bg, 0xffffff, "light --default-bg-color");
    assert_eq!(DARK.canvas_bg, 0x121212, "dark --default-bg-color");
    assert_eq!(LIGHT.island_bg, 0xffffff, "light --island-bg-color");
    assert_eq!(DARK.island_bg, 0x232329, "dark --island-bg-color");
}

#[test]
fn selected_tool_button_uses_the_primary_container_pair() {
    // Upstream paints a checked tool with --color-primary-light and inverts the
    // glyph to --color-on-primary-container. Those two must stay in step or the
    // active tool becomes unreadable.
    for tokens in [LIGHT, DARK] {
        assert_ne!(
            tokens.selected_bg(),
            tokens.selected_fg(),
            "checked tool needs contrast between its fill and its glyph"
        );
        assert_ne!(tokens.selected_bg(), tokens.island());
    }
}

#[test]
fn dark_palette_is_actually_dark_and_light_is_actually_light() {
    let luminance = |c: u32| {
        let (r, g, b) = ((c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff);
        0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
    };
    assert!(luminance(LIGHT.canvas_bg) > 200.0);
    assert!(luminance(DARK.canvas_bg) < 60.0);
    // Text must sit at the opposite end of the range from its surface.
    assert!(luminance(LIGHT.on_surface) < 60.0);
    assert!(luminance(DARK.on_surface) > 200.0);
}

#[test]
fn palette_toggles_round_trip() {
    assert_eq!(Palette::Light.toggled(), Palette::Dark);
    assert_eq!(Palette::Dark.toggled(), Palette::Light);
    assert!(!Palette::Light.is_dark() && Palette::Dark.is_dark());
    assert_eq!(Tokens::for_palette(Palette::Dark).palette, Palette::Dark);
    assert_eq!(
        Tokens::for_palette(Palette::Light).island_bg,
        LIGHT.island_bg
    );
}

/// The sizing constants are *measurements*, not rem arithmetic.
///
/// This test used to assert `TOOL_ICON == 1.25rem` (20px) because that is what
/// upstream's stylesheet appears to say. The rendered glyph on excalidraw.com
/// is 16px, and the 20px reading made the whole toolbar visibly heavier. Only
/// measured values belong here.
#[test]
fn sizes_match_what_excalidraw_com_actually_renders() {
    // Chrome geometry, read off the live DOM.
    assert_eq!(size::CONTAINER_PADDING, 16.0);
    assert_eq!(size::BUTTON_LG, 36.0, "tool and chrome buttons");
    assert_eq!(size::ICON, 16.0, "every 36px button holds a 16px glyph");
    assert_eq!(size::TOOLBAR_PADDING, 4.0, "island is 4 + 36 + 4 = 44 tall");
    assert_eq!(size::TOOLBAR_GAP, 4.0, "button pitch is 40px");
    assert_eq!(size::RADIUS_LG, 8.0);
    // The mobile/small footer button is not used by the desktop toolbar.
    assert_eq!(size::BUTTON, 32.0);
}

#[test]
fn the_toolbar_is_the_only_drop_shadowed_surface() {
    // The visible difference that made the chrome look wrong: every chip
    // (hamburger, zoom, undo/redo, help) is a flat --color-surface-low fill
    // wearing a 1px --color-surface-lowest *ring*, while the toolbar is white
    // with the three-layer island shadow.
    for tokens in [LIGHT, DARK] {
        assert_ne!(
            tokens.surface_lowest(),
            tokens.surface_low(),
            "the chip's ring has to be visible against its own fill"
        );
    }
    // In light the two surfaces are far apart (lavender-grey vs white). In dark
    // they are the *same* #232329 — upstream relies on the ring alone there, so
    // only the light theme is asserted to differ.
    assert_ne!(
        LIGHT.surface_low(),
        LIGHT.island(),
        "in light a chip must read as a different surface from the toolbar"
    );
    assert_eq!(LIGHT.surface_low, 0xececf4, "light --color-surface-low");
    assert_eq!(
        LIGHT.surface_lowest, 0xffffff,
        "light --color-surface-lowest"
    );
    assert_eq!(DARK.surface_low, 0x232329, "dark --color-surface-low");
    assert_eq!(DARK.surface_lowest, 0x121212, "dark --color-surface-lowest");

    // And the separator colour is surface-high, not a mid grey.
    assert_eq!(LIGHT.surface_high, 0xf1f0ff, "--color-surface-high");
    assert_eq!(LIGHT.separator(), LIGHT.surface_high());

    let ring = excalidraw_ui::design::chip_ring_shadow(LIGHT);
    assert_eq!(ring.len(), 1, "the ring is a single shadow layer");
    assert_eq!(ring[0].blur_radius, gpui_kit::px(0.0), "a ring has no blur");
    assert_eq!(ring[0].spread_radius, gpui_kit::px(1.0), "exactly 1px wide");
    assert!(!ring[0].inset, "the ring sits outside the fill");
}

#[test]
fn toolbar_buttons_leave_room_for_their_glyph() {
    const { assert!(size::BUTTON_LG > size::ICON) };
    // The shortcut letter is 10px in a 36px button — not "smaller than half the
    // glyph", which is what this test used to claim and which upstream simply
    // does not do. What actually keeps the two apart is that the badge is
    // pinned *below* the glyph's centre line and hard against the right edge,
    // so it clears the 16px glyph diagonally.
    const { assert!(size::KEYBINDING_FONT <= size::ICON) };
    const { assert!(size::KEYBINDING_TOP > size::BUTTON_LG / 2.0) };
    const { assert!(size::KEYBINDING_TOP + size::KEYBINDING_FONT <= size::BUTTON_LG) };
}

#[test]
fn island_shadow_layers_match_the_upstream_stack() {
    let shadows = excalidraw_ui::design::island_shadow();
    assert_eq!(shadows.len(), 3, "--shadow-island is a three-layer stack");
    // Upstream grows blur and opacity downwards; the last layer is the soft
    // drop that gives islands their lift.
    assert!(shadows[0].blur_radius < shadows[1].blur_radius);
    assert!(shadows[1].blur_radius < shadows[2].blur_radius);
    assert!(shadows[2].offset.y > shadows[0].offset.y);
}
