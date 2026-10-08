use gpui_excalidraw as excalidraw;
use gpui_kit::*;

fn main() {
    // The chrome is drawn with the *real* Excalidraw glyphs, which are embedded
    // in `excalidraw-ui` behind its own `AssetSource`. Registering it here is
    // what makes every `svg()` in the toolbar resolve; without it GPUI would
    // have no way to find `icons/<name>.svg` and the buttons would come out
    // blank.
    gpui_kit::application()
        .with_assets(excalidraw::ui::icons::IconAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            // Canvas text is only hand-drawn if the hand-drawn glyphs are
            // actually installed: Excalifont and Virgil are embedded in the UI
            // crate, and the text system needs them before the first paint or
            // the opening frame comes out in a system font.
            excalidraw::ui::fonts::register_hand_drawn_fonts(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| {
                    // Upstream opens onto an empty canvas with the welcome
                    // screen; the demo scene stays available for tests and the
                    // verify examples. `EXCALIDRAW_UI_STATE` boots straight
                    // into a given state for the screenshot loop.
                    let mut editor = excalidraw::Editor::new();
                    match std::env::var("EXCALIDRAW_UI_STATE").as_deref() {
                        Ok("scene") => editor.add_demo_scene(),
                        Ok("menu") => editor.set_menu_open(true),
                        Ok("prefs") => editor.debug_open_prefs(),
                        Ok("more") => editor.set_more_tools_open(true),
                        Ok("palette") => editor.open_palette(),
                        Ok("find") => {
                            editor.add_demo_scene();
                            editor.open_find();
                            editor.set_find_query("e");
                        }
                        Ok("help") => editor.open_help(),
                        _ => {}
                    }
                    editor
                })
            })
            .expect("failed to open window");
        });
}
