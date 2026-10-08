use std::{collections::HashMap, fmt::Write, sync::OnceLock};

use excalidraw_core::{
    bounds::{common_bounds, element_bounds},
    element::Element,
    scene::{BinaryFileData, Scene},
};

use crate::shape::{Drawable, Op, render_element};

pub type BinaryFiles = HashMap<String, BinaryFileData>;

pub struct SvgExporter;

pub fn rasterize_png(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, svg_options()).map_err(|e| e.to_string())?;
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let width = (tree.size().width() * scale).ceil().max(1.0) as u32;
    let height = (tree.size().height() * scale).ceil().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("invalid pixmap size {width}x{height}"))?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

pub fn rasterize_svg(svg: &[u8], scale: f32) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(svg).map_err(|e| e.to_string())?;
    rasterize_png(text, scale)
}

pub fn export_png(scene: &Scene, background: &str, scale: f32) -> Result<Vec<u8>, String> {
    export_png_with_files(scene, background, scale, &BinaryFiles::new())
}

pub fn export_png_with_files(
    scene: &Scene,
    background: &str,
    scale: f32,
    files: &BinaryFiles,
) -> Result<Vec<u8>, String> {
    rasterize_png(
        &SvgExporter::export_with_files(scene, background, files),
        scale,
    )
}

fn svg_options() -> &'static usvg::Options<'static> {
    static OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut options = usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        options
    })
}

impl SvgExporter {
    pub fn export(scene: &Scene, background: &str) -> String {
        Self::export_with_files(scene, background, &BinaryFiles::new())
    }

    pub fn export_with_files(scene: &Scene, background: &str, files: &BinaryFiles) -> String {
        let elements: Vec<&Element> = scene.non_deleted().collect();
        let bounds = common_bounds(&elements)
            .unwrap_or(excalidraw_core::Bounds::new(0.0, 0.0, 100.0, 100.0));
        let padding = 20.0;
        let width = bounds.width() + padding * 2.0;
        let height = bounds.height() + padding * 2.0;
        let ox = bounds.min_x - padding;
        let oy = bounds.min_y - padding;

        let mut svg = String::new();
        let _ = write!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.1}\" height=\"{:.1}\" viewBox=\"{:.1} {:.1} {:.1} {:.1}\">",
            width, height, ox, oy, width, height
        );

        if !excalidraw_core::color::is_transparent(background) {
            let _ = write!(
                svg,
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>",
                ox, oy, width, height, background
            );
        }

        for element in &elements {
            let drawables = render_element(element);
            for d in drawables {
                write_drawable(&mut svg, &d, files);
            }
        }

        svg.push_str("</svg>");
        svg
    }

    pub fn export_element(element: &Element, background: &str) -> String {
        Self::export_element_with_files(element, background, &BinaryFiles::new())
    }

    pub fn export_element_with_files(
        element: &Element,
        background: &str,
        files: &BinaryFiles,
    ) -> String {
        let b = element_bounds(element);
        let padding = 10.0;
        let width = b.width() + padding * 2.0;
        let height = b.height() + padding * 2.0;
        let ox = b.min_x - padding;
        let oy = b.min_y - padding;

        let mut svg = String::new();
        let _ = write!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.1}\" height=\"{:.1}\" viewBox=\"{:.1} {:.1} {:.1} {:.1}\">",
            width, height, ox, oy, width, height
        );
        if !excalidraw_core::color::is_transparent(background) {
            let _ = write!(
                svg,
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>",
                ox, oy, width, height, background
            );
        }
        for d in render_element(element) {
            write_drawable(&mut svg, &d, files);
        }
        svg.push_str("</svg>");
        svg
    }
}

fn write_drawable(svg: &mut String, d: &Drawable, files: &BinaryFiles) {
    if let Some(t) = &d.text {
        let color = format!(
            "rgba({},{},{},{})",
            (t.color.r * 255.0) as u8,
            (t.color.g * 255.0) as u8,
            (t.color.b * 255.0) as u8,
            t.color.a
        );
        // The anchor point lives *inside* the text box: `base.x` is the box's
        // left edge, so centered/right text must be offset by the box width,
        // otherwise the glyphs render to the left of where they belong.
        let (anchor, anchor_x) = match t.text_align {
            excalidraw_core::types::TextAlign::Left => ("start", t.x),
            excalidraw_core::types::TextAlign::Center => ("middle", t.x + t.width / 2.0),
            excalidraw_core::types::TextAlign::Right => ("end", t.x + t.width),
        };
        let font_family = match t.font_family {
            excalidraw_core::types::FontFamily::Virgil => "Virgil, sans-serif",
            excalidraw_core::types::FontFamily::Helvetica => "Helvetica, sans-serif",
            excalidraw_core::types::FontFamily::Cascadia => "Cascadia, monospace",
            excalidraw_core::types::FontFamily::Assistant => "Assistant, sans-serif",
            excalidraw_core::types::FontFamily::Excalifont => "Excalifont, sans-serif",
            excalidraw_core::types::FontFamily::ComicShanns => "ComicShanns, cursive",
            excalidraw_core::types::FontFamily::Liberation => "Liberation Sans, sans-serif",
            excalidraw_core::types::FontFamily::Nunito => "Nunito, sans-serif",
            excalidraw_core::types::FontFamily::Lilita => "Lilita One, sans-serif",
            excalidraw_core::types::FontFamily::Xiaolai => "Xiaolai, sans-serif",
        };
        // Excalidraw keeps text vertically centered inside its box; lay each
        // physical line out with an explicit `<tspan>` so multi-line labels
        // (and container-bound sticky-note text) render correctly in SVG.
        let lines: Vec<&str> = t.text.split('\n').collect();
        let line_h = t.font_size * t.line_height;
        let total_h = line_h * lines.len() as f64;
        let block_top = match t.vertical_align {
            excalidraw_core::types::VerticalAlign::Top => t.y,
            excalidraw_core::types::VerticalAlign::Middle => t.y + (t.height - total_h) / 2.0,
            excalidraw_core::types::VerticalAlign::Bottom => t.y + t.height - total_h,
        };
        let _ = write!(
            svg,
            "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" font-family=\"{}\" fill=\"{}\" text-anchor=\"{}\" dominant-baseline=\"middle\">",
            anchor_x,
            block_top + line_h / 2.0,
            t.font_size,
            font_family,
            color,
            anchor
        );
        for (i, line) in lines.iter().enumerate() {
            let y = block_top + (i as f64 + 0.5) * line_h;
            let _ = write!(
                svg,
                "<tspan x=\"{:.2}\" y=\"{:.2}\">{}</tspan>",
                anchor_x,
                y,
                escape_xml(line)
            );
        }
        svg.push_str("</text>");
        return;
    }

    if let Some(img) = &d.image {
        if let Some(data) = img.file_id.as_deref().and_then(|id| files.get(id)) {
            let _ = write!(
                svg,
                "<image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"none\" href=\"{}\"/>",
                img.x,
                img.y,
                img.width,
                img.height,
                escape_xml(&data.data_url)
            );
            return;
        }
        let _ = write!(
            svg,
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#d0d0d0\" stroke=\"#868e96\" stroke-width=\"1\"/>",
            img.x, img.y, img.width, img.height
        );
        let _ = write!(
            svg,
            "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"12\" fill=\"#495057\" text-anchor=\"middle\">image</text>",
            img.x + img.width / 2.0,
            img.y + img.height / 2.0
        );
        return;
    }

    // Free-draw ink is a *filled outline* (perfect-freehand), not a stroked
    // path — upstream emits `<path fill=strokeColor>` for the pencil.
    if let Some(ink) = &d.ink {
        let path = ops_to_path(ink);
        let fill = format!(
            "rgba({},{},{},{})",
            (d.options.stroke_color.r * 255.0) as u8,
            (d.options.stroke_color.g * 255.0) as u8,
            (d.options.stroke_color.b * 255.0) as u8,
            d.options.stroke_color.a
        );
        let _ = write!(
            svg,
            "<path d=\"{}\" fill=\"{}\" stroke=\"none\" fill-rule=\"nonzero\"/>",
            path, fill
        );
        return;
    }

    for set in &d.sets {
        let path = ops_to_path(set);
        let stroke = format!(
            "rgba({},{},{},{})",
            (d.options.stroke_color.r * 255.0) as u8,
            (d.options.stroke_color.g * 255.0) as u8,
            (d.options.stroke_color.b * 255.0) as u8,
            d.options.stroke_color.a
        );

        let fill = match d.options.fill_color {
            Some(c) => format!(
                "rgba({},{},{},{})",
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
                c.a
            ),
            None => "none".to_string(),
        };

        let _ = write!(
            svg,
            "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
            path, fill, stroke, d.options.stroke_width
        );
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn ops_to_path(ops: &[Op]) -> String {
    let mut path = String::new();
    for op in ops {
        match op {
            Op::MoveTo(x, y) => {
                let _ = write!(path, "M{:.2} {:.2} ", x, y);
            }
            Op::LineTo(x, y) => {
                let _ = write!(path, "L{:.2} {:.2} ", x, y);
            }
            Op::QuadraticTo(cx, cy, x, y) => {
                let _ = write!(path, "Q{:.2} {:.2} {:.2} {:.2} ", cx, cy, x, y);
            }
            Op::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                let _ = write!(
                    path,
                    "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                    c1x, c1y, c2x, c2y, x, y
                );
            }
            Op::Close => path.push('Z'),
        }
    }
    path.trim_end().to_string()
}
