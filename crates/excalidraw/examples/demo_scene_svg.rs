//! Renders the editor's built-in demo scene to SVG so the in-app content can
//! be inspected without opening a window.
//!
//! Run with: `cargo run -p excalidraw --example demo_scene_svg`

use excalidraw::Editor;

fn main() {
    let mut editor = Editor::new();
    editor.add_demo_scene();
    let svg = editor.export_svg_string();
    std::fs::write("demo-scene.svg", &svg).expect("write demo-scene.svg");
    println!(
        "wrote demo-scene.svg ({} elements, {} bytes)",
        editor.document.scene.len(),
        svg.len()
    );
}
