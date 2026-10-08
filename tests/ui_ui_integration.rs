use gpui_excalidraw::ui::Editor;
use gpui_kit::{
    AppContext, Bounds, Entity, Point, TestAppContext, WindowBounds, WindowHandle, WindowOptions,
    px, size, test::TestWindowExt,
};

fn open_editor(cx: &mut TestAppContext) -> (WindowHandle<gpui_kit::base::Root>, Entity<Editor>) {
    open_editor_sized(cx, 800.0, 600.0)
}

fn open_editor_sized(
    cx: &mut TestAppContext,
    width: f32,
    height: f32,
) -> (WindowHandle<gpui_kit::base::Root>, Entity<Editor>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        let bounds = Bounds {
            origin: Point::default(),
            size: size(px(width), px(height)),
        };
        let (window, content) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |_, cx| {
                let mut editor = Editor::new();
                editor.add_demo_scene();
                cx.new(|_| editor)
            },
        )
        .expect("open test window");
        (
            window.downcast::<gpui_kit::base::Root>().expect("root"),
            content,
        )
    })
}

#[gpui_kit::test]
fn editor_renders_toolbar_and_canvas(cx: &mut TestAppContext) {
    let (window, _content) = open_editor(cx);
    cx.update_window(window.into(), |_, _window, cx| {
        _window.render_frame(cx);
    })
    .unwrap();
}

#[gpui_kit::test]
fn chrome_islands_are_laid_out_and_clickable(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        for id in [
            "menu",
            "tool-lock",
            "tool-handIcon",
            "tool-SelectionIcon",
            "tool-RectangleIcon",
            "tool-DiamondIcon",
            "tool-EllipseIcon",
            "tool-ArrowIcon",
            "tool-LineIcon",
            "tool-FreedrawIcon",
            "tool-TextIcon",
            "tool-stickyNoteToolIcon",
            "tool-EraserIcon",
            "more-tools",
            "library",
            "theme",
            "zoom-out",
            "zoom-in",
            "undo",
            "redo",
        ] {
            let snapshot = window.find(id);
            assert!(snapshot.visible(), "{id} is not visible");
            let bounds = snapshot.bounds();
            assert!(
                bounds.size.width >= px(20.0) && bounds.size.height >= px(20.0),
                "{id} is too small to hit: {bounds:?}"
            );
        }

        window.click("tool-lock", cx);
        content.update(cx, |editor, _| {
            assert!(editor.document.tool_locked, "the lock click never arrived");
        });
        window.click("tool-lock", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.document.tool_locked,
                "the lock click did not toggle"
            );
        });

        let before = content.read(cx).document.scene.len();
        window.click("tool-RectangleIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Rectangle);
            assert_eq!(
                editor.document.scene.len(),
                before,
                "the canvas reacted to a toolbar click"
            );
        });

        window.click("more-tools", cx);
        window.render_frame(cx);
        window.click("more-laserPointerToolIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Laser);
            assert!(!editor.is_more_tools_open(), "the popover should close");
        });

        window.click("tool-EllipseIcon", cx);
        let before = content.read(cx).document.scene.len();
        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(520.0), px(400.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                before + 1,
                "dragging on the canvas should add an ellipse"
            );
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "an unlocked tool hands back the selection tool"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn hamburger_menu_rows_are_clickable(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));

        window.render_frame(cx);
        window.click("fm-prefs", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));

        window.render_frame(cx);
        window.click("prefs-grid", cx);
        content.update(cx, |editor, _| {
            assert!(editor.document.show_grid, "the grid row did nothing");
            assert!(editor.is_menu_open(), "the submenu should stay open");
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn hamburger_menu_is_capped_and_scrolls(cx: &mut TestAppContext) {
    let height = 380.0;
    let (window, _content) = open_editor_sized(cx, 800.0, height);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        let menu = window.find("file-menu").bounds();

        let cap = height - 32.0 - 36.0;
        assert!(
            menu.size.height <= px(cap),
            "the menu should stop short of the window edge, got {menu:?}"
        );

        assert!(
            !window.find("fm-lang").visible(),
            "with the cap in place the language select starts below the fold"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn language_select_is_a_listbox_not_a_cycle(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Language;

    let (window, content) = open_editor_sized(cx, 800.0, 900.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        let before = content.read(cx).i18n.language();
        window.click("fm-lang", cx);
        content.update(cx, |editor, _| {
            assert!(editor.is_language_open(), "the select never expanded");
            assert_eq!(
                editor.i18n.language(),
                before,
                "opening the list changed the language"
            );
        });

        window.render_frame(cx);
        for language in Language::all() {
            let id = gpui_kit::SharedString::from(format!("lang-{}", language.code()));
            assert!(window.try_find(id.clone()).is_some(), "{id} is missing");
        }
        let panel = window.find("lang-list").bounds();
        assert!(
            panel.size.height <= px(900.0 - 32.0 - 36.0),
            "the list should stop short of the window edge, got {panel:?}"
        );
        assert!(
            window.find("lang-en").visible(),
            "the list should open on its first row"
        );

        window.click("lang-de-DE", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.i18n.language(), Language::De);
            assert!(!editor.is_language_open(), "the list should collapse");
            assert!(editor.is_menu_open(), "the menu should stay open");
        });

        window.render_frame(cx);
        window.click("fm-lang", cx);
        window.render_frame(cx);
        window.click("lang-en", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.i18n.language(), Language::En);
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn canvas_background_row_ends_with_the_custom_colour_trigger(cx: &mut TestAppContext) {
    let (window, content) = open_editor_sized(cx, 800.0, 900.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("menu", cx);
        window.render_frame(cx);

        assert!(
            window.try_find("viewbg-transparent").is_none(),
            "the canvas row must not offer a transparent preset"
        );
        assert!(
            window.try_find("viewbg-custom").is_some(),
            "the canvas row should end with the custom-colour trigger"
        );

        window.click("viewbg-f8f9fa", cx);
        assert_eq!(
            content.read(cx).document.view_background_color,
            "#f8f9fa",
            "picking a tint should set the canvas background"
        );

        window.render_frame(cx);
        window.click("viewbg-custom", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("color-hex-field").is_some(),
            "the trigger should open the hex field"
        );
        assert!(
            window.try_find("color-black").is_none(),
            "the canvas-background popup must not show the palette grid"
        );
        assert!(
            window.try_find("shade-0").is_none(),
            "the canvas-background popup must not show the shade ramp"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn shape_switch_retypes_a_lone_selected_element(cx: &mut TestAppContext) {
    use gpui_excalidraw::core::element::ElementType;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let (id, center) = {
            let editor = content.read(cx);
            let base = editor.document.scene.elements[2].base();
            (
                base.id.clone(),
                Point::new(
                    px((base.x + base.width / 2.0) as f32),
                    px((base.y + base.height / 2.0) as f32),
                ),
            )
        };
        window.drag(center, center, cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.selected_count(), 1, "the canvas click missed");
        });

        window.render_frame(cx);
        for control in [
            "shape-switch",
            "shape-rectangle",
            "shape-diamond",
            "shape-ellipse",
        ] {
            assert!(window.find(control).visible(), "{control} did not paint");
        }
        assert!(
            window.try_find("shape-line").is_none(),
            "a diamond must not be offered a line button"
        );

        let version = content
            .read(cx)
            .document
            .scene
            .get(&id)
            .unwrap()
            .base()
            .version;
        window.click("shape-diamond", cx);
        content.update(cx, |editor, _| {
            let element = editor.document.scene.get(&id).unwrap();
            assert_eq!(element.kind(), ElementType::Diamond);
            assert_eq!(element.base().version, version, "a no-op still bumped");
        });

        window.render_frame(cx);
        window.click("shape-rectangle", cx);
        content.update(cx, |editor, _| {
            let element = editor.document.scene.get(&id).unwrap();
            assert_eq!(
                element.kind(),
                ElementType::Rectangle,
                "the click missed the button"
            );
            assert!(
                element.base().version > version,
                "the conversion was not recorded"
            );
        });
        window.press("ctrl-z", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.get(&id).unwrap().kind(),
                ElementType::Diamond,
                "undo did not restore the shape"
            );
        });

        let ids: Vec<String> = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .map(|e| e.id().to_string())
            .collect();
        content.update(cx, |editor, _| {
            editor.document.select_many(ids[..2].to_vec())
        });
        window.render_frame(cx);
        assert!(
            window.try_find("shape-switch").is_none(),
            "multi-select kept the switcher"
        );

        let text_id = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .find(|e| e.kind() == ElementType::Text)
            .map(|e| e.id().to_string())
            .expect("the demo scene holds a text element");
        content.update(cx, |editor, _| editor.document.select(&text_id));
        window.render_frame(cx);
        assert!(
            window.try_find("shape-switch").is_none(),
            "text got a shape switcher"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn canvas_click_on_a_locked_element_offers_an_unlock_bubble(cx: &mut TestAppContext) {
    use gpui_excalidraw::core::element::{Element, ElementBase, ElementType};

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let mut base = ElementBase::new("unlock-target".to_string(), ElementType::Rectangle);
        base.x = 300.0;
        base.y = 250.0;
        base.width = 150.0;
        base.height = 100.0;
        content.update(cx, |editor, _| {
            editor.document.scene.add(Element::Rectangle { base });

            let other = editor.document.scene.elements[1].id().to_string();
            editor
                .document
                .select_many(vec!["unlock-target".to_string(), other]);
        });

        window.render_frame(cx);
        window.click("arrange-lock", cx);
        content.update(cx, |editor, _| {
            assert!(
                editor
                    .document
                    .scene
                    .get("unlock-target")
                    .unwrap()
                    .base()
                    .locked,
                "the panel's padlock never locked the selection"
            );
        });

        window.render_frame(cx);
        let centre = Point::new(px(375.0), px(300.0));
        window.drag(centre, centre, cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.selected_count(),
                0,
                "a locked element should not become the selection directly"
            );
        });

        window.render_frame(cx);
        let popup = window.find("unlock-popup").bounds();
        let bottom_edge = popup.origin.y + popup.size.height;
        assert!(
            bottom_edge <= px(250.0) && bottom_edge >= px(230.0),
            "the padlock should sit just above the element's top edge, got {popup:?}"
        );

        window.click("unlock-popup", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor
                    .document
                    .scene
                    .get("unlock-target")
                    .unwrap()
                    .base()
                    .locked,
                "the bubble did not unlock the element"
            );
            assert_eq!(editor.selected_count(), 1);
            assert!(editor.document.selected.contains("unlock-target"));
        });
        window.render_frame(cx);
        assert!(
            window.try_find("unlock-popup").is_none(),
            "the bubble should clear once it has done its job"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn canvas_buttons_track_a_generated_iframe(cx: &mut TestAppContext) {
    use gpui_excalidraw::core::element::{
        Element, ElementBase, ElementType, FrameElement, IframeElement,
    };

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let mut base = ElementBase::new("gen-iframe".to_string(), ElementType::Iframe);
        base.x = 300.0;
        base.y = 250.0;
        base.width = 200.0;
        base.height = 150.0;
        base.link = Some("https://example.com/embed".to_string());
        base.custom_data = Some(serde_json::json!({ "generationData": { "status": "done" } }));
        content.update(cx, |editor, _| {
            editor
                .document
                .scene
                .add(Element::Iframe(IframeElement { base }));
            editor.document.select("gen-iframe");
        });

        window.render_frame(cx);
        assert!(
            window.find("canvas-copyIcon").visible(),
            "a finished generated iframe should offer a copy button"
        );
        assert!(window.find("canvas-fullscreenIcon").visible());
        let island = window.find("canvas-buttons").bounds();

        assert!(
            (island.origin.x.to_f64() - 510.0).abs() < 1.0,
            "the island should hang off the right edge, got {island:?}"
        );
        assert!(
            (island.origin.y.to_f64() - 250.0).abs() < 1.0,
            "the island should start at the element's top edge, got {island:?}"
        );

        window.click("canvas-copyIcon", cx);

        content.update(cx, |editor, _| {
            editor
                .document
                .scene
                .get_mut("gen-iframe")
                .unwrap()
                .base_mut()
                .custom_data =
                Some(serde_json::json!({ "generationData": { "status": "pending" } }));
        });
        window.render_frame(cx);
        assert!(
            window.try_find("canvas-buttons").is_none(),
            "an unfinished generation must not get the island"
        );

        content.update(cx, |editor, _| {
            editor.document.scene.remove("gen-iframe");
            let mut base = ElementBase::new("magic".to_string(), ElementType::MagicFrame);
            base.x = 300.0;
            base.y = 250.0;
            base.width = 200.0;
            base.height = 150.0;
            editor
                .document
                .scene
                .add(Element::MagicFrame(FrameElement { base, name: None }));
            editor.document.select("magic");
        });
        window.render_frame(cx);
        assert!(window.find("canvas-MagicIcon").visible());
        assert!(
            window.try_find("canvas-copyIcon").is_none(),
            "a magic frame is not a copy-source element"
        );
    })
    .unwrap();

    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://example.com/embed"));
}

#[gpui_kit::test]
fn keyboard_shortcuts_survive_goroot_level_dispatch(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        window.press("r", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.tool,
                Tool::Rectangle,
                "the r shortcut never reached the editor"
            );
        });

        window.press("e", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Eraser, "the e shortcut is dead");
        });

        window.press("v", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Selection, "v is dead");
        });

        window.click("menu", cx);
        content.update(cx, |editor, _| assert!(editor.is_menu_open()));
        window.press("escape", cx);
        content.update(cx, |editor, _| {
            assert!(!editor.is_menu_open(), "escape did not reach the editor");
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn ctrl_z_undoes_through_the_real_dispatch_path(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        let before = content.read(cx).document.scene.len();

        window.click("tool-RectangleIcon", cx);
        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(500.0), px(380.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.scene.len(), before + 1);
            assert_eq!(editor.document.tool, Tool::Selection);
        });

        window.press("ctrl-z", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                before,
                "ctrl-z never reached the editor"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn ctrl_slash_filters_and_runs_the_command_palette(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        window.press("ctrl-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        window.input("rect", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.palette_query(), "rect");
            let results: Vec<&str> = editor
                .palette_results()
                .into_iter()
                .map(|(id, ..)| id)
                .collect();
            assert_eq!(
                results,
                vec!["tool-rectangle"],
                "the filter is not matching"
            );
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_palette_open(),
                "running a command should dismiss it"
            );
            assert_eq!(editor.document.tool, Tool::Rectangle);
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn palette_rows_are_clickable_over_the_backdrop(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("tool-RectangleIcon", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.document.tool, Tool::Rectangle)
        });

        window.click("menu", cx);
        window.render_frame(cx);
        window.click("fm-palette", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        window.render_frame(cx);
        window.click("palette-row-0", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "the row click missed"
            );
            assert!(!editor.is_palette_open());
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn palette_escape_dismisses_without_running(cx: &mut TestAppContext) {
    use gpui_excalidraw::ui::Tool;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_palette_open()));

        window.press("escape", cx);
        content.update(cx, |editor, _| {
            assert!(!editor.is_palette_open());
            assert_eq!(
                editor.document.tool,
                Tool::Selection,
                "escape ran a command"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn save_raises_the_export_dialog(cx: &mut TestAppContext) {
    let (window, content) = open_editor_sized(cx, 1000.0, 800.0);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("save-dialog-backdrop").is_none(),
            "the dialog must not be up before it is asked for"
        );

        window.click("menu", cx);
        window.render_frame(cx);
        window.click("fm-save", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            assert!(editor.save_dialog_open(), "the menu row raised nothing");
            let untitled = editor.i18n.t("labels.untitled");

            assert!(
                editor.project_name_value().starts_with(&untitled),
                "the name field should show App.getName()'s fallback, got {:?}",
                editor.project_name_value()
            );
        });

        assert!(window.find("save-dialog-backdrop").visible());
        let field = window.find("save-filename").bounds();
        assert!(
            field.size.width > px(120.0) && field.size.width < px(240.0),
            "the name field is the wrong width: {field:?}"
        );

        let field_centre = field.origin.x + field.size.width / 2.0;
        assert!(
            (field_centre - px(500.0)).abs() < px(12.0),
            "the dialog is not centred: {field:?}"
        );
        let button = window.find("save-card-button").bounds();
        assert!(
            button.size.width > px(60.0) && button.size.width < px(240.0),
            "the card's button should hug its label: {button:?}"
        );
        assert!(
            window.find("save-card-button").visible(),
            "the card's button is off-screen"
        );

        let seeded = content.update(cx, |editor, _| editor.project_name_value());
        window.input("x", cx);
        content.update(cx, |editor, _| {
            assert!(
                editor.project_name_value().ends_with('x'),
                "typing did not reach the name field: {:?}",
                editor.project_name_value()
            );
        });
        assert_eq!(
            window.find("save-filename").value(),
            Some(format!("{seeded}x").as_str()),
            "the field did not repaint after a keystroke"
        );

        window.press("escape", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.save_dialog_open(),
                "escape did not dismiss the modal"
            );
        });

        window.press("ctrl-s", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(editor.save_dialog_open(), "Ctrl+S raised nothing");
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn ctrl_f_finds_and_cycles_text_matches(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-f", cx);
        content.update(cx, |editor, _| assert!(editor.is_find_open()));

        window.input("e", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_query(), "e");
            assert_eq!(editor.find_match_count(), 2);
            assert_eq!(editor.find_index(), 0);
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_index(), 1, "enter did not step")
        });

        window.press("enter", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.find_index(), 0, "the stepper should wrap");
        });

        window.press("escape", cx);
        content.update(cx, |editor, _| assert!(!editor.is_find_open()));
    })
    .unwrap();
}

#[gpui_kit::test]
fn help_dialog_opens_on_question_mark(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("shift-/", cx);
        content.update(cx, |editor, _| assert!(editor.is_help_open()));

        window.render_frame(cx);
        assert!(
            window.find("help-close").visible(),
            "the help dialog did not paint"
        );

        window.press("escape", cx);
        content.update(cx, |editor, _| assert!(!editor.is_help_open()));
    })
    .unwrap();
}

#[gpui_kit::test]
fn png_export_from_the_editor_is_a_real_png(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let bytes = content
            .update(cx, |editor, _| editor.export_png_bytes(2.0))
            .expect("export should succeed");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a PNG signature");

        let path = std::env::temp_dir().join("excalidraw-ui-test-export.png");
        let path_string = path.to_string_lossy().into_owned();
        content
            .update(cx, |editor, _| editor.export_png_to(&path_string))
            .expect("writing the PNG should succeed");
        let written = std::fs::read(&path).expect("the exported file should exist");
        assert_eq!(&written[..8], b"\x89PNG\r\n\x1a\n");
        let _ = std::fs::remove_file(&path);
    })
    .unwrap();
}

#[gpui_kit::test]
fn eye_dropper_picks_the_colour_under_the_pointer(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        let before = content
            .read(cx)
            .document
            .scene
            .get(&target)
            .unwrap()
            .base()
            .stroke_color
            .clone();
        let expected = content.read(cx).export_background().to_ascii_lowercase();

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        window.click("eye-dropper-stroke", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                editor.is_eye_dropper_active(),
                "the trigger never armed the picker"
            );
        });
        assert!(
            window.find("eye-dropper").visible(),
            "the backdrop did not paint"
        );

        window.click_at("eye-dropper", Point::new(px(400.0), px(275.0)), cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_eye_dropper_active(),
                "clicking should end the pick"
            );
            let stroke = &editor
                .document
                .scene
                .get(&target)
                .unwrap()
                .base()
                .stroke_color;
            assert_ne!(stroke, &before, "the picked colour never landed");
            assert_eq!(
                stroke.to_ascii_lowercase(),
                expected,
                "the pick should have sampled the canvas background"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn escape_abandons_an_eye_dropper_pick(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);
        let before = content
            .read(cx)
            .document
            .scene
            .get(&target)
            .unwrap()
            .base()
            .stroke_color
            .clone();

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        window.click("eye-dropper-stroke", cx);
        content.update(cx, |editor, _| assert!(editor.is_eye_dropper_active()));

        window.press("escape", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert!(
                !editor.is_eye_dropper_active(),
                "escape should cancel the pick"
            );
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                before,
                "a cancelled pick must not write a colour"
            );
        });
        assert!(
            window.try_find("eye-dropper").is_none(),
            "the backdrop should be torn down"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn arrow_shortcut_shows_a_hint_beside_the_pointer(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        window.drag(
            Point::new(px(400.0), px(300.0)),
            Point::new(px(400.0), px(300.0)),
            cx,
        );

        window.press("a", cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.cursor_hint_icon(),
                Some("sharpArrowIcon"),
                "the letter shortcut should hint the arrow tool"
            );
        });
        assert!(
            window.find("cursor-hint").visible(),
            "the hint did not paint"
        );

        window.drag(
            Point::new(px(420.0), px(320.0)),
            Point::new(px(420.0), px(320.0)),
            cx,
        );
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.cursor_hint_icon(),
                None,
                "a press should hide the hint"
            );
        });

        window.press("5", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.cursor_hint_icon(), Some("sharpArrowIcon"));
        });
    })
    .unwrap();

    cx.executor()
        .advance_clock(std::time::Duration::from_millis(900));
    cx.run_until_parked();
    let icon = cx
        .update_window(window.into(), |_, _, cx| {
            content.read(cx).cursor_hint_icon()
        })
        .unwrap();
    assert_eq!(icon, None, "the hint should auto-hide");
}

#[gpui_kit::test]
fn external_file_drag_shows_the_overlay_and_a_scene_drop_replaces(cx: &mut TestAppContext) {
    use std::path::PathBuf;

    use gpui_kit::{ExternalPaths, FileDropEvent, PlatformInput};

    let (window, content) = open_editor(cx);

    let path = std::env::temp_dir().join("gpui-excalidraw-ui-drop-test.excalidraw");
    let path_string = path.to_string_lossy().into_owned();
    cx.update(|cx| {
        content
            .update(cx, |editor, _| editor.save_to(&path_string))
            .expect("exporting the demo scene should succeed");
    });
    let saved_len = cx.update(|cx| content.read(cx).document.scene.len());

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let paths = ExternalPaths([PathBuf::from(&path_string)].into_iter().collect());
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Entered {
                position: Point::new(px(400.0), px(300.0)),
                paths,
            }),
            cx,
        );
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.file_drag_kind(),
                Some("scene"),
                "a .excalidraw file is a scene"
            );
        });
        assert!(window.find("file-drop-overlay").visible());

        window.dispatch_event(PlatformInput::FileDrop(FileDropEvent::Exited), cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| assert_eq!(editor.file_drag_kind(), None));
        assert!(window.try_find("file-drop-overlay").is_none());

        content.update(cx, |editor, _| editor.document.scene.elements.clear());
        window.render_frame(cx);

        let paths = ExternalPaths([PathBuf::from(&path_string)].into_iter().collect());
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Entered {
                position: Point::new(px(400.0), px(300.0)),
                paths,
            }),
            cx,
        );
        window.render_frame(cx);
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Submit {
                position: Point::new(px(400.0), px(300.0)),
            }),
            cx,
        );
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.document.scene.len(),
                saved_len,
                "the drop did not load the file"
            );
            let message = editor
                .toast_message()
                .expect("the replace should report itself");
            assert!(
                message.contains("Content replaced"),
                "unexpected toast text: {message}"
            );
        });
        assert!(window.find("toast").visible(), "the toast did not paint");

        window.click("toast-close", cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.toast_message(), None, "the close button did nothing");
        });
    })
    .unwrap();

    let _ = std::fs::remove_file(&path);
}

#[gpui_kit::test]
fn colour_picker_popup_writes_the_clicked_swatch(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        assert!(
            window.try_find("color-red").is_none(),
            "the palette should not be painted before the trigger is clicked"
        );

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        assert!(
            window.find("color-red").visible(),
            "the trigger never opened the palette"
        );
        assert!(
            window.find("color-hex-field").visible(),
            "the hex row is missing"
        );
        assert!(
            window.find("shade-0").visible(),
            "a ramped colour should offer its ramp"
        );

        window.click("color-red", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                "#e03131",
                "clicking a swatch should write that colour"
            );
        });

        window.click("shade-0", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor
                    .document
                    .scene
                    .get(&target)
                    .unwrap()
                    .base()
                    .stroke_color,
                "#fff5f5",
                "stepping the ramp should move the whole grid to that shade"
            );
        });

        window.click("color-trigger-stroke", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("color-red").is_none(),
            "clicking the trigger again should close the popup"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn font_picker_popup_searches_and_selects(cx: &mut TestAppContext) {
    use gpui_excalidraw::core::element::Element;

    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let text_id = content
            .read(cx)
            .document
            .scene
            .elements
            .iter()
            .find_map(|element| match element {
                Element::Text(_) => Some(element.id().to_string()),
                _ => None,
            })
            .expect("the demo scene has a text element");
        content.update(cx, |editor, _| editor.document.select(&text_id));
        window.render_frame(cx);

        assert!(
            window.try_find("font-search").is_none(),
            "the list starts closed"
        );
        window.click("font-trigger", cx);
        window.render_frame(cx);
        assert!(
            window.find("font-search").visible(),
            "the trigger did not open the list"
        );
        assert!(
            window.find("font-item-Comic Shanns").visible(),
            "every family should be listed"
        );

        window.input("comic", cx);
        window.render_frame(cx);
        assert!(window.find("font-item-Comic Shanns").visible());
        assert!(
            window.try_find("font-item-Nunito").is_none(),
            "a needle should hide the families it does not match"
        );

        window.click("font-item-Comic Shanns", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            let family = match editor.document.scene.get(&text_id).unwrap() {
                Element::Text(text) => text.font_family,
                _ => panic!("the text element changed type"),
            };
            assert_eq!(
                family,
                gpui_excalidraw::core::types::FontFamily::ComicShanns,
                "clicking a row should set the family"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn element_link_dialog_writes_and_clears_the_link(cx: &mut TestAppContext) {
    let (window, content) = open_editor(cx);

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let target = content.read(cx).document.scene.elements[0].id().to_string();
        content.update(cx, |editor, _| editor.document.select(&target));
        window.render_frame(cx);

        assert!(
            window.try_find("link-input").is_none(),
            "the dialog starts closed"
        );
        window.click("single-link", cx);
        window.render_frame(cx);
        assert!(
            window.find("link-input").visible(),
            "the control did not open the dialog"
        );

        assert!(window.try_find("link-remove").is_none());

        window.input("example.com", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.link_input(),
                "example.com",
                "typing should reach the field"
            );
        });

        window.click("link-confirm", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(editor.selected_link().as_deref(), Some("example.com"));
        });
        assert!(
            window.try_find("link-input").is_none(),
            "confirming should dismiss the dialog"
        );

        window.click("single-link", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.link_input(),
                "example.com",
                "the field should be seeded"
            );
        });
        window.click("link-remove", cx);
        window.render_frame(cx);
        window.click("link-confirm", cx);
        window.render_frame(cx);
        content.update(cx, |editor, _| {
            assert_eq!(
                editor.selected_link(),
                None,
                "confirming an empty field clears the link"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn ctrl_v_pastes_clipboard_text_as_text_elements(cx: &mut TestAppContext) {
    use gpui_excalidraw::core::element::Element;

    let (window, content) = open_editor(cx);

    cx.update(|cx| {
        cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(
            "first\n\nsecond".to_string(),
        ));
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("ctrl-v", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            let bodies: Vec<String> = editor
                .document
                .selected_elements()
                .iter()
                .filter_map(|element| match element {
                    Element::Text(text) => Some(text.text.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(
                bodies,
                vec!["first".to_string(), "second".to_string()],
                "the paste should make one text element per non-blank line"
            );
        });
    })
    .unwrap();
}

#[gpui_kit::test]
fn ctrl_v_while_editing_pastes_into_the_text_element(cx: &mut TestAppContext) {
    use gpui_excalidraw::{core::element::Element, ui::Tool};

    let (window, content) = open_editor(cx);

    cx.update(|cx| {
        cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string("pasteme".to_string()));
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);

        let id = content.update(cx, |editor, _| {
            editor.set_tool(Tool::Text);
            editor.on_canvas_mouse_down(400.0, 300.0, gpui_kit::Modifiers::default(), 1);
            editor.editing_text_id().map(str::to_string)
        });
        let id = id.expect("the text tool should open a new element for editing");

        window.press("ctrl-v", cx);
        window.render_frame(cx);

        content.update(cx, |editor, _| {
            let element = editor
                .document
                .scene
                .get(&id)
                .expect("the text element vanished");
            match element {
                Element::Text(text) => assert_eq!(text.text, "pasteme"),
                _ => panic!("the paste changed the element type"),
            }
        });
    })
    .unwrap();
}
