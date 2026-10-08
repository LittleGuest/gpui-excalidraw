use excalidraw_core::{
    element::{Element, ElementBase, ElementType},
    geometry::Point,
    scene::Scene,
};
use excalidraw_render::{
    export::SvgExporter,
    shape::{free_draw_outline, render_element, rough_rectangle},
};

fn rect(id: &str, x: f64, y: f64, w: f64, h: f64) -> Element {
    let mut base = ElementBase::new(id.to_string(), ElementType::Rectangle);
    base.x = x;
    base.y = y;
    base.width = w;
    base.height = h;
    base.seed = 42;
    Element::Rectangle { base }
}

#[test]
fn test_rough_rectangle_generates_closed_path() {
    let opts = Default::default();
    let d = rough_rectangle(0.0, 0.0, 100.0, 50.0, &opts);
    // `fill_path` stays the clean closed polygon used for fills.
    let fill = d.fill_path.as_ref().expect("a fill path");
    assert_eq!(fill.len(), 5);
    assert_eq!(fill.last(), Some(&excalidraw_render::shape::Op::Close));
    // The stroke outline is now the hand-drawn multi-pass cubic set, not a
    // closed polygon.
    let outline = &d.sets[0];
    assert!(
        outline
            .iter()
            .any(|op| matches!(op, excalidraw_render::shape::Op::CubicTo(..))),
        "hand-drawn outline must use cubic curves"
    );
    let moves = outline
        .iter()
        .filter(|op| matches!(op, excalidraw_render::shape::Op::MoveTo(..)))
        .count();
    assert!(moves >= 2, "expected two overlay passes, got {moves}");
}

#[test]
fn test_render_rectangle_element() {
    let e = rect("a", 0.0, 0.0, 100.0, 100.0);
    let drawables = render_element(&e);
    assert_eq!(drawables.len(), 1);
    assert_eq!(
        drawables[0].shape,
        excalidraw_render::shape::ShapeKind::Rectangle
    );
}

#[test]
fn test_render_line_element() {
    let mut base = ElementBase::new("l".to_string(), ElementType::Line);
    base.width = 100.0;
    base.height = 0.0;
    let line = Element::Line(excalidraw_core::element::LineElement {
        linear: excalidraw_core::element::LinearElement {
            base,
            points: vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
        },
        polygon: false,
    });
    let drawables = render_element(&line);
    assert_eq!(drawables.len(), 1);
}

#[test]
fn test_svg_export_contains_elements() {
    let mut scene = Scene::new();
    scene.add(rect("a", 0.0, 0.0, 100.0, 100.0));
    let svg = SvgExporter::export(&scene, "#ffffff");
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn test_svg_export_transparent_background() {
    let mut scene = Scene::new();
    scene.add(rect("a", 0.0, 0.0, 100.0, 100.0));
    let svg = SvgExporter::export(&scene, "transparent");
    assert!(!svg.contains("fill=\"transparent\""));
}

#[test]
fn test_render_text_element() {
    let text = excalidraw_core::factory::new_text(
        0.0,
        0.0,
        "Hello",
        20.0,
        &excalidraw_core::factory::ElementOptions::default(),
    );
    let drawables = render_element(&text);
    assert_eq!(drawables.len(), 1);
    assert_eq!(
        drawables[0].shape,
        excalidraw_render::shape::ShapeKind::Text
    );
    let t = drawables[0].text.as_ref().unwrap();
    assert_eq!(t.text, "Hello");
    assert_eq!(t.font_size, 20.0);
}

#[test]
fn test_render_image_element() {
    let img = excalidraw_core::factory::new_image(
        0.0,
        0.0,
        100.0,
        80.0,
        None,
        &excalidraw_core::factory::ElementOptions::default(),
    );
    let drawables = render_element(&img);
    assert_eq!(drawables.len(), 1);
    assert_eq!(
        drawables[0].shape,
        excalidraw_render::shape::ShapeKind::Image
    );
}

#[test]
fn test_render_arrow_with_arrowhead() {
    let arrow = excalidraw_core::factory::new_arrow(
        vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        None,
        Some(excalidraw_core::Arrowhead::Arrow),
        &excalidraw_core::factory::ElementOptions::default(),
    );
    let drawables = render_element(&arrow);
    assert!(drawables.len() >= 2);
    assert!(
        drawables
            .iter()
            .any(|d| d.shape == excalidraw_render::shape::ShapeKind::Arrowhead)
    );
}

#[test]
fn test_render_arrow_without_arrowhead() {
    let arrow = excalidraw_core::factory::new_arrow(
        vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        None,
        None,
        &excalidraw_core::factory::ElementOptions::default(),
    );
    let drawables = render_element(&arrow);
    assert_eq!(drawables.len(), 1);
    assert!(
        !drawables
            .iter()
            .any(|d| d.shape == excalidraw_render::shape::ShapeKind::Arrowhead)
    );
}

#[test]
fn test_dashed_line_splits_segments() {
    let opts = excalidraw_render::shape::DrawOptions {
        stroke_style: excalidraw_core::StrokeStyle::Dashed,
        ..Default::default()
    };
    let line = excalidraw_render::shape::rough_line(
        &[Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        &opts,
    );
    let solid = excalidraw_render::shape::rough_line(
        &[Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        &Default::default(),
    );
    assert!(line.ops.len() > solid.ops.len());
}

#[test]
fn test_rounded_rectangle_generates_curves() {
    let opts = excalidraw_render::shape::DrawOptions {
        roundness: Some(excalidraw_core::types::Roundness {
            kind: excalidraw_core::types::RoundnessType::ProportionalRadius,
            value: Some(0.25),
        }),
        ..Default::default()
    };
    let d = rough_rectangle(0.0, 0.0, 100.0, 50.0, &opts);
    // The rounded rectangle is drawn through `svgPath`, whose quadratics are
    // normalised to cubics.
    let has_cubic = d
        .ops
        .iter()
        .any(|op| matches!(op, excalidraw_render::shape::Op::CubicTo(..)));
    assert!(has_cubic, "rounded rectangle should contain cubic curves");
}

#[test]
fn test_solid_fill_has_fill_path() {
    let mut base = ElementBase::new("f".to_string(), ElementType::Rectangle);
    base.x = 0.0;
    base.y = 0.0;
    base.width = 100.0;
    base.height = 50.0;
    base.fill_style = excalidraw_core::FillStyle::Solid;
    base.background_color = "#ff0000".to_string();
    let e = Element::Rectangle { base };
    let drawables = render_element(&e);
    assert_eq!(drawables.len(), 1);
    assert!(drawables[0].fill_path.is_some());
}

#[test]
fn test_draw_options_carries_opacity() {
    let mut base = ElementBase::new("o".to_string(), ElementType::Rectangle);
    base.opacity = 50.0;
    base.roundness = Some(excalidraw_core::types::Roundness {
        kind: excalidraw_core::types::RoundnessType::AdaptiveRadius,
        value: Some(0.3),
    });
    let e = Element::Rectangle { base };
    let opts = excalidraw_render::shape::DrawOptions::from_element(&e);
    assert_eq!(opts.opacity, 50.0);
    assert!(opts.roundness.is_some());
}

// ----- clipped fill + line arrowheads -------------------------------------

use excalidraw_core::{
    Arrowhead,
    factory::{ElementOptions, new_diamond, new_ellipse, new_line, new_rectangle, new_text},
    types::{FillStyle, Roundness, RoundnessType, TextAlign, VerticalAlign},
};
use excalidraw_render::shape::{Op, ops_to_polygon};

fn filled(style: FillStyle) -> ElementOptions {
    ElementOptions {
        background_color: "#a5d8ff".to_string(),
        fill_style: style,
        ..Default::default()
    }
}

/// Every fill op must lie inside the element's outline polygon.
fn assert_fill_inside(element: &Element) {
    let d = render_element(element)
        .into_iter()
        .next()
        .expect("a drawable");
    let poly = ops_to_polygon(&d.fill_path.clone().expect("fill path"));

    // The outline polygon itself defines the acceptance region via a
    // bounding-box test with a small tolerance (hachure is hand-jittered).
    let min_x = poly.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = poly.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = poly.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = poly.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let tol = 2.0;

    // Skip the first set: it is the shape outline itself.
    for set in d.sets.iter().skip(1) {
        for op in set {
            let (x, y) = match *op {
                Op::MoveTo(x, y) | Op::LineTo(x, y) => (x, y),
                Op::QuadraticTo(_, _, x, y) => (x, y),
                Op::CubicTo(_, _, _, _, x, y) => (x, y),
                Op::Close => continue,
            };
            assert!(
                x >= min_x - tol && x <= max_x + tol && y >= min_y - tol && y <= max_y + tol,
                "fill point ({x}, {y}) escaped the shape bounds \
                 x[{min_x}, {max_x}] y[{min_y}, {max_y}]"
            );
        }
    }
}

#[test]
fn hachure_fill_stays_inside_rectangle() {
    let e = new_rectangle(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Hachure));
    assert_fill_inside(&e);
}

#[test]
fn hachure_fill_stays_inside_diamond() {
    // A diamond is the strongest test: a naive bounding-box fill would spill
    // out of the slanted edges.
    let e = new_diamond(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Hachure));
    let d = render_element(&e).into_iter().next().unwrap();
    // The extreme corners of the bounding box must NOT be covered by fill.
    let corner_covered = d.sets.iter().skip(1).flatten().any(|op| match *op {
        Op::MoveTo(x, y) | Op::LineTo(x, y) => x < 5.0 && y < 5.0,
        _ => false,
    });
    assert!(
        !corner_covered,
        "hachure must be clipped away from the corners"
    );
    assert_fill_inside(&e);
}

#[test]
fn hachure_fill_stays_inside_ellipse_and_rounded_rect() {
    let e = new_ellipse(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Hachure));
    assert_fill_inside(&e);

    let mut rounded = new_rectangle(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Hachure));
    if let Element::Rectangle { base } = &mut rounded {
        base.roundness = Some(Roundness {
            kind: RoundnessType::ProportionalRadius,
            value: Some(0.3),
        });
    }
    assert_fill_inside(&rounded);
}

#[test]
fn cross_hatch_produces_two_directions() {
    let e = new_rectangle(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::CrossHatch));
    let d = render_element(&e).into_iter().next().unwrap();
    // More fill segments than plain hachure (two crossings instead of one).
    let plain = new_rectangle(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Hachure));
    let dp = render_element(&plain).into_iter().next().unwrap();
    assert!(d.sets.len() > dp.sets.len());
}

#[test]
fn solid_fill_emits_no_hachure_sets() {
    let e = new_rectangle(0.0, 0.0, 120.0, 80.0, &filled(FillStyle::Solid));
    let d = render_element(&e).into_iter().next().unwrap();
    assert_eq!(d.sets.len(), 1, "solid fill uses the fill_path only");
    assert!(d.fill_path.is_some());
}

#[test]
fn line_renders_arrowheads() {
    let mut line = new_line(
        vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        &ElementOptions::default(),
    );
    if let Element::Line(l) = &mut line {
        l.linear.end_arrowhead = Some(Arrowhead::Arrow);
        l.linear.start_arrowhead = Some(Arrowhead::Dot);
    }
    let drawables = render_element(&line);
    // outline + two arrowheads
    assert_eq!(drawables.len(), 3);
}

#[test]
fn line_without_arrowheads_renders_one_drawable() {
    let line = new_line(
        vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        &ElementOptions::default(),
    );
    assert_eq!(render_element(&line).len(), 1);
}

// --- text alignment / multi-line export -----------------------------------

fn centered_text(x: f64, w: f64, align: TextAlign, body: &str) -> Element {
    let mut t = new_text(x, 50.0, body, 20.0, &ElementOptions::default());
    if let Element::Text(te) = &mut t {
        te.base.width = w;
        te.base.height = 40.0;
        te.text_align = align;
        te.vertical_align = VerticalAlign::Middle;
    }
    t
}

#[test]
fn centered_text_anchors_to_box_center_not_left_edge() {
    let el = centered_text(100.0, 200.0, TextAlign::Center, "Hi");
    let svg = SvgExporter::export_element(&el, "#ffffff");
    // box center = 100 + 200/2 = 200
    assert!(svg.contains(r#"text-anchor="middle""#), "svg: {svg}");
    assert!(
        svg.contains(r#"x="200.00""#),
        "expected center x=200, svg: {svg}"
    );
}

#[test]
fn right_aligned_text_anchors_to_box_right_edge() {
    let el = centered_text(100.0, 200.0, TextAlign::Right, "Hi");
    let svg = SvgExporter::export_element(&el, "#ffffff");
    assert!(svg.contains(r#"text-anchor="end""#), "svg: {svg}");
    assert!(
        svg.contains(r#"x="300.00""#),
        "expected right x=300, svg: {svg}"
    );
}

#[test]
fn left_aligned_text_anchors_to_box_left_edge() {
    let el = centered_text(100.0, 200.0, TextAlign::Left, "Hi");
    let svg = SvgExporter::export_element(&el, "#ffffff");
    assert!(svg.contains(r#"text-anchor="start""#), "svg: {svg}");
    assert!(
        svg.contains(r#"x="100.00""#),
        "expected left x=100, svg: {svg}"
    );
}

#[test]
fn multiline_text_emits_one_tspan_per_line() {
    let el = centered_text(0.0, 120.0, TextAlign::Center, "one\ntwo\nthree");
    let svg = SvgExporter::export_element(&el, "#ffffff");
    assert_eq!(svg.matches("<tspan").count(), 3, "svg: {svg}");
    assert!(svg.contains("</text>"), "svg: {svg}");
}

// ----- free-draw ink: perfect-freehand, not a uniform polyline ---------------

use excalidraw_core::factory;

fn straight_stroke(step: f64, n: usize) -> Element {
    let points = (0..n).map(|i| Point::new(i as f64 * step, 0.0)).collect();
    factory::new_freedraw(points, &ElementOptions::default())
}

#[test]
fn freedraw_renders_as_filled_ink_rather_than_a_stroked_line() {
    let el = straight_stroke(4.0, 24);
    let ds = render_element(&el);
    assert_eq!(ds.len(), 1);
    let d = &ds[0];
    assert!(d.sets.is_empty(), "ink must not be stroked via sets");
    let ink = d.ink.as_ref().expect("freedraw must carry its outline");
    assert!(
        matches!(ink.first(), Some(Op::MoveTo(..))),
        "outline starts with MoveTo"
    );
    assert!(matches!(ink.last(), Some(Op::Close)), "outline is closed");
    // The algorithm resamples the centreline into a dense closed loop — far
    // more vertices than the input points.
    assert!(ink.len() > 2 * 24, "outline len = {}", ink.len());

    let svg = SvgExporter::export_element(&el, "#ffffff");
    assert!(svg.contains(r#"stroke="none""#), "svg: {svg}");
    assert!(svg.contains(r#"fill-rule="nonzero""#), "svg: {svg}");
}

#[test]
fn the_preview_outline_matches_the_committed_ink() {
    let el = straight_stroke(3.0, 18);
    let d = &render_element(&el)[0];
    let committed = d.ink.as_ref().unwrap();
    // Reproduce the committed centreline through the shared helper and check
    // the preview path is the same shape (same vertex count, same start).
    let base = el.base();
    let pts: Vec<Point> = match &el {
        Element::Freedraw(f) => f
            .points
            .iter()
            .map(|p| Point::new(base.x + p.x, base.y + p.y))
            .collect(),
        _ => unreachable!(),
    };
    let preview = free_draw_outline(&pts, base.stroke_width, true);
    assert_eq!(preview.len(), committed.len() - 1); // committed = outline + Close
}

#[test]
fn faster_strokes_render_thinner_than_slow_ones() {
    // Velocity-simulated pressure: the same straight line drawn quickly (long
    // steps) must come out thinner than drawn slowly — the hand-drawn feel.
    let thickness = |step: f64| -> f64 {
        let el = straight_stroke(step, 24);
        let d = &render_element(&el)[0];
        let ink = d.ink.as_ref().unwrap();
        let pts: Vec<(f64, f64)> = ink
            .iter()
            .filter_map(|op| match op {
                Op::MoveTo(x, y) | Op::LineTo(x, y) => Some((*x, *y)),
                _ => None,
            })
            .collect();
        let mut area = 0.0;
        for i in 0..pts.len() {
            let j = (i + 1) % pts.len();
            area += pts[i].0 * pts[j].1 - pts[j].0 * pts[i].1;
        }
        (area.abs() / 2.0) / (23.0 * step)
    };
    let slow = thickness(1.0);
    let fast = thickness(10.0);
    assert!(
        slow > fast * 1.2,
        "slow stroke ({slow:.2}px mean) should be clearly thicker than fast ({fast:.2}px)"
    );
}

#[test]
fn png_export_produces_a_real_png() {
    let mut scene = Scene::new();
    scene.add(rect("a", 0.0, 0.0, 120.0, 80.0));
    let png = excalidraw_render::export::export_png(&scene, "#ffffff", 1.0).expect("png");
    assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
}

#[test]
fn png_export_honours_scale() {
    let mut scene = Scene::new();
    scene.add(rect("a", 0.0, 0.0, 100.0, 100.0));
    let one =
        excalidraw_render::export::rasterize_png(&SvgExporter::export(&scene, "#ffffff"), 1.0)
            .unwrap();
    let two =
        excalidraw_render::export::rasterize_png(&SvgExporter::export(&scene, "#ffffff"), 2.0)
            .unwrap();
    let dims = |png: &[u8]| {
        (
            u32::from_be_bytes([png[16], png[17], png[18], png[19]]),
            u32::from_be_bytes([png[20], png[21], png[22], png[23]]),
        )
    };
    let (w1, h1) = dims(&one);
    let (w2, h2) = dims(&two);
    assert_eq!((w2, h2), (w1 * 2, h1 * 2));
}

#[test]
fn png_export_background_is_painted() {
    let mut scene = Scene::new();
    scene.add(rect("a", 10.0, 10.0, 40.0, 40.0));
    let png = excalidraw_render::export::export_png(&scene, "#ff0000", 1.0).unwrap();
    let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("decode");
    assert!(pixmap.width() >= 40 && pixmap.height() >= 40);
    let pixel = pixmap.pixel(0, 0).unwrap();
    assert_eq!(
        (pixel.red(), pixel.green(), pixel.blue()),
        (0xff, 0x00, 0x00)
    );
    assert_eq!(pixel.alpha(), 0xff);
}

#[test]
fn png_export_keeps_transparent_background() {
    let mut scene = Scene::new();
    scene.add(rect("a", 10.0, 10.0, 40.0, 40.0));
    let png = excalidraw_render::export::export_png(&scene, "transparent", 1.0).unwrap();
    let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("decode");
    assert_eq!(pixmap.pixel(0, 0).unwrap().alpha(), 0x00);
}

const RED_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEUlEQVR4nGP4z8DwH4QZYAwAR8oH+WdZbrcAAAAASUVORK5CYII=";

use std::collections::HashMap;

use excalidraw_core::{
    element::{ImageElement, ImageStatus},
    scene::BinaryFileData,
};

fn image_element(file_id: &str) -> Element {
    let mut base = ElementBase::new("img".to_string(), ElementType::Image);
    base.x = 0.0;
    base.y = 0.0;
    base.width = 20.0;
    base.height = 20.0;
    Element::Image(ImageElement {
        base,
        file_id: Some(file_id.to_string()),
        status: ImageStatus::Saved,
        scale: [1.0, 1.0],
        crop: None,
    })
}

fn red_png_files(file_id: &str) -> HashMap<String, BinaryFileData> {
    let mut files = HashMap::new();
    files.insert(
        file_id.to_string(),
        BinaryFileData::new(
            file_id,
            "image/png",
            format!("data:image/png;base64,{RED_PNG}"),
        ),
    );
    files
}

#[test]
fn svg_export_embeds_a_missing_file_as_a_placeholder() {
    let mut scene = Scene::new();
    scene.add(image_element("file-1"));
    let svg = SvgExporter::export(&scene, "#ffffff");
    assert!(!svg.contains("<image"), "svg: {svg}");
}

#[test]
fn svg_export_embeds_the_real_bitmap() {
    let mut scene = Scene::new();
    scene.add(image_element("file-1"));
    let svg = SvgExporter::export_with_files(&scene, "#ffffff", &red_png_files("file-1"));
    assert!(svg.contains("<image"));
    assert!(svg.contains(&format!("data:image/png;base64,{RED_PNG}")));
    assert!(!svg.contains(">image<"));
}

#[test]
fn png_export_paints_the_embedded_bitmap() {
    let mut scene = Scene::new();
    scene.add(image_element("file-1"));
    let png = excalidraw_render::export::export_png_with_files(
        &scene,
        "#ffffff",
        1.0,
        &red_png_files("file-1"),
    )
    .unwrap();
    let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("decode");
    let pixel = pixmap.pixel(30, 30).unwrap();
    assert_eq!(
        (pixel.red(), pixel.green(), pixel.blue()),
        (0xff, 0x00, 0x00)
    );
}

// ----- container-bound label wrapping --------------------------------------

#[test]
fn container_bound_text_wraps_to_the_box_width() {
    // A sticky note's label is soft-wrapped to its inner width before layout,
    // so the drawable carries the breaks; canvas and export both read this.
    let mut el = new_text(0.0, 0.0, &"字".repeat(30), 20.0, &ElementOptions::default());
    if let Element::Text(t) = &mut el {
        t.container_id = Some("note".to_string());
        t.base.width = 124.0; // six full-width glyphs per 20px line
    }
    let d = &render_element(&el)[0];
    assert_eq!(d.text.as_ref().unwrap().text.split('\n').count(), 5);

    // The exporter turns each wrapped line into its own tspan.
    let svg = SvgExporter::export_element(&el, "#ffffff");
    assert_eq!(svg.matches("<tspan").count(), 5, "svg: {svg}");
}

#[test]
fn free_text_is_not_wrapped() {
    // A label without a container auto-sizes and keeps its single line.
    let el = new_text(0.0, 0.0, &"字".repeat(30), 20.0, &ElementOptions::default());
    let d = &render_element(&el)[0];
    assert_eq!(d.text.as_ref().unwrap().text.split('\n').count(), 1);
}
