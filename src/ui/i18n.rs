use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    En,
    Id,
    De,
    Es,
    Eu,
    Fr,
    It,
    Nl,
    Pl,
    PtPt,
    PtBr,
    Ro,
    Sk,
    Sl,
    Sv,
    Tr,
    Ru,
    Uk,
    He,
    Ar,
    Fa,
    Ja,
    ZhCn,
    ZhTw,
    Ko,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Id => "id-ID",
            Self::De => "de-DE",
            Self::Es => "es-ES",
            Self::Eu => "eu-ES",
            Self::Fr => "fr-FR",
            Self::It => "it-IT",
            Self::Nl => "nl-NL",
            Self::Pl => "pl-PL",
            Self::PtPt => "pt-PT",
            Self::PtBr => "pt-BR",
            Self::Ro => "ro-RO",
            Self::Sk => "sk-SK",
            Self::Sl => "sl-SI",
            Self::Sv => "sv-SE",
            Self::Tr => "tr-TR",
            Self::Ru => "ru-RU",
            Self::Uk => "uk-UA",
            Self::He => "he-IL",
            Self::Ar => "ar-SA",
            Self::Fa => "fa-IR",
            Self::Ja => "ja-JP",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
            Self::Ko => "ko-KR",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Id => "Bahasa Indonesia",
            Self::De => "Deutsch",
            Self::Es => "Español",
            Self::Eu => "Euskara",
            Self::Fr => "Français",
            Self::It => "Italiano",
            Self::Nl => "Nederlands",
            Self::Pl => "Polski",
            Self::PtPt => "Português",
            Self::PtBr => "Português Brasileiro",
            Self::Ro => "Română",
            Self::Sk => "Slovenčina",
            Self::Sl => "Slovenščina",
            Self::Sv => "Svenska",
            Self::Tr => "Türkçe",
            Self::Ru => "Русский",
            Self::Uk => "Українська",
            Self::He => "עברית",
            Self::Ar => "العربية",
            Self::Fa => "فارسی",
            Self::Ja => "日本語",
            Self::ZhCn => "简体中文",
            Self::ZhTw => "繁體中文",
            Self::Ko => "한국어",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "id" | "id-ID" | "in" | "in-ID" => Self::Id,
            "de" | "de-DE" | "de-AT" | "de-CH" | "de-LI" | "de-LU" => Self::De,
            "es" | "es-ES" | "es-MX" | "es-AR" => Self::Es,
            "eu" | "eu-ES" => Self::Eu,
            "fr" | "fr-FR" | "fr-BE" | "fr-CA" | "fr-CH" | "fr-LU" => Self::Fr,
            "it" | "it-IT" | "it-CH" => Self::It,
            "nl" | "nl-NL" | "nl-BE" => Self::Nl,
            "pl" | "pl-PL" => Self::Pl,
            "pt-PT" => Self::PtPt,
            "pt" | "pt-BR" => Self::PtBr,
            "ro" | "ro-RO" => Self::Ro,
            "sk" | "sk-SK" => Self::Sk,
            "sl" | "sl-SI" => Self::Sl,
            "sv" | "sv-SE" => Self::Sv,
            "tr" | "tr-TR" => Self::Tr,
            "ru" | "ru-RU" => Self::Ru,
            "uk" | "uk-UA" => Self::Uk,
            "he" | "he-IL" | "iw" | "iw-IL" => Self::He,
            "ar" | "ar-SA" => Self::Ar,
            "fa" | "fa-IR" => Self::Fa,
            "ja" | "ja-JP" => Self::Ja,
            "zh-CN" | "zh" | "zh-Hans" => Self::ZhCn,
            "zh-TW" | "zh-HK" | "zh-Hant" => Self::ZhTw,
            "ko" | "ko-KR" => Self::Ko,
            _ => Self::En,
        }
    }

    pub fn all() -> [Self; 25] {
        [
            Self::En,
            Self::Id,
            Self::De,
            Self::Es,
            Self::Eu,
            Self::Fr,
            Self::It,
            Self::Nl,
            Self::Pl,
            Self::PtPt,
            Self::PtBr,
            Self::Ro,
            Self::Sk,
            Self::Sl,
            Self::Sv,
            Self::Tr,
            Self::Ru,
            Self::Uk,
            Self::He,
            Self::Ar,
            Self::Fa,
            Self::Ja,
            Self::ZhCn,
            Self::ZhTw,
            Self::Ko,
        ]
    }

    pub fn detect() -> Self {
        let tag = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default();
        Self::from_locale(&tag)
    }

    pub fn from_locale(tag: &str) -> Self {
        let tag = tag.split('.').next().unwrap_or_default();
        let tag = tag.split('@').next().unwrap_or_default();
        if tag.is_empty() || tag.eq_ignore_ascii_case("c") || tag.eq_ignore_ascii_case("posix") {
            return Self::En;
        }
        Self::from_code(&tag.replace('_', "-"))
    }

    pub fn native_name(self) -> &'static str {
        self.label()
    }
}

pub struct I18n {
    current: Arc<RwLock<Language>>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::detect())
    }
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self {
            current: Arc::new(RwLock::new(language)),
        }
    }

    pub fn language(&self) -> Language {
        *self.current.read().unwrap()
    }

    pub fn set_language(&self, language: Language) {
        *self.current.write().unwrap() = language;
    }

    pub fn t(&self, key: &str) -> String {
        let lang = self.language();
        let map = translations();
        if let Some(v) = map.get(lang.code()).and_then(|m| m.get(key)) {
            return v.to_string();
        }

        if let Some(v) = map.get("en").and_then(|m| m.get(key)) {
            return v.to_string();
        }
        key.to_string()
    }
}

pub fn translations() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut all = HashMap::new();
    all.insert("en", en());
    all.insert("id-ID", id_id());
    all.insert("de-DE", de());
    all.insert("es-ES", es());
    all.insert("eu-ES", eu_es());
    all.insert("fr-FR", fr());
    all.insert("it-IT", it());
    all.insert("nl-NL", nl_nl());
    all.insert("pl-PL", pl_pl());
    all.insert("pt-PT", pt_pt());
    all.insert("pt-BR", pt_br());
    all.insert("ro-RO", ro_ro());
    all.insert("sk-SK", sk_sk());
    all.insert("sl-SI", sl_si());
    all.insert("sv-SE", sv_se());
    all.insert("tr-TR", tr_tr());
    all.insert("ru-RU", ru());
    all.insert("uk-UA", uk_ua());
    all.insert("he-IL", he_il());
    all.insert("ar-SA", ar_sa());
    all.insert("fa-IR", fa_ir());
    all.insert("ja-JP", ja());
    all.insert("zh-CN", zh_cn());
    all.insert("zh-TW", zh_tw());
    all.insert("ko-KR", ko());

    for (code, strings) in save_dialog_strings() {
        if let Some(map) = all.get_mut(code) {
            map.extend(strings);
        }
    }
    all
}

const SAVE_DIALOG_KEYS: [&str; 5] = [
    "exportDialog.disk_title",
    "exportDialog.disk_details",
    "exportDialog.disk_button",
    "labels.fileTitle",
    "labels.untitled",
];

static SAVE_DIALOG: [(&str, [&str; 5]); 25] = [
    (
        "en",
        [
            "Save to disk",
            "Export the scene data to a file from which you can import later.",
            "Save to file",
            "File name",
            "Untitled",
        ],
    ),
    (
        "id-ID",
        [
            "Simpan ke disk",
            "Ekspor data pemandangan ke file yang mana Anda dapat impor nanti.",
            "Simpan ke file",
            "Nama file",
            "Tanpa judul",
        ],
    ),
    (
        "de-DE",
        [
            "Auf Festplatte speichern",
            "Exportiere die Zeichnungsdaten in eine Datei, aus der du später importieren kannst.",
            "Als Datei speichern",
            "Dateiname",
            "Unbenannt",
        ],
    ),
    (
        "es-ES",
        [
            "Guardar en disco",
            "Exportar los datos de la escena a un archivo desde el cual pueda importar más tarde.",
            "Guardar en archivo",
            "Nombre del archivo",
            "Sin título",
        ],
    ),
    (
        "eu-ES",
        [
            "Gorde diskoan",
            "Esportatu eszenaren datuak geroago inportatu ahal izango duzun fitxategi batan.",
            "Gorde fitxategian",
            "Fitxategi izena",
            "Izengabea",
        ],
    ),
    (
        "fr-FR",
        [
            "Enregistrer sur le disque",
            "Exporter les données de la scène comme un fichier que vous pourrez importer ultérieurement.",
            "Enregistrer comme fichier",
            "Nom du fichier",
            "Sans-titre",
        ],
    ),
    (
        "it-IT",
        [
            "Salva su disco",
            "Esporta i dati della scena su file, dal quale potrai importare in seguito.",
            "Salva su file",
            "Nome del file",
            "Senza titolo",
        ],
    ),
    (
        "nl-NL",
        [
            "Opslaan op schijf",
            "De scènegegevens exporteren naar een bestand waaruit u later kunt importeren.",
            "Opslaan naar bestand",
            "Bestandsnaam",
            "Naamloos",
        ],
    ),
    (
        "pl-PL",
        [
            "Zapisz na dysku",
            "Eksportuj dane sceny do pliku, z którego możesz importować później.",
            "Zapisz do pliku",
            "Nazwa pliku",
            "Bez tytułu",
        ],
    ),
    (
        "pt-PT",
        [
            "Guardar no disco",
            "Exportar os dados da cena para um ficheiro do qual poderá importar mais tarde.",
            "Guardar num ficheiro",
            "Nome do ficheiro",
            "Sem título",
        ],
    ),
    (
        "pt-BR",
        [
            "Salvar no computador",
            "Exportar os dados da cena para um arquivo que você poderá importar mais tarde.",
            "Salvar em um arquivo",
            "Nome do arquivo",
            "Sem título",
        ],
    ),
    (
        "ro-RO",
        [
            "Salvare pe disc",
            "Exportă datele scenei pe un fișier din care poți importa mai târziu.",
            "Salvare în fișier",
            "Nume de fișier",
            "Nedenumit",
        ],
    ),
    (
        "sk-SK",
        [
            "Uložiť na disk",
            "Exportovať údaje scény do súboru, z ktorého môžu byť neskôr importované.",
            "Uložiť do súboru",
            "Názov súboru",
            "Bez názvu",
        ],
    ),
    (
        "sl-SI",
        [
            "Shrani na disk",
            "Izvozite podatke scene v datoteko, iz katere jo lahko pozneje uvozite.",
            "Shrani v datoteko",
            "Ime datoteke",
            "Neimenovana",
        ],
    ),
    (
        "sv-SE",
        [
            "Spara till disk",
            "Exportera skissdata till en fil som du kan importera från senare.",
            "Spara till fil",
            "Filnamn",
            "Namnlös",
        ],
    ),
    (
        "tr-TR",
        [
            "Belleğe kaydet",
            "Sahne verilerini daha sonra içe aktarabileceğiniz bir dosyaya aktarın.",
            "Dosyaya kaydet",
            "Dosya adı",
            "Adsız",
        ],
    ),
    (
        "ru-RU",
        [
            "Сохранить на диск",
            "Экспортировать данные сцены в файл, из которого можно импортировать позже.",
            "Сохранить в файл",
            "Имя файла",
            "Безымянный",
        ],
    ),
    (
        "uk-UA",
        [
            "Зберегти на диск",
            "Експорт даних сцени в файл, з якого можна імпортувати пізніше.",
            "Зберегти до файлу",
            "Назва файла",
            "Без назви",
        ],
    ),
    (
        "he-IL",
        [
            "שמור לכונן",
            "ייצא מידע של הקנבאס לקובץ שתוכל לייבא אחר כך.",
            "שמירה לקובץ",
            "שם קובץ",
            "ללא כותרת",
        ],
    ),
    (
        "ar-SA",
        [
            "حفظ الملف على الجهاز",
            "تصدير بيانات المشهد إلى ملف يمكنك الاستيراد منه لاحقًا.",
            "حفظ إلى ملف",
            "اسم الملف",
            "غير معنون",
        ],
    ),
    (
        "fa-IR",
        [
            "ذخیره در دیسک",
            "داده های صحنه را به فایلی که بعداً می توانید از آن وارد کنید صادر کنید.",
            "ذخیره در فایل",
            "نام فایل",
            "بدون عنوان",
        ],
    ),
    (
        "ja-JP",
        [
            "ディスクに保存",
            "シーンデータを後からインポートできるファイルにエクスポートします。",
            "ファイルへ保存",
            "ファイル名",
            "無題",
        ],
    ),
    (
        "zh-CN",
        [
            "保存到本地",
            "将画布数据导出为文件，以便以后导入",
            "保存为文件",
            "文件名",
            "无标题",
        ],
    ),
    (
        "zh-TW",
        [
            "儲存至硬碟",
            "將場景匯出為可供匯入之檔案",
            "儲存至檔案",
            "檔案名稱",
            "無標題",
        ],
    ),
    (
        "ko-KR",
        [
            "디스크에 저장",
            "나중에 다시 불러올 수 있도록 화면 데이터를 내보냅니다.",
            "파일로 저장",
            "파일 이름",
            "제목 없음",
        ],
    ),
];

fn save_dialog_strings() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    SAVE_DIALOG
        .iter()
        .map(|(code, values)| {
            (
                *code,
                SAVE_DIALOG_KEYS
                    .iter()
                    .copied()
                    .zip(values.iter().copied())
                    .collect(),
            )
        })
        .collect()
}

fn en() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Select");
    m.insert("toolbar.hand", "Hand tool");
    m.insert("toolbar.rectangle", "Rectangle");
    m.insert("toolbar.diamond", "Diamond");
    m.insert("toolbar.ellipse", "Ellipse");
    m.insert("toolbar.arrow", "Arrow");
    m.insert("toolbar.line", "Line");
    m.insert("toolbar.freedraw", "Draw");
    m.insert("toolbar.text", "Text");
    m.insert("toolbar.image", "Insert image");
    m.insert("toolbar.frame", "Frame tool");
    m.insert("toolbar.eraser", "Eraser");
    m.insert("toolbar.laser", "Laser pointer");
    m.insert("toolbar.lock", "Lock");
    m.insert("toolbar.library", "Library");
    m.insert("action.undo", "Undo");
    m.insert("action.redo", "Redo");
    m.insert("action.delete", "Delete");
    m.insert("action.duplicate", "Duplicate");
    m.insert("action.exportPng", "Export as PNG");
    m.insert("action.exportSvg", "Export as SVG");
    m.insert("action.copyToClipboard", "Copy to clipboard");
    m.insert("action.zoomIn", "Zoom in");
    m.insert("action.zoomOut", "Zoom out");
    m.insert("action.resetZoom", "Reset zoom");
    m.insert("action.toggleTheme", "Toggle theme");
    m.insert("action.toggleGrid", "Toggle grid");
    m.insert("panel.layers", "Layers");
    m.insert("panel.properties", "Properties");
    m.insert("panel.library", "Library");
    m.insert("menu.language", "Language");

    m.insert("menu.open", "Open");
    m.insert("menu.save", "Save to…");
    m.insert("menu.exportImage", "Export image…");
    m.insert("menu.exportSvg", "Export to SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Toggle grid");
    m.insert("menu.layers", "Layers");
    m.insert("menu.reset", "Reset the canvas");
    m.insert("menu.help", "Help");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Open");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Group selection");
    m.insert("action.ungroup", "Ungroup selection");
    m.insert("ctx.duplicate", "Duplicate");
    m.insert("ctx.delete", "Delete");
    m.insert("ctx.copy", "Copy");
    m.insert("ctx.cut", "Cut");
    m.insert("ctx.paste", "Paste");
    m.insert("ctx.selectAll", "Select all");
    m.insert("ctx.bringFront", "Bring to front");
    m.insert("ctx.sendBack", "Send to back");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "None");
    m.insert("ah.arrow", "Arrow");
    m.insert("ah.bar", "Bar");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Triangle");
    m.insert("ah.diamond", "Diamond");
    m.insert("theme.light", "Light");
    m.insert("theme.dark", "Dark");
    m.insert("stats.elements", "Elements");
    m.insert(
        "hint.moveCanvas",
        "To move canvas, hold {{Scroll wheel}} or {{Space}} while dragging, or use the hand tool",
    );
    m.insert("library.addSelected", "Add selected");
    m.insert("library.empty", "No reusable items yet.");
    m.insert("color.stroke", "Stroke");
    m.insert("color.background", "Background");
    m.insert("prop.strokeWidth", "Stroke width");
    m.insert("prop.fillStyle", "Fill");
    m.insert("prop.opacity", "Opacity");
    m.insert("prop.roughness", "Roughness");
    m.insert("prop.fontSize", "Font size");
    m.insert("prop.fontFamily", "Font");
    m.insert("prop.textAlign", "Text align");
    m.insert("prop.strokeStyle", "Stroke style");
    m.insert("prop.arrowheads", "Arrowheads");
    m.insert("prop.roundness", "Roundness");
    m.insert("prop.lock", "Lock");
    m.insert("prop.unlock", "Unlock");
    m.insert("prop.group", "Group");
    m.insert("prop.ungroup", "Ungroup");
    m.insert("prop.align", "Align");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Align left");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Align right");
    m.insert("align.top", "Align top");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Align bottom");
    m.insert("fill.hachure", "Hachure");
    m.insert("fill.solid", "Solid");
    m.insert("fill.zigzag", "Zigzag");
    m.insert("fill.crossHatch", "Cross-hatch");
    m.insert("stroke.solid", "Solid");
    m.insert("stroke.dashed", "Dashed");
    m.insert("stroke.dotted", "Dotted");
    m.insert("round.none", "None");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Left");
    m.insert("textalign.center", "Center");
    m.insert("textalign.right", "Right");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Canvas background");
    m.insert("menu.followUs", "Follow us");
    m.insert("menu.discordGroup", "Discord chat");
    m.insert("menu.save", "Save to...");
    m.insert("menu.exportImage", "Export image...");
    m.insert("menu.collab", "Live collaboration...");
    m.insert("menu.commandPalette", "Command palette");
    m.insert("menu.findOnCanvas", "Find on canvas");
    m.insert("action.zoomToFit", "Zoom to fit");
    m.insert("palette.placeholder", "Type a command…");
    m.insert("palette.noResults", "No results found");
    m.insert("find.placeholder", "Find text on canvas");
    m.insert("find.noResults", "No results");
    m.insert("help.tools", "Tools");
    m.insert("help.actions", "Actions");
    m.insert("help.view", "View");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferences");
    m.insert("menu.theme", "Theme");
    m.insert("menu.toolLock", "Tool lock");
    m.insert("menu.snapToObjects", "Snap to objects");
    m.insert("menu.zenMode", "Zen mode");
    m.insert("menu.viewMode", "View mode");
    m.insert("menu.canvasStats", "Canvas & shape properties");
    m.insert("menu.arrowBinding", "Arrow binding");
    m.insert("menu.snapToMidpoints", "Snap to midpoints");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web Embed");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Lasso selection");
    m.insert("toolbar.textToDiagram", "Text to diagram");
    m.insert("toolbar.mermaid", "Mermaid to Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe to code");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Your drawings are saved in your browser storage.",
    );
    m.insert(
        "welcome.center2",
        "Your browser storage might be accidentally cleared.",
    );
    m.insert(
        "welcome.center3",
        "Save your work to a file from time to time to avoid losing it.",
    );
    m.insert("stats.title", "Properties");
    m.insert("stats.shapes", "Shapes");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Convert to code");
    m.insert("labels.copySource", "Copy source to clipboard");

    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );

    m.insert("colorPicker.colors", "Colors");
    m.insert("colorPicker.shades", "Shades");
    m.insert("colorPicker.noShades", "No shades available for this color");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Most used custom colors",
    );
    m.insert("colorPicker.hexCode", "Hex code");
    m.insert("fontList.sceneFonts", "In this scene");
    m.insert("fontList.availableFonts", "Available fonts");
    m.insert("fontList.empty", "No fonts found");
    m.insert("quickSearch.placeholder", "Quick search");
    m.insert("elementLink.title", "Link to object");
    m.insert(
        "elementLink.desc",
        "Click on a shape on canvas or paste a link.",
    );
    m.insert("buttons.remove", "Remove");
    m.insert("buttons.cancel", "Cancel");
    m.insert("buttons.confirm", "Confirm");
    m
}

fn zh_cn() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "选择");
    m.insert("toolbar.hand", "抓手工具");
    m.insert("toolbar.rectangle", "矩形");
    m.insert("toolbar.diamond", "菱形");
    m.insert("toolbar.ellipse", "椭圆");
    m.insert("toolbar.arrow", "箭头");
    m.insert("toolbar.line", "直线");
    m.insert("toolbar.freedraw", "画笔");
    m.insert("toolbar.text", "文本");
    m.insert("toolbar.image", "插入图像");
    m.insert("toolbar.frame", "画框工具");
    m.insert("toolbar.eraser", "橡皮擦");
    m.insert("toolbar.laser", "激光笔");
    m.insert("toolbar.lock", "锁定");
    m.insert("toolbar.library", "素材库");
    m.insert("action.undo", "撤销");
    m.insert("action.redo", "重做");
    m.insert("action.delete", "删除");
    m.insert("action.duplicate", "复制");
    m.insert("action.exportPng", "导出为 PNG");
    m.insert("action.exportSvg", "导出为 SVG");
    m.insert("action.copyToClipboard", "复制到剪贴板");
    m.insert("action.zoomIn", "放大");
    m.insert("action.zoomOut", "缩小");
    m.insert("action.resetZoom", "重置缩放");
    m.insert("action.toggleTheme", "切换主题");
    m.insert("action.toggleGrid", "切换网格");
    m.insert("panel.layers", "图层");
    m.insert("panel.properties", "属性");
    m.insert("panel.library", "素材库");
    m.insert("menu.language", "语言");
    m.insert("theme.light", "浅色");
    m.insert("theme.dark", "深色");
    m.insert("stats.elements", "元素");
    m.insert(
        "hint.moveCanvas",
        "移动画布：拖动时按住{{滚轮}}或{{空格}}，或使用抓手工具",
    );
    m.insert("library.addSelected", "添加选中项");
    m.insert("library.empty", "还没有可复用的素材。");
    m.insert("color.stroke", "描边");
    m.insert("color.background", "背景");
    m.insert("prop.strokeWidth", "线宽");
    m.insert("prop.fillStyle", "填充");
    m.insert("prop.opacity", "透明度");
    m.insert("prop.roughness", "粗糙度");
    m.insert("prop.fontSize", "字号");
    m.insert("prop.fontFamily", "字体");
    m.insert("prop.textAlign", "对齐");
    m.insert("prop.strokeStyle", "线型");
    m.insert("prop.arrowheads", "箭头");
    m.insert("prop.roundness", "圆角");
    m.insert("prop.lock", "锁定");
    m.insert("prop.unlock", "解锁");
    m.insert("prop.group", "分组");
    m.insert("prop.ungroup", "取消分组");
    m.insert("prop.align", "对齐");
    m.insert("prop.distribute", "分布");
    m.insert("align.left", "左对齐");
    m.insert("align.centerH", "水平居中");
    m.insert("align.right", "右对齐");
    m.insert("align.top", "顶部对齐");
    m.insert("align.centerV", "垂直居中");
    m.insert("align.bottom", "底部对齐");
    m.insert("fill.hachure", "影线");
    m.insert("fill.solid", "实心");
    m.insert("fill.zigzag", "锯齿");
    m.insert("fill.crossHatch", "交叉影线");
    m.insert("stroke.solid", "实线");
    m.insert("stroke.dashed", "虚线");
    m.insert("stroke.dotted", "点线");
    m.insert("round.none", "无");
    m.insert("round.small", "小");
    m.insert("round.medium", "中");
    m.insert("round.large", "大");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "黑体");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "漫画");
    m.insert("textalign.left", "左对齐");
    m.insert("textalign.center", "居中");
    m.insert("textalign.right", "右对齐");
    m.insert("welcome.title", "Excalidraw");
    m.insert("welcome.subtitle", "绘制手绘风格图表的虚拟白板");
    m.insert("menu.file", "菜单");
    m.insert("menu.new", "新建");
    m.insert("menu.save", "保存文件");
    m.insert("menu.load", "打开文件");
    m.insert("menu.exportSvg", "导出 SVG");
    m.insert("action.snap", "吸附网格");
    m.insert("action.group", "组合");
    m.insert("action.ungroup", "取消组合");
    m.insert("ctx.duplicate", "复制副本");
    m.insert("ctx.delete", "删除");
    m.insert("ctx.copy", "复制");
    m.insert("ctx.cut", "剪切");
    m.insert("ctx.paste", "粘贴");
    m.insert("ctx.selectAll", "全选");
    m.insert("ctx.bringFront", "置于顶层");
    m.insert("ctx.sendBack", "置于底层");
    m.insert("prop.start", "起点");
    m.insert("prop.end", "终点");
    m.insert("ah.none", "无");
    m.insert("ah.arrow", "箭头");
    m.insert("ah.bar", "竖线");
    m.insert("ah.dot", "圆点");
    m.insert("ah.triangle", "三角");
    m.insert("ah.diamond", "菱形");
    m.insert("menu.canvasBackground", "画布背景");
    m.insert("menu.followUs", "关注我们");
    m.insert("menu.discordGroup", "Discord 群组");
    m.insert("menu.open", "打开");
    m.insert("menu.exportImage", "导出图片");
    m.insert("menu.reset", "重置画布");
    m.insert("menu.help", "帮助");
    m.insert(
        "menu.helpHint",
        "拖动即可绘制。从工具栏选择工具，或按对应按键。",
    );
    m.insert("menu.layers", "图层");
    m.insert("menu.snap", "吸附到网格");
    m.insert("menu.grid", "切换网格显示");
    m.insert("menu.save", "保存到...");
    m.insert("menu.exportImage", "导出图片...");
    m.insert("menu.collab", "实时协作...");
    m.insert("menu.commandPalette", "命令面板");
    m.insert("menu.findOnCanvas", "在画布上查找");
    m.insert("action.zoomToFit", "缩放至适应");
    m.insert("palette.placeholder", "输入命令…");
    m.insert("palette.noResults", "未找到结果");
    m.insert("find.placeholder", "在画布上查找文本");
    m.insert("find.noResults", "无结果");
    m.insert("help.tools", "工具");
    m.insert("help.actions", "操作");
    m.insert("help.view", "视图");
    m.insert("help.files", "文件");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "首选项");
    m.insert("menu.theme", "主题");
    m.insert("menu.toolLock", "工具锁");
    m.insert("menu.snapToObjects", "吸附至对象");
    m.insert("menu.zenMode", "禅模式");
    m.insert("menu.viewMode", "查看模式");
    m.insert("menu.canvasStats", "画布与形状属性");
    m.insert("menu.arrowBinding", "箭头绑定");
    m.insert("menu.snapToMidpoints", "吸附到中点");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "嵌入网页");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "套索选择");
    m.insert("toolbar.textToDiagram", "文字至图表");
    m.insert("toolbar.mermaid", "Mermaid 至 Excalidraw");
    m.insert("toolbar.wireframeToCode", "线框图至代码");
    m.insert("toolbar.generate", "Generate");
    m.insert("welcome.center1", "您的绘图已保存在您的浏览器存储中。");
    m.insert("welcome.center2", "浏览器存储可能会被无意清除。");
    m.insert(
        "welcome.center3",
        "将您的工作定期保存到一个文件，以避免丢失。",
    );
    m.insert("stats.title", "属性");
    m.insert("stats.shapes", "形状");
    m.insert("stats.selectedType", "类型");
    m.insert("status.saved", "已保存");
    m.insert(
        "status.unavailable",
        "需要 Excalidraw 后端服务，当前构建不可用",
    );
    m.insert("labels.convertToCode", "转换成代码");
    m.insert("labels.copySource", "复制源码到剪贴板");
    m.insert("fileDrop.replace", "松开以替换内容");
    m.insert("fileDrop.add", "松开以添加到画布");
    m.insert("fileDrop.importLibrary", "松开以导入素材库");
    m.insert("fileDrop.replaceHint", "按住 {{Shift}} 以保留现有内容");
    m.insert("fileDrop.keepHint", "将保留你的现有内容");
    m.insert("fileDrop.libraryHint", "素材库文件将被追加");
    m.insert("fileDrop.replacedToast", "内容已替换。按 {{Ctrl+Z}} 撤销");
    m.insert("colorPicker.colors", "颜色");
    m.insert("colorPicker.shades", "色调明暗");
    m.insert("colorPicker.noShades", "此颜色没有可用的明暗变化");
    m.insert("colorPicker.mostUsedCustomColors", "常用自定义颜色");
    m.insert("colorPicker.hexCode", "十六进制值");
    m.insert("fontList.sceneFonts", "画布中的");
    m.insert("fontList.availableFonts", "可用字体");
    m.insert("fontList.empty", "未找到字体");
    m.insert("quickSearch.placeholder", "快速搜索");
    m.insert("elementLink.title", "链接到对象");
    m.insert("elementLink.desc", "点击画布上的形状或粘贴链接。");
    m.insert("buttons.remove", "删除");
    m.insert("buttons.cancel", "取消");
    m.insert("buttons.confirm", "确定");
    m
}

fn zh_tw() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "選取");
    m.insert("toolbar.rectangle", "矩形");
    m.insert("toolbar.diamond", "菱形");
    m.insert("toolbar.ellipse", "橢圓");
    m.insert("toolbar.arrow", "箭頭");
    m.insert("toolbar.line", "直線");
    m.insert("toolbar.freedraw", "畫筆");
    m.insert("toolbar.text", "文字");
    m.insert("toolbar.image", "插入圖片");
    m.insert("action.undo", "復原");
    m.insert("action.redo", "重做");
    m.insert("action.delete", "刪除");
    m.insert("action.exportPng", "匯出為 PNG");
    m.insert("action.exportSvg", "匯出為 SVG");
    m.insert("action.zoomIn", "放大");
    m.insert("action.zoomOut", "縮小");
    m.insert("action.toggleTheme", "切換主題");
    m.insert("menu.language", "語言");
    m.insert("theme.light", "淺色");
    m.insert("theme.dark", "深色");
    m.insert("menu.canvasBackground", "畫布背景");
    m.insert("menu.followUs", "追蹤我們");
    m.insert("menu.discordGroup", "Discord 群組");
    m.insert("action.group", "群組");
    m.insert("action.ungroup", "解除群組");
    m.insert("ctx.bringFront", "移到最前");
    m.insert("ctx.copy", "複製");
    m.insert("ctx.delete", "刪除");
    m.insert("ctx.duplicate", "建立副本");
    m.insert("ctx.paste", "貼上");
    m.insert("ctx.sendBack", "移到最後");
    m.insert(
        "hint.moveCanvas",
        "移動畫布：拖曳時按住{{滾輪}}或{{空白鍵}}，或使用抓手工具",
    );
    m.insert("library.addSelected", "加入圖庫");
    m.insert("library.empty", "圖庫是空的");
    m.insert("menu.exportImage", "匯出圖片");
    m.insert("menu.exportSvg", "匯出 SVG");
    m.insert("menu.grid", "切換網格顯示");
    m.insert("menu.help", "說明");
    m.insert(
        "menu.helpHint",
        "拖曳即可繪製。從工具列選擇工具，或按對應按鍵。",
    );
    m.insert("menu.layers", "圖層");
    m.insert("menu.open", "開啟");
    m.insert("menu.reset", "重設畫布");
    m.insert("menu.snap", "貼齊網格");
    m.insert("panel.layers", "圖層");
    m.insert("panel.library", "圖庫");
    m.insert("menu.save", "儲存到...");
    m.insert("menu.exportImage", "匯出圖片...");
    m.insert("menu.collab", "即時協作...");
    m.insert("menu.commandPalette", "命令面板");
    m.insert("menu.findOnCanvas", "在畫布上尋找");
    m.insert("toolbar.hand", "抓手工具");
    m.insert("toolbar.frame", "框架工具");
    m.insert("toolbar.eraser", "橡皮擦");
    m.insert("toolbar.laser", "雷射筆");
    m.insert("action.duplicate", "複製");
    m.insert("ctx.selectAll", "全選");
    m.insert("action.copyToClipboard", "複製到剪貼簿");
    m.insert("ctx.cut", "剪下");
    m.insert("action.resetZoom", "重設縮放");
    m.insert("action.toggleGrid", "切換網格");
    m.insert("action.zoomToFit", "縮放至適應");
    m.insert("palette.placeholder", "輸入指令…");
    m.insert("palette.noResults", "找不到結果");
    m.insert("find.placeholder", "在畫布上尋找文字");
    m.insert("find.noResults", "沒有結果");
    m.insert("help.tools", "工具");
    m.insert("help.actions", "操作");
    m.insert("help.view", "檢視");
    m.insert("help.files", "檔案");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "偏好設定");
    m.insert("menu.theme", "主題");
    m.insert("menu.toolLock", "工具鎖");
    m.insert("menu.snapToObjects", "貼齊至物件");
    m.insert("menu.zenMode", "禪模式");
    m.insert("menu.viewMode", "檢視模式");
    m.insert("menu.canvasStats", "畫布與形狀屬性");
    m.insert("menu.arrowBinding", "箭頭綁定");
    m.insert("menu.snapToMidpoints", "貼齊到中點");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "嵌入網站");
    m.insert("toolbar.drawToShape", "畫出形狀");
    m.insert("toolbar.bucketFill", "油漆桶填充工具");
    m.insert("toolbar.lasso", "繩索選取");
    m.insert("toolbar.textToDiagram", "文字轉圖表");
    m.insert("toolbar.mermaid", "Mermaid 至 Excalidraw");
    m.insert("toolbar.wireframeToCode", "線框稿轉為程式碼");
    m.insert("toolbar.generate", "Generate");
    m.insert("welcome.center1", "您的繪圖已保存在您的瀏覽器儲存空間中。");
    m.insert("welcome.center2", "瀏覽器儲存空間可能會被意外清除。");
    m.insert("welcome.center3", "定期將您的工作儲存到檔案，以避免遺失。");
    m.insert("stats.title", "屬性");
    m.insert("stats.shapes", "形狀");
    m.insert("stats.selectedType", "類型");
    m.insert("status.saved", "已儲存");
    m.insert(
        "status.unavailable",
        "需要 Excalidraw 後端服務，目前建置無法使用",
    );
    m.insert("labels.convertToCode", "轉換為程式碼");
    m.insert("labels.copySource", "複製來源至剪貼簿");
    m.insert("fileDrop.replace", "放開以取代內容");
    m.insert("fileDrop.add", "放開以加入畫布");
    m.insert("fileDrop.importLibrary", "放開以匯入圖庫");
    m.insert("fileDrop.replaceHint", "按住 {{Shift}} 以保留現有內容");
    m.insert("fileDrop.keepHint", "將保留你現有的內容");
    m.insert("fileDrop.libraryHint", "圖庫檔案將被附加");
    m.insert("fileDrop.replacedToast", "內容已取代。按 {{Ctrl+Z}} 復原");
    m.insert("colorPicker.colors", "顏色");
    m.insert("colorPicker.shades", "漸變色");
    m.insert("colorPicker.noShades", "沒有此顏色的漸變色");
    m.insert("colorPicker.mostUsedCustomColors", "最常使用的自訂顏色");
    m.insert("colorPicker.hexCode", "Hex 碼");
    m.insert("fontList.sceneFonts", "在此場景中");
    m.insert("fontList.availableFonts", "可用字型");
    m.insert("fontList.empty", "找不到字型");
    m.insert("quickSearch.placeholder", "快速搜尋");
    m.insert("elementLink.title", "連結至物件");
    m.insert("elementLink.desc", "點擊畫布上的形狀或貼上連結");
    m.insert("buttons.remove", "刪除");
    m.insert("buttons.cancel", "取消");
    m.insert("buttons.confirm", "確認");
    m
}

fn ja() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "選択");
    m.insert("toolbar.rectangle", "長方形");
    m.insert("toolbar.diamond", "ひし形");
    m.insert("toolbar.ellipse", "楕円");
    m.insert("toolbar.arrow", "矢印");
    m.insert("toolbar.line", "直線");
    m.insert("toolbar.freedraw", "描画");
    m.insert("toolbar.text", "テキスト");
    m.insert("toolbar.image", "画像を挿入");
    m.insert("action.undo", "元に戻す");
    m.insert("action.redo", "やり直す");
    m.insert("action.delete", "削除");
    m.insert("action.exportPng", "PNG としてエクスポート");
    m.insert("action.exportSvg", "SVG としてエクスポート");
    m.insert("action.zoomIn", "拡大");
    m.insert("action.zoomOut", "縮小");
    m.insert("action.toggleTheme", "テーマを切り替え");
    m.insert("menu.language", "言語");
    m.insert("theme.light", "ライト");
    m.insert("theme.dark", "ダーク");
    m.insert("action.group", "グループ化");
    m.insert("action.ungroup", "グループ解除");
    m.insert("ctx.bringFront", "最前面へ");
    m.insert("ctx.copy", "コピー");
    m.insert("ctx.delete", "削除");
    m.insert("ctx.duplicate", "複製");
    m.insert("ctx.paste", "貼り付け");
    m.insert("ctx.sendBack", "最背面へ");
    m.insert("hint.moveCanvas", "キャンバスを移動するには、ドラッグ中に{{スクロールホイール}}か{{スペース}}を押し続けるか、ハンドツールを使用します");
    m.insert("library.addSelected", "ライブラリに追加");
    m.insert("library.empty", "ライブラリは空です");
    m.insert("menu.exportImage", "画像をエクスポート");
    m.insert("menu.exportSvg", "SVG にエクスポート");
    m.insert("menu.grid", "グリッドを切り替え");
    m.insert("menu.help", "ヘルプ");
    m.insert(
        "menu.helpHint",
        "ドラッグで描画します。ツールバーからツールを選ぶか、キーを押してください。",
    );
    m.insert("menu.layers", "レイヤー");
    m.insert("menu.open", "開く");
    m.insert("menu.reset", "キャンバスをリセット");
    m.insert("menu.save", "保存先…");
    m.insert("menu.snap", "グリッドにスナップ");
    m.insert("panel.layers", "レイヤー");
    m.insert("panel.library", "ライブラリ");
    m.insert("menu.canvasBackground", "キャンバスの背景");
    m.insert("menu.followUs", "フォローする");
    m.insert("menu.discordGroup", "Discordグループ");
    m.insert("menu.save", "保存先...");
    m.insert("menu.exportImage", "画像をエクスポート...");
    m.insert("menu.collab", "ライブ共同編集...");
    m.insert("menu.commandPalette", "コマンドパレット");
    m.insert("menu.findOnCanvas", "キャンバス内を検索");
    m.insert("toolbar.hand", "ハンドツール");
    m.insert("toolbar.frame", "フレーム");
    m.insert("toolbar.eraser", "消しゴム");
    m.insert("toolbar.laser", "レーザーポインター");
    m.insert("action.duplicate", "複製");
    m.insert("ctx.selectAll", "すべて選択");
    m.insert("action.copyToClipboard", "クリップボードにコピー");
    m.insert("ctx.cut", "切り取り");
    m.insert("action.resetZoom", "ズームをリセット");
    m.insert("action.toggleGrid", "グリッドの表示切替");
    m.insert("action.zoomToFit", "全体を表示");
    m.insert("palette.placeholder", "コマンドを入力…");
    m.insert("palette.noResults", "結果が見つかりません");
    m.insert("find.placeholder", "キャンバス内のテキストを検索");
    m.insert("find.noResults", "結果なし");
    m.insert("help.tools", "ツール");
    m.insert("help.actions", "操作");
    m.insert("help.view", "表示");
    m.insert("help.files", "ファイル");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "環境設定");
    m.insert("menu.theme", "テーマ");
    m.insert("menu.toolLock", "ツールロック");
    m.insert("menu.snapToObjects", "オブジェクトにスナップ");
    m.insert("menu.zenMode", "禅モード");
    m.insert("menu.viewMode", "表示モード");
    m.insert("menu.canvasStats", "キャンバスと図形の属性");
    m.insert("menu.arrowBinding", "矢印のバインド");
    m.insert("menu.snapToMidpoints", "中点にスナップ");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web埋め込み");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "なげなわ選択");
    m.insert("toolbar.textToDiagram", "テキストからダイアグラムを生成");
    m.insert("toolbar.mermaid", "Mermaid を Excalidraw に変換");
    m.insert(
        "toolbar.wireframeToCode",
        "ワイヤーフレームからコードを生成",
    );
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "描画内容はブラウザのストレージに保存されます。",
    );
    m.insert(
        "welcome.center2",
        "ブラウザのストレージは誤って消去される可能性があります。",
    );
    m.insert(
        "welcome.center3",
        "作業内容をこまめにファイルに保存してください。",
    );
    m.insert("stats.title", "プロパティ");
    m.insert("stats.shapes", "図形");
    m.insert("stats.selectedType", "種類");
    m.insert("status.saved", "保存しました");
    m.insert(
        "status.unavailable",
        "Excalidraw バックエンドが必要です — このビルドでは利用できません",
    );
    m.insert("labels.convertToCode", "コードに変換");
    m.insert("labels.copySource", "ソースをクリップボードにコピー");
    m.insert("fileDrop.replace", "ドロップして内容を置き換え");
    m.insert("fileDrop.add", "ドロップしてキャンバスに追加");
    m.insert("fileDrop.importLibrary", "ドロップしてライブラリを読み込み");
    m.insert(
        "fileDrop.replaceHint",
        "{{Shift}} を押したままにすると既存の内容を保持します",
    );
    m.insert("fileDrop.keepHint", "既存の内容は保持されます");
    m.insert("fileDrop.libraryHint", "ライブラリファイルは追加されます");
    m.insert(
        "fileDrop.replacedToast",
        "内容を置き換えました。{{Ctrl+Z}} で元に戻せます",
    );
    m.insert("colorPicker.colors", "色");
    m.insert("colorPicker.shades", "影");
    m.insert("colorPicker.noShades", "この色で利用できる影はありません");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "最も使用されているカスタム色",
    );
    m.insert("colorPicker.hexCode", "16進コード");
    m.insert("fontList.sceneFonts", "このシーン内");
    m.insert("fontList.availableFonts", "利用可能フォント");
    m.insert("fontList.empty", "フォントなし");
    m.insert("quickSearch.placeholder", "クイック検索");
    m.insert("elementLink.title", "オブジェクトにリンク");
    m.insert(
        "elementLink.desc",
        "キャンバス上の図形をクリックするか、リンクを貼り付けます。",
    );
    m.insert("buttons.remove", "削除");
    m.insert("buttons.cancel", "キャンセル");
    m.insert("buttons.confirm", "確認");
    m
}

fn ko() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "선택");
    m.insert("toolbar.rectangle", "사각형");
    m.insert("toolbar.diamond", "마름모");
    m.insert("toolbar.ellipse", "타원");
    m.insert("toolbar.arrow", "화살표");
    m.insert("toolbar.line", "선");
    m.insert("toolbar.freedraw", "그리기");
    m.insert("toolbar.text", "텍스트");
    m.insert("toolbar.image", "이미지 삽입");
    m.insert("action.undo", "실행 취소");
    m.insert("action.redo", "다시 실행");
    m.insert("action.delete", "삭제");
    m.insert("action.exportPng", "PNG로 내보내기");
    m.insert("action.exportSvg", "SVG로 내보내기");
    m.insert("action.zoomIn", "확대");
    m.insert("action.zoomOut", "축소");
    m.insert("action.toggleTheme", "테마 전환");
    m.insert("menu.language", "언어");
    m.insert("theme.light", "라이트");
    m.insert("theme.dark", "다크");
    m.insert("action.group", "그룹화");
    m.insert("action.ungroup", "그룹 해제");
    m.insert("ctx.bringFront", "맨 앞으로");
    m.insert("ctx.copy", "복사");
    m.insert("ctx.delete", "삭제");
    m.insert("ctx.duplicate", "복제");
    m.insert("ctx.paste", "붙여넣기");
    m.insert("ctx.sendBack", "맨 뒤로");
    m.insert("hint.moveCanvas", "캔버스를 이동하려면 드래그하는 동안 {{스크롤 휠}} 또는 {{스페이스}}를 누르고 있거나 손 도구를 사용하세요");
    m.insert("library.addSelected", "라이브러리에 추가");
    m.insert("library.empty", "라이브러리가 비어 있습니다");
    m.insert("menu.exportImage", "이미지 내보내기");
    m.insert("menu.exportSvg", "SVG로 내보내기");
    m.insert("menu.grid", "격자 전환");
    m.insert("menu.help", "도움말");
    m.insert(
        "menu.helpHint",
        "드래그하여 그립니다. 도구 모음에서 도구를 고르거나 해당 키를 누르세요.",
    );
    m.insert("menu.layers", "레이어");
    m.insert("menu.open", "열기");
    m.insert("menu.reset", "캔버스 초기화");
    m.insert("menu.save", "다른 이름으로 저장…");
    m.insert("menu.snap", "격자에 맞추기");
    m.insert("panel.layers", "레이어");
    m.insert("panel.library", "라이브러리");
    m.insert("menu.canvasBackground", "캔버스 배경");
    m.insert("menu.followUs", "팔로우");
    m.insert("menu.discordGroup", "Discord 그룹");
    m.insert("menu.save", "다른 이름으로 저장...");
    m.insert("menu.exportImage", "이미지 내보내기...");
    m.insert("menu.collab", "실시간 협업...");
    m.insert("menu.commandPalette", "명령 팔레트");
    m.insert("menu.findOnCanvas", "캔버스에서 찾기");
    m.insert("toolbar.hand", "손 도구");
    m.insert("toolbar.frame", "프레임 도구");
    m.insert("toolbar.eraser", "지우개");
    m.insert("toolbar.laser", "레이저 포인터");
    m.insert("action.duplicate", "복제");
    m.insert("ctx.selectAll", "모두 선택");
    m.insert("action.copyToClipboard", "클립보드에 복사");
    m.insert("ctx.cut", "잘라내기");
    m.insert("action.resetZoom", "확대/축소 초기화");
    m.insert("action.toggleGrid", "격자 전환");
    m.insert("action.zoomToFit", "화면에 맞추기");
    m.insert("palette.placeholder", "명령을 입력하세요…");
    m.insert("palette.noResults", "결과를 찾을 수 없습니다");
    m.insert("find.placeholder", "캔버스에서 텍스트 찾기");
    m.insert("find.noResults", "결과 없음");
    m.insert("help.tools", "도구");
    m.insert("help.actions", "작업");
    m.insert("help.view", "보기");
    m.insert("help.files", "파일");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "환경설정");
    m.insert("menu.theme", "테마");
    m.insert("menu.toolLock", "도구 잠금");
    m.insert("menu.snapToObjects", "개체에 스냅");
    m.insert("menu.zenMode", "젠 모드");
    m.insert("menu.viewMode", "보기 모드");
    m.insert("menu.canvasStats", "캔버스 및 도형 속성");
    m.insert("menu.arrowBinding", "화살표 바인딩");
    m.insert("menu.snapToMidpoints", "중심점에 스냅");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "웹 임베드");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "맞춤형 부분 선택");
    m.insert("toolbar.textToDiagram", "텍스트를 표로 만들기");
    m.insert("toolbar.mermaid", "Mermaid를 Excalidraw로");
    m.insert("toolbar.wireframeToCode", "와이어프레임을 코드로");
    m.insert("toolbar.generate", "Generate");
    m.insert("welcome.center1", "그림은 브라우저 저장소에 저장됩니다.");
    m.insert(
        "welcome.center2",
        "브라우저 저장소는 실수로 지워질 수 있습니다.",
    );
    m.insert(
        "welcome.center3",
        "작업 손실을 피하려면 주기적으로 파일로 저장하세요.",
    );
    m.insert("stats.title", "속성");
    m.insert("stats.shapes", "도형");
    m.insert("stats.selectedType", "유형");
    m.insert("status.saved", "저장됨");
    m.insert(
        "status.unavailable",
        "Excalidraw 백엔드가 필요합니다 — 이 빌드에서는 사용할 수 없습니다",
    );
    m.insert("labels.convertToCode", "코드로 변환하기");
    m.insert("labels.copySource", "클립보드에 복사하기");
    m.insert("fileDrop.replace", "놓아서 내용 바꾸기");
    m.insert("fileDrop.add", "놓아서 캔버스에 추가");
    m.insert("fileDrop.importLibrary", "놓아서 라이브러리 가져오기");
    m.insert(
        "fileDrop.replaceHint",
        "{{Shift}}를 누르고 있으면 기존 내용이 유지됩니다",
    );
    m.insert("fileDrop.keepHint", "기존 내용이 유지됩니다");
    m.insert("fileDrop.libraryHint", "라이브러리 파일이 추가됩니다");
    m.insert(
        "fileDrop.replacedToast",
        "내용을 바꿨습니다. {{Ctrl+Z}}로 실행 취소",
    );
    m.insert("colorPicker.colors", "색상");
    m.insert("colorPicker.shades", "색조");
    m.insert("colorPicker.noShades", "사용할 수 있는 색조가 없음");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "가장 많이 사용된 색상들",
    );
    m.insert("colorPicker.hexCode", "Hex 코드");

    m.insert("fontList.sceneFonts", "In this scene");
    m.insert("fontList.availableFonts", "Available fonts");
    m.insert("fontList.empty", "No fonts found");
    m.insert("quickSearch.placeholder", "Quick search");
    m.insert("elementLink.title", "객체에 연결");
    m.insert(
        "elementLink.desc",
        "캔버스에서 도형을 클릭하거나 링크를 붙여 넣으세요.",
    );
    m.insert("buttons.remove", "삭제");
    m.insert("buttons.cancel", "취소");
    m.insert("buttons.confirm", "확인");
    m
}

fn de() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Auswählen");
    m.insert("toolbar.rectangle", "Rechteck");
    m.insert("toolbar.diamond", "Raute");
    m.insert("toolbar.ellipse", "Ellipse");
    m.insert("toolbar.arrow", "Pfeil");
    m.insert("toolbar.line", "Linie");
    m.insert("toolbar.freedraw", "Zeichnen");
    m.insert("toolbar.text", "Text");
    m.insert("action.undo", "Rückgängig");
    m.insert("action.redo", "Wiederholen");
    m.insert("action.delete", "Löschen");
    m.insert("action.exportPng", "Als PNG exportieren");
    m.insert("action.exportSvg", "Als SVG exportieren");
    m.insert("action.zoomIn", "Vergrößern");
    m.insert("action.zoomOut", "Verkleinern");
    m.insert("action.toggleTheme", "Design umschalten");
    m.insert("menu.language", "Sprache");
    m.insert("theme.light", "Hell");
    m.insert("theme.dark", "Dunkel");
    m.insert("action.group", "Gruppieren");
    m.insert("action.ungroup", "Gruppierung aufheben");
    m.insert("ctx.bringFront", "Nach vorn");
    m.insert("ctx.copy", "Kopieren");
    m.insert("ctx.delete", "Löschen");
    m.insert("ctx.duplicate", "Duplizieren");
    m.insert("ctx.paste", "Einfügen");
    m.insert("ctx.sendBack", "Nach hinten");
    m.insert("hint.moveCanvas", "Leinwand verschieben: Beim Ziehen {{Mausrad}} oder {{Leertaste}} gedrückt halten oder das Hand-Werkzeug verwenden");
    m.insert("library.addSelected", "Zur Bibliothek hinzufügen");
    m.insert("library.empty", "Deine Bibliothek ist leer");
    m.insert("menu.exportImage", "Bild exportieren");
    m.insert("menu.exportSvg", "Als SVG exportieren");
    m.insert("menu.grid", "Raster umschalten");
    m.insert("menu.help", "Hilfe");
    m.insert(
        "menu.helpHint",
        "Ziehen zum Zeichnen. Wähle ein Werkzeug aus der Werkzeugleiste oder drücke seine Taste.",
    );
    m.insert("menu.layers", "Ebenen");
    m.insert("menu.open", "Öffnen");
    m.insert("menu.reset", "Leinwand zurücksetzen");
    m.insert("menu.save", "Speichern unter…");
    m.insert("menu.snap", "Am Raster ausrichten");
    m.insert("panel.layers", "Ebenen");
    m.insert("panel.library", "Bibliothek");
    m.insert("menu.canvasBackground", "Leinwandhintergrund");
    m.insert("menu.followUs", "Folge uns");
    m.insert("menu.discordGroup", "Discord-Chat");
    m.insert("menu.save", "Speichern unter...");
    m.insert("menu.exportImage", "Bild exportieren...");
    m.insert("menu.collab", "Live-Zusammenarbeit...");
    m.insert("menu.commandPalette", "Befehlspalette");
    m.insert("menu.findOnCanvas", "Auf der Leinwand suchen");
    m.insert("toolbar.hand", "Handwerkzeug");
    m.insert("toolbar.image", "Bild einfügen");
    m.insert("toolbar.frame", "Rahmenwerkzeug");
    m.insert("toolbar.eraser", "Radierer");
    m.insert("toolbar.laser", "Laserpointer");
    m.insert("action.duplicate", "Duplizieren");
    m.insert("ctx.selectAll", "Alles auswählen");
    m.insert("action.copyToClipboard", "In die Zwischenablage kopieren");
    m.insert("ctx.cut", "Ausschneiden");
    m.insert("action.resetZoom", "Zoom zurücksetzen");
    m.insert("action.toggleGrid", "Raster umschalten");
    m.insert("action.zoomToFit", "An Größe anpassen");
    m.insert("palette.placeholder", "Befehl eingeben…");
    m.insert("palette.noResults", "Keine Ergebnisse gefunden");
    m.insert("find.placeholder", "Text auf der Leinwand suchen");
    m.insert("find.noResults", "Keine Ergebnisse");
    m.insert("help.tools", "Werkzeuge");
    m.insert("help.actions", "Aktionen");
    m.insert("help.view", "Ansicht");
    m.insert("help.files", "Dateien");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Einstellungen");
    m.insert("menu.theme", "Design");
    m.insert("menu.toolLock", "Werkzeugsperre");
    m.insert("menu.snapToObjects", "An Objekten ausrichten");
    m.insert("menu.zenMode", "Zen-Modus");
    m.insert("menu.viewMode", "Ansichtsmodus");
    m.insert("menu.canvasStats", "Leinwand- & Formeigenschaften");
    m.insert("menu.arrowBinding", "Pfeilbindung");
    m.insert("menu.snapToMidpoints", "An Mittelpunkten ausrichten");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web-Einbettung");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Lassoauswahl");
    m.insert("toolbar.textToDiagram", "Text zu Diagramm");
    m.insert("toolbar.mermaid", "Mermaid zu Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe zu Code");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Deine Zeichnungen werden im Browser-Speicher gesichert.",
    );
    m.insert(
        "welcome.center2",
        "Der Browser-Speicher kann versehentlich gelöscht werden.",
    );
    m.insert(
        "welcome.center3",
        "Sichere deine Arbeit regelmäßig in einer Datei, um nichts zu verlieren.",
    );
    m.insert("stats.title", "Eigenschaften");
    m.insert("stats.shapes", "Formen");
    m.insert("stats.selectedType", "Typ");
    m.insert("status.saved", "Gespeichert");
    m.insert(
        "status.unavailable",
        "Benötigt das Excalidraw-Backend — in diesem Build nicht verfügbar",
    );
    m.insert("labels.convertToCode", "In Code konvertieren");
    m.insert("labels.copySource", "Quelle in Zwischenablage kopieren");
    m.insert("fileDrop.replace", "Zum Ersetzen des Inhalts ablegen");
    m.insert("fileDrop.add", "Zum Hinzufügen zur Zeichenfläche ablegen");
    m.insert(
        "fileDrop.importLibrary",
        "Zum Importieren der Bibliothek ablegen",
    );
    m.insert(
        "fileDrop.replaceHint",
        "{{Umschalt}} halten, um vorhandenen Inhalt zu behalten",
    );
    m.insert(
        "fileDrop.keepHint",
        "Dein vorhandener Inhalt bleibt erhalten",
    );
    m.insert(
        "fileDrop.libraryHint",
        "Bibliotheksdateien werden angehängt",
    );
    m.insert(
        "fileDrop.replacedToast",
        "Inhalt ersetzt. {{Strg+Z}} zum Rückgängigmachen",
    );
    m.insert("colorPicker.colors", "Farben");
    m.insert("colorPicker.shades", "Schattierungen");
    m.insert(
        "colorPicker.noShades",
        "Keine Schattierungen für diese Farbe verfügbar",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Beliebteste benutzerdefinierte Farben",
    );
    m.insert("colorPicker.hexCode", "Hex-Code");
    m.insert("fontList.sceneFonts", "In dieser Szene");
    m.insert("fontList.availableFonts", "Verfügbare Schriftarten");
    m.insert("fontList.empty", "Keine Schriftarten gefunden");
    m.insert("quickSearch.placeholder", "Schnellsuche");
    m.insert("elementLink.title", "Link zum Objekt");
    m.insert(
        "elementLink.desc",
        "Klicke auf eine Form auf der Zeichenfläche oder füge einen Link ein.",
    );
    m.insert("buttons.remove", "Entfernen");
    m.insert("buttons.cancel", "Abbrechen");
    m.insert("buttons.confirm", "Bestätigen");
    m
}

fn fr() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Sélectionner");
    m.insert("toolbar.rectangle", "Rectangle");
    m.insert("toolbar.diamond", "Losange");
    m.insert("toolbar.ellipse", "Ellipse");
    m.insert("toolbar.arrow", "Flèche");
    m.insert("toolbar.line", "Ligne");
    m.insert("toolbar.freedraw", "Dessiner");
    m.insert("toolbar.text", "Texte");
    m.insert("action.undo", "Annuler");
    m.insert("action.redo", "Rétablir");
    m.insert("action.delete", "Supprimer");
    m.insert("action.exportPng", "Exporter en PNG");
    m.insert("action.exportSvg", "Exporter en SVG");
    m.insert("action.zoomIn", "Zoom avant");
    m.insert("action.zoomOut", "Zoom arrière");
    m.insert("action.toggleTheme", "Basculer le thème");
    m.insert("menu.language", "Langue");
    m.insert("theme.light", "Clair");
    m.insert("theme.dark", "Sombre");
    m.insert("action.group", "Grouper");
    m.insert("action.ungroup", "Dissocier");
    m.insert("ctx.bringFront", "Premier plan");
    m.insert("ctx.copy", "Copier");
    m.insert("ctx.delete", "Supprimer");
    m.insert("ctx.duplicate", "Dupliquer");
    m.insert("ctx.paste", "Coller");
    m.insert("ctx.sendBack", "Arrière-plan");
    m.insert("hint.moveCanvas", "Pour déplacer la toile, maintenez la {{molette}} ou la {{barre d'espace}} tout en faisant glisser, ou utilisez l'outil main");
    m.insert("library.addSelected", "Ajouter à la bibliothèque");
    m.insert("library.empty", "Votre bibliothèque est vide");
    m.insert("menu.exportImage", "Exporter l'image");
    m.insert("menu.exportSvg", "Exporter en SVG");
    m.insert("menu.grid", "Afficher la grille");
    m.insert("menu.help", "Aide");
    m.insert("menu.helpHint", "Tracez en faisant glisser. Choisissez un outil dans la barre d'outils ou appuyez sur sa touche.");
    m.insert("menu.layers", "Calques");
    m.insert("menu.open", "Ouvrir");
    m.insert("menu.reset", "Réinitialiser la toile");
    m.insert("menu.save", "Enregistrer sous…");
    m.insert("menu.snap", "Aligner sur la grille");
    m.insert("panel.layers", "Calques");
    m.insert("panel.library", "Bibliothèque");
    m.insert("menu.canvasBackground", "Arrière-plan du canevas");
    m.insert("menu.followUs", "Suivez-nous");
    m.insert("menu.discordGroup", "Salon Discord");
    m.insert("menu.save", "Enregistrer sous...");
    m.insert("menu.exportImage", "Exporter l'image...");
    m.insert("menu.collab", "Collaboration en direct...");
    m.insert("menu.commandPalette", "Palette de commandes");
    m.insert("menu.findOnCanvas", "Trouver sur le canevas");
    m.insert("toolbar.hand", "Outil main");
    m.insert("toolbar.image", "Insérer une image");
    m.insert("toolbar.frame", "Outil de cadre");
    m.insert("toolbar.eraser", "Gomme");
    m.insert("toolbar.laser", "Pointeur laser");
    m.insert("action.duplicate", "Dupliquer");
    m.insert("ctx.selectAll", "Tout sélectionner");
    m.insert("action.copyToClipboard", "Copier dans le presse-papiers");
    m.insert("ctx.cut", "Couper");
    m.insert("action.resetZoom", "Réinitialiser le zoom");
    m.insert("action.toggleGrid", "Afficher/masquer la grille");
    m.insert("action.zoomToFit", "Ajuster à l'écran");
    m.insert("palette.placeholder", "Saisir une commande…");
    m.insert("palette.noResults", "Aucun résultat");
    m.insert("find.placeholder", "Rechercher du texte sur le canevas");
    m.insert("find.noResults", "Aucun résultat");
    m.insert("help.tools", "Outils");
    m.insert("help.actions", "Actions");
    m.insert("help.view", "Affichage");
    m.insert("help.files", "Fichiers");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Préférences");
    m.insert("menu.theme", "Thème");
    m.insert("menu.toolLock", "Verrou d'outil");
    m.insert("menu.snapToObjects", "Aimanter aux objets");
    m.insert("menu.zenMode", "Mode Zen");
    m.insert("menu.viewMode", "Mode visualisation");
    m.insert("menu.canvasStats", "Propriétés du canevas et des formes");
    m.insert("menu.arrowBinding", "Liaison des flèches");
    m.insert("menu.snapToMidpoints", "Aimanter aux milieux");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Intégration Web");
    m.insert("toolbar.drawToShape", "Dessiner pour façonner");
    m.insert("toolbar.bucketFill", "Remplissage du seau");
    m.insert("toolbar.lasso", "Sélection lasso");
    m.insert("toolbar.textToDiagram", "Texte vers diagramme");
    m.insert("toolbar.mermaid", "De Mermaid à Excalidraw");
    m.insert("toolbar.wireframeToCode", "Modèle en fil de fer vers code");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Vos dessins sont enregistrés dans le stockage de votre navigateur.",
    );
    m.insert(
        "welcome.center2",
        "Ce stockage peut être effacé par accident.",
    );
    m.insert(
        "welcome.center3",
        "Enregistrez régulièrement votre travail dans un fichier pour éviter toute perte.",
    );
    m.insert("stats.title", "Propriétés");
    m.insert("stats.shapes", "Formes");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Enregistré");
    m.insert(
        "status.unavailable",
        "Nécessite le backend Excalidraw — indisponible dans cette version",
    );
    m.insert("labels.convertToCode", "Convertir en code");
    m.insert(
        "labels.copySource",
        "Copier la source dans le presse-papiers",
    );
    m.insert("fileDrop.replace", "Déposer pour remplacer le contenu");
    m.insert("fileDrop.add", "Déposer pour ajouter à la toile");
    m.insert(
        "fileDrop.importLibrary",
        "Déposer pour importer la bibliothèque",
    );
    m.insert(
        "fileDrop.replaceHint",
        "Maintenir {{Maj}} pour conserver le contenu existant",
    );
    m.insert("fileDrop.keepHint", "Votre contenu existant sera conservé");
    m.insert(
        "fileDrop.libraryHint",
        "Les fichiers de bibliothèque seront ajoutés",
    );
    m.insert(
        "fileDrop.replacedToast",
        "Contenu remplacé. {{Ctrl+Z}} pour annuler",
    );
    m.insert("colorPicker.colors", "Couleurs");
    m.insert("colorPicker.shades", "Nuances");
    m.insert(
        "colorPicker.noShades",
        "Aucune nuance disponible pour cette couleur",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Couleurs personnalisées les plus fréquemment utilisées",
    );
    m.insert("colorPicker.hexCode", "Code hex");
    m.insert("fontList.sceneFonts", "Dans cette scène");
    m.insert("fontList.availableFonts", "Polices disponibles");
    m.insert("fontList.empty", "Aucune police trouvée");
    m.insert("quickSearch.placeholder", "Recherche rapide");
    m.insert("elementLink.title", "Lien vers un objet");
    m.insert(
        "elementLink.desc",
        "Cliquez sur une forme sur le canvas ou collez un lien.",
    );
    m.insert("buttons.remove", "Supprimer");
    m.insert("buttons.cancel", "Annuler");
    m.insert("buttons.confirm", "Confirmer");
    m
}

fn es() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Seleccionar");
    m.insert("toolbar.rectangle", "Rectángulo");
    m.insert("toolbar.diamond", "Rombo");
    m.insert("toolbar.ellipse", "Elipse");
    m.insert("toolbar.arrow", "Flecha");
    m.insert("toolbar.line", "Línea");
    m.insert("toolbar.freedraw", "Dibujar");
    m.insert("toolbar.text", "Texto");
    m.insert("action.undo", "Deshacer");
    m.insert("action.redo", "Rehacer");
    m.insert("action.delete", "Eliminar");
    m.insert("action.exportPng", "Exportar como PNG");
    m.insert("action.exportSvg", "Exportar como SVG");
    m.insert("action.zoomIn", "Acercar");
    m.insert("action.zoomOut", "Alejar");
    m.insert("action.toggleTheme", "Cambiar tema");
    m.insert("menu.language", "Idioma");
    m.insert("theme.light", "Claro");
    m.insert("theme.dark", "Oscuro");
    m.insert("action.group", "Agrupar");
    m.insert("action.ungroup", "Desagrupar");
    m.insert("ctx.bringFront", "Traer al frente");
    m.insert("ctx.copy", "Copiar");
    m.insert("ctx.delete", "Eliminar");
    m.insert("ctx.duplicate", "Duplicar");
    m.insert("ctx.paste", "Pegar");
    m.insert("ctx.sendBack", "Enviar al fondo");
    m.insert("hint.moveCanvas", "Para mover el lienzo, mantén pulsada la {{rueda del ratón}} o {{espacio}} mientras arrastras, o usa la herramienta mano");
    m.insert("library.addSelected", "Añadir a la biblioteca");
    m.insert("library.empty", "Tu biblioteca está vacía");
    m.insert("menu.exportImage", "Exportar imagen");
    m.insert("menu.exportSvg", "Exportar a SVG");
    m.insert("menu.grid", "Mostrar cuadrícula");
    m.insert("menu.help", "Ayuda");
    m.insert(
        "menu.helpHint",
        "Arrastra para dibujar. Elige una herramienta de la barra o pulsa su tecla.",
    );
    m.insert("menu.layers", "Capas");
    m.insert("menu.open", "Abrir");
    m.insert("menu.reset", "Restablecer el lienzo");
    m.insert("menu.save", "Guardar en…");
    m.insert("menu.snap", "Ajustar a la cuadrícula");
    m.insert("panel.layers", "Capas");
    m.insert("panel.library", "Biblioteca");
    m.insert("menu.canvasBackground", "Fondo del lienzo");
    m.insert("menu.followUs", "Síguenos");
    m.insert("menu.discordGroup", "Chat de Discord");
    m.insert("menu.save", "Guardar en...");
    m.insert("menu.exportImage", "Exportar imagen...");
    m.insert("menu.collab", "Colaboración en vivo...");
    m.insert("menu.commandPalette", "Paleta de comandos");
    m.insert("menu.findOnCanvas", "Buscar en el lienzo");
    m.insert("toolbar.hand", "Herramienta de mano");
    m.insert("toolbar.image", "Insertar imagen");
    m.insert("toolbar.frame", "Herramienta Estructura");
    m.insert("toolbar.eraser", "Goma");
    m.insert("toolbar.laser", "Puntero láser");
    m.insert("action.duplicate", "Duplicar");
    m.insert("ctx.selectAll", "Seleccionar todo");
    m.insert("action.copyToClipboard", "Copiar al portapapeles");
    m.insert("ctx.cut", "Cortar");
    m.insert("action.resetZoom", "Restablecer zoom");
    m.insert("action.toggleGrid", "Alternar cuadrícula");
    m.insert("action.zoomToFit", "Ajustar a la pantalla");
    m.insert("palette.placeholder", "Escribe un comando…");
    m.insert("palette.noResults", "No se encontraron resultados");
    m.insert("find.placeholder", "Buscar texto en el lienzo");
    m.insert("find.noResults", "Sin resultados");
    m.insert("help.tools", "Herramientas");
    m.insert("help.actions", "Acciones");
    m.insert("help.view", "Vista");
    m.insert("help.files", "Archivos");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferencias");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Bloqueo de herramienta");
    m.insert("menu.snapToObjects", "Ajustar a objetos");
    m.insert("menu.zenMode", "Modo Zen");
    m.insert("menu.viewMode", "Modo de visualización");
    m.insert("menu.canvasStats", "Propiedades del lienzo y las formas");
    m.insert("menu.arrowBinding", "Vinculación de flechas");
    m.insert("menu.snapToMidpoints", "Ajustar a puntos medios");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Incrustar Web");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Selección de lazo");
    m.insert("toolbar.textToDiagram", "Texto a diagrama");
    m.insert("toolbar.mermaid", "Mermaid a Excalidraw");
    m.insert("toolbar.wireframeToCode", "Esquema a código");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Tus dibujos se guardan en el almacenamiento de tu navegador.",
    );
    m.insert(
        "welcome.center2",
        "Ese almacenamiento puede borrarse sin querer.",
    );
    m.insert(
        "welcome.center3",
        "Guarda tu trabajo en un archivo de vez en cuando para no perderlo.",
    );
    m.insert("stats.title", "Propiedades");
    m.insert("stats.shapes", "Formas");
    m.insert("stats.selectedType", "Tipo");
    m.insert("status.saved", "Guardado");
    m.insert(
        "status.unavailable",
        "Requiere el backend de Excalidraw — no disponible en esta compilación",
    );
    m.insert("labels.convertToCode", "Convertir a código");
    m.insert("labels.copySource", "Copiar fuente al portapapeles");
    m.insert("fileDrop.replace", "Suelta para reemplazar el contenido");
    m.insert("fileDrop.add", "Suelta para añadir al lienzo");
    m.insert(
        "fileDrop.importLibrary",
        "Suelta para importar la biblioteca",
    );
    m.insert(
        "fileDrop.replaceHint",
        "Mantén {{Mayús}} para conservar el contenido existente",
    );
    m.insert("fileDrop.keepHint", "Tu contenido existente se conservará");
    m.insert(
        "fileDrop.libraryHint",
        "Los archivos de biblioteca se añadirán",
    );
    m.insert(
        "fileDrop.replacedToast",
        "Contenido reemplazado. {{Ctrl+Z}} para deshacer",
    );
    m.insert("colorPicker.colors", "Colores");
    m.insert("colorPicker.shades", "Sombras");
    m.insert(
        "colorPicker.noShades",
        "No hay sombras disponibles para este color",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Colores personalizados más utilizados",
    );
    m.insert("colorPicker.hexCode", "Código Hexadecimal");
    m.insert("fontList.sceneFonts", "En esta escena");
    m.insert("fontList.availableFonts", "Fuentes disponibles");
    m.insert("fontList.empty", "No se han encontrado fuentes");
    m.insert("quickSearch.placeholder", "Búsqueda rápida");
    m.insert("elementLink.title", "Enlace al objeto");
    m.insert(
        "elementLink.desc",
        "Haga clic en una forma en lienzo o pegue un enlace.",
    );
    m.insert("buttons.remove", "Eliminar");
    m.insert("buttons.cancel", "Cancelar");
    m.insert("buttons.confirm", "Confirmar");
    m
}

fn it() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Seleziona");
    m.insert("toolbar.rectangle", "Rettangolo");
    m.insert("toolbar.diamond", "Rombo");
    m.insert("toolbar.ellipse", "Ellisse");
    m.insert("toolbar.arrow", "Freccia");
    m.insert("toolbar.line", "Linea");
    m.insert("toolbar.freedraw", "Disegna");
    m.insert("toolbar.text", "Testo");
    m.insert("action.undo", "Annulla");
    m.insert("action.redo", "Ripeti");
    m.insert("action.delete", "Elimina");
    m.insert("action.exportPng", "Esporta come PNG");
    m.insert("action.exportSvg", "Esporta come SVG");
    m.insert("action.zoomIn", "Ingrandisci");
    m.insert("action.zoomOut", "Riduci");
    m.insert("action.toggleTheme", "Cambia tema");
    m.insert("menu.language", "Lingua");
    m.insert("theme.light", "Chiaro");
    m.insert("theme.dark", "Scuro");
    m.insert("action.group", "Raggruppa");
    m.insert("action.ungroup", "Separa");
    m.insert("ctx.bringFront", "Porta in primo piano");
    m.insert("ctx.copy", "Copia");
    m.insert("ctx.delete", "Elimina");
    m.insert("ctx.duplicate", "Duplica");
    m.insert("ctx.paste", "Incolla");
    m.insert("ctx.sendBack", "Porta in secondo piano");
    m.insert("hint.moveCanvas", "Per spostare la tela, tieni premuta la {{rotella}} o la {{barra spaziatrice}} mentre trascini, oppure usa lo strumento mano");
    m.insert("library.addSelected", "Aggiungi alla libreria");
    m.insert("library.empty", "La tua libreria è vuota");
    m.insert("menu.exportImage", "Esporta immagine");
    m.insert("menu.exportSvg", "Esporta in SVG");
    m.insert("menu.grid", "Mostra griglia");
    m.insert("menu.help", "Guida");
    m.insert(
        "menu.helpHint",
        "Trascina per disegnare. Scegli uno strumento dalla barra o premi il suo tasto.",
    );
    m.insert("menu.layers", "Livelli");
    m.insert("menu.open", "Apri");
    m.insert("menu.reset", "Reimposta la tela");
    m.insert("menu.save", "Salva in…");
    m.insert("menu.snap", "Aggancia alla griglia");
    m.insert("panel.layers", "Livelli");
    m.insert("panel.library", "Libreria");
    m.insert("menu.canvasBackground", "Sfondo della tela");
    m.insert("menu.followUs", "Seguici");
    m.insert("menu.discordGroup", "Chat Discord");
    m.insert("menu.save", "Salva su...");
    m.insert("menu.exportImage", "Esporta immagine...");
    m.insert("menu.collab", "Collaborazione in tempo reale...");
    m.insert("menu.commandPalette", "Palette dei comandi");
    m.insert("menu.findOnCanvas", "Trova sulla tela");
    m.insert("toolbar.hand", "Strumento mano");
    m.insert("toolbar.image", "Inserisci immagine");
    m.insert("toolbar.frame", "Strumento riquadro");
    m.insert("toolbar.eraser", "Gomma");
    m.insert("toolbar.laser", "Puntatore laser");
    m.insert("action.duplicate", "Duplica");
    m.insert("ctx.selectAll", "Seleziona tutto");
    m.insert("action.copyToClipboard", "Copia negli appunti");
    m.insert("ctx.cut", "Taglia");
    m.insert("action.resetZoom", "Reimposta zoom");
    m.insert("action.toggleGrid", "Attiva/disattiva griglia");
    m.insert("action.zoomToFit", "Adatta allo schermo");
    m.insert("palette.placeholder", "Digita un comando…");
    m.insert("palette.noResults", "Nessun risultato trovato");
    m.insert("find.placeholder", "Cerca testo sulla tela");
    m.insert("find.noResults", "Nessun risultato");
    m.insert("help.tools", "Strumenti");
    m.insert("help.actions", "Azioni");
    m.insert("help.view", "Visualizzazione");
    m.insert("help.files", "File");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferenze");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Blocco strumento");
    m.insert("menu.snapToObjects", "Aggancia agli oggetti");
    m.insert("menu.zenMode", "Modalità Zen");
    m.insert("menu.viewMode", "Modalità visualizzazione");
    m.insert("menu.canvasStats", "Proprietà di tela e forme");
    m.insert("menu.arrowBinding", "Vincolo freccia");
    m.insert("menu.snapToMidpoints", "Aggancia ai punti medi");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Incorporamento Web");
    m.insert("toolbar.drawToShape", "Disegna per la forma");
    m.insert("toolbar.bucketFill", "Secchiello riempimento");
    m.insert("toolbar.lasso", "Selezione a lazo");
    m.insert("toolbar.textToDiagram", "Testo a diagramma");
    m.insert("toolbar.mermaid", "Da Mermaid a Excalidraw");
    m.insert("toolbar.wireframeToCode", "Dal wireframe al codice");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "I tuoi disegni sono salvati nella memoria del browser.",
    );
    m.insert(
        "welcome.center2",
        "La memoria del browser può essere cancellata per errore.",
    );
    m.insert(
        "welcome.center3",
        "Salva di tanto in tanto il tuo lavoro su un file per non perderlo.",
    );
    m.insert("stats.title", "Proprietà");
    m.insert("stats.shapes", "Forme");
    m.insert("stats.selectedType", "Tipo");
    m.insert("status.saved", "Salvato");
    m.insert(
        "status.unavailable",
        "Richiede il backend Excalidraw — non disponibile in questa build",
    );
    m.insert("labels.convertToCode", "Converti in codice");
    m.insert("labels.copySource", "Copia sorgente negli appunti");
    m.insert("fileDrop.replace", "Rilascia per sostituire il contenuto");
    m.insert("fileDrop.add", "Rilascia per aggiungere alla tela");
    m.insert(
        "fileDrop.importLibrary",
        "Rilascia per importare la libreria",
    );
    m.insert(
        "fileDrop.replaceHint",
        "Tieni premuto {{Maiusc}} per mantenere il contenuto esistente",
    );
    m.insert(
        "fileDrop.keepHint",
        "Il contenuto esistente verrà mantenuto",
    );
    m.insert(
        "fileDrop.libraryHint",
        "I file della libreria verranno aggiunti",
    );
    m.insert(
        "fileDrop.replacedToast",
        "Contenuto sostituito. {{Ctrl+Z}} per annullare",
    );
    m.insert("colorPicker.colors", "Colori");
    m.insert("colorPicker.shades", "Sfumature");
    m.insert(
        "colorPicker.noShades",
        "Nessuna sfumatura disponibile per questo colore",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Colori personalizzati più utilizzati",
    );
    m.insert("colorPicker.hexCode", "Codice esadecimale");
    m.insert("fontList.sceneFonts", "In questa scena");
    m.insert("fontList.availableFonts", "Font disponibili");
    m.insert("fontList.empty", "Nessun font trovato");
    m.insert("quickSearch.placeholder", "Ricerca rapida");
    m.insert("elementLink.title", "Link all'oggetto");
    m.insert(
        "elementLink.desc",
        "Fare clic su una forma su tela o incollare un link.",
    );
    m.insert("buttons.remove", "Rimuovi");
    m.insert("buttons.cancel", "Annulla");
    m.insert("buttons.confirm", "Conferma");
    m
}

fn pt_br() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Selecionar");
    m.insert("toolbar.rectangle", "Retângulo");
    m.insert("toolbar.diamond", "Losango");
    m.insert("toolbar.ellipse", "Elipse");
    m.insert("toolbar.arrow", "Seta");
    m.insert("toolbar.line", "Linha");
    m.insert("toolbar.freedraw", "Desenhar");
    m.insert("toolbar.text", "Texto");
    m.insert("action.undo", "Desfazer");
    m.insert("action.redo", "Refazer");
    m.insert("action.delete", "Excluir");
    m.insert("action.exportPng", "Exportar como PNG");
    m.insert("action.exportSvg", "Exportar como SVG");
    m.insert("action.zoomIn", "Ampliar");
    m.insert("action.zoomOut", "Reduzir");
    m.insert("action.toggleTheme", "Alternar tema");
    m.insert("menu.language", "Idioma");
    m.insert("theme.light", "Claro");
    m.insert("theme.dark", "Escuro");
    m.insert("action.group", "Agrupar");
    m.insert("action.ungroup", "Desagrupar");
    m.insert("ctx.bringFront", "Trazer para a frente");
    m.insert("ctx.copy", "Copiar");
    m.insert("ctx.delete", "Eliminar");
    m.insert("ctx.duplicate", "Duplicar");
    m.insert("ctx.paste", "Colar");
    m.insert("ctx.sendBack", "Enviar para trás");
    m.insert("hint.moveCanvas", "Para mover a tela, segure a {{roda do mouse}} ou {{espaço}} enquanto arrasta, ou use a ferramenta de mão");
    m.insert("library.addSelected", "Adicionar à biblioteca");
    m.insert("library.empty", "Sua biblioteca está vazia");
    m.insert("menu.exportImage", "Exportar imagem");
    m.insert("menu.exportSvg", "Exportar para SVG");
    m.insert("menu.grid", "Mostrar grade");
    m.insert("menu.help", "Ajuda");
    m.insert(
        "menu.helpHint",
        "Arraste para desenhar. Escolha uma ferramenta na barra ou pressione sua tecla.",
    );
    m.insert("menu.layers", "Camadas");
    m.insert("menu.open", "Abrir");
    m.insert("menu.reset", "Redefinir a tela");
    m.insert("menu.save", "Salvar em…");
    m.insert("menu.snap", "Alinhar à grade");
    m.insert("panel.layers", "Camadas");
    m.insert("panel.library", "Biblioteca");
    m.insert("menu.canvasBackground", "Fundo da tela");
    m.insert("menu.followUs", "Siga-nos");
    m.insert("menu.discordGroup", "Chat do Discord");
    m.insert("menu.save", "Salvar em...");
    m.insert("menu.exportImage", "Exportar imagem...");
    m.insert("menu.collab", "Colaboração ao vivo...");
    m.insert("menu.commandPalette", "Paleta de comandos");
    m.insert("menu.findOnCanvas", "Localizar na tela");
    m.insert("toolbar.hand", "Ferramenta mão");
    m.insert("toolbar.image", "Inserir imagem");
    m.insert("toolbar.frame", "Ferramenta de quadro");
    m.insert("toolbar.eraser", "Borracha");
    m.insert("toolbar.laser", "Ponteiro de laser");
    m.insert("action.duplicate", "Duplicar");
    m.insert("ctx.selectAll", "Selecionar tudo");
    m.insert(
        "action.copyToClipboard",
        "Copiar para a área de transferência",
    );
    m.insert("ctx.cut", "Recortar");
    m.insert("action.resetZoom", "Redefinir zoom");
    m.insert("action.toggleGrid", "Alternar grade");
    m.insert("action.zoomToFit", "Ajustar à tela");
    m.insert("palette.placeholder", "Digite um comando…");
    m.insert("palette.noResults", "Nenhum resultado encontrado");
    m.insert("find.placeholder", "Procurar texto na tela");
    m.insert("find.noResults", "Sem resultados");
    m.insert("help.tools", "Ferramentas");
    m.insert("help.actions", "Ações");
    m.insert("help.view", "Visualização");
    m.insert("help.files", "Arquivos");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferências");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Bloqueio de ferramenta");
    m.insert("menu.snapToObjects", "Ajustar aos objetos");
    m.insert("menu.zenMode", "Modo Zen");
    m.insert("menu.viewMode", "Modo de visualização");
    m.insert("menu.canvasStats", "Propriedades da tela e das formas");
    m.insert("menu.arrowBinding", "Vinculação de setas");
    m.insert("menu.snapToMidpoints", "Ajustar aos pontos médios");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Página web incorporada");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Seleção de laço");
    m.insert("toolbar.textToDiagram", "Texto para diagrama");
    m.insert("toolbar.mermaid", "Mermaid para Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe para código");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Seus desenhos ficam salvos no armazenamento do seu navegador.",
    );
    m.insert(
        "welcome.center2",
        "Esse armazenamento pode ser apagado sem querer.",
    );
    m.insert(
        "welcome.center3",
        "Salve seu trabalho em um arquivo de vez em quando para não perdê-lo.",
    );
    m.insert("stats.title", "Propriedades");
    m.insert("stats.shapes", "Formas");
    m.insert("stats.selectedType", "Tipo");
    m.insert("status.saved", "Salvo");
    m.insert(
        "status.unavailable",
        "Requer o backend do Excalidraw — indisponível nesta versão",
    );
    m.insert("labels.convertToCode", "Converter para código");
    m.insert(
        "labels.copySource",
        "Copiar origem para área de transferência",
    );
    m.insert("fileDrop.replace", "Solte para substituir o conteúdo");
    m.insert("fileDrop.add", "Solte para adicionar à tela");
    m.insert("fileDrop.importLibrary", "Solte para importar a biblioteca");
    m.insert(
        "fileDrop.replaceHint",
        "Segure {{Shift}} para manter o conteúdo existente",
    );
    m.insert("fileDrop.keepHint", "Seu conteúdo existente será mantido");
    m.insert(
        "fileDrop.libraryHint",
        "Arquivos da biblioteca serão adicionados",
    );
    m.insert(
        "fileDrop.replacedToast",
        "Conteúdo substituído. {{Ctrl+Z}} para desfazer",
    );
    m.insert("colorPicker.colors", "Cores");
    m.insert("colorPicker.shades", "Tons");
    m.insert("colorPicker.noShades", "Sem tons disponíveis para essa cor");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Cores personalizadas mais usadas",
    );
    m.insert("colorPicker.hexCode", "Código hexadecimal");
    m.insert("fontList.sceneFonts", "Neste cenário");
    m.insert("fontList.availableFonts", "Fontes disponíveis");
    m.insert("fontList.empty", "Nenhuma fonte encontrada");
    m.insert("quickSearch.placeholder", "Busca rápida");
    m.insert("elementLink.title", "Links para o objeto");
    m.insert(
        "elementLink.desc",
        "Clique em uma forma na tela ou cole um link.",
    );
    m.insert("buttons.remove", "Remover");
    m.insert("buttons.cancel", "Cancelar");
    m.insert("buttons.confirm", "Confirmar");
    m
}

fn ru() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Выбрать");
    m.insert("toolbar.rectangle", "Прямоугольник");
    m.insert("toolbar.diamond", "Ромб");
    m.insert("toolbar.ellipse", "Эллипс");
    m.insert("toolbar.arrow", "Стрелка");
    m.insert("toolbar.line", "Линия");
    m.insert("toolbar.freedraw", "Рисовать");
    m.insert("toolbar.text", "Текст");
    m.insert("action.undo", "Отменить");
    m.insert("action.redo", "Повторить");
    m.insert("action.delete", "Удалить");
    m.insert("action.exportPng", "Экспорт в PNG");
    m.insert("action.exportSvg", "Экспорт в SVG");
    m.insert("action.zoomIn", "Приблизить");
    m.insert("action.zoomOut", "Отдалить");
    m.insert("action.toggleTheme", "Переключить тему");
    m.insert("menu.language", "Язык");
    m.insert("theme.light", "Светлая");
    m.insert("theme.dark", "Тёмная");
    m.insert("action.group", "Группировать");
    m.insert("action.ungroup", "Разгруппировать");
    m.insert("ctx.bringFront", "На передний план");
    m.insert("ctx.copy", "Копировать");
    m.insert("ctx.delete", "Удалить");
    m.insert("ctx.duplicate", "Дублировать");
    m.insert("ctx.paste", "Вставить");
    m.insert("ctx.sendBack", "На задний план");
    m.insert("hint.moveCanvas", "Чтобы переместить холст, удерживайте {{колесо мыши}} или {{пробел}} при перетаскивании или используйте инструмент «Рука»");
    m.insert("library.addSelected", "Добавить в библиотеку");
    m.insert("library.empty", "Ваша библиотека пуста");
    m.insert("menu.exportImage", "Экспортировать изображение");
    m.insert("menu.exportSvg", "Экспорт в SVG");
    m.insert("menu.grid", "Показать сетку");
    m.insert("menu.help", "Справка");
    m.insert(
        "menu.helpHint",
        "Рисуйте перетаскиванием. Выберите инструмент на панели или нажмите его клавишу.",
    );
    m.insert("menu.layers", "Слои");
    m.insert("menu.open", "Открыть");
    m.insert("menu.reset", "Сбросить холст");
    m.insert("menu.save", "Сохранить в…");
    m.insert("menu.snap", "Привязать к сетке");
    m.insert("panel.layers", "Слои");
    m.insert("panel.library", "Библиотека");
    m.insert("menu.canvasBackground", "Фон холста");
    m.insert("menu.followUs", "Подписаться");
    m.insert("menu.discordGroup", "Discord-чат");
    m.insert("menu.save", "Сохранить в...");
    m.insert("menu.exportImage", "Экспортировать изображение...");
    m.insert("menu.collab", "Совместная работа...");
    m.insert("menu.commandPalette", "Палитра команд");
    m.insert("menu.findOnCanvas", "Найти на холсте");
    m.insert("toolbar.hand", "Инструмент рука");
    m.insert("toolbar.image", "Вставить изображение");
    m.insert("toolbar.frame", "Фреймовый инструмент");
    m.insert("toolbar.eraser", "Ластик");
    m.insert("toolbar.laser", "Лазерная указка");
    m.insert("action.duplicate", "Дублировать");
    m.insert("ctx.selectAll", "Выбрать всё");
    m.insert("action.copyToClipboard", "Копировать в буфер обмена");
    m.insert("ctx.cut", "Вырезать");
    m.insert("action.resetZoom", "Сбросить масштаб");
    m.insert("action.toggleGrid", "Переключить сетку");
    m.insert("action.zoomToFit", "По размеру экрана");
    m.insert("palette.placeholder", "Введите команду…");
    m.insert("palette.noResults", "Ничего не найдено");
    m.insert("find.placeholder", "Искать текст на холсте");
    m.insert("find.noResults", "Нет результатов");
    m.insert("help.tools", "Инструменты");
    m.insert("help.actions", "Действия");
    m.insert("help.view", "Вид");
    m.insert("help.files", "Файлы");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Настройки");
    m.insert("menu.theme", "Тема");
    m.insert("menu.toolLock", "Блокировка инструмента");
    m.insert("menu.snapToObjects", "Привязка к объектам");
    m.insert("menu.zenMode", "Дзен-режим");
    m.insert("menu.viewMode", "Режим просмотра");
    m.insert("menu.canvasStats", "Свойства холста и фигур");
    m.insert("menu.arrowBinding", "Привязка стрелок");
    m.insert("menu.snapToMidpoints", "Привязка к серединам");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Встроенная страница");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Выделение лассо");
    m.insert("toolbar.textToDiagram", "Текст в диаграмму");
    m.insert("toolbar.mermaid", "Mermaid в Excalidraw");
    m.insert("toolbar.wireframeToCode", "Каркас для кода");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Ваши рисунки сохраняются в хранилище браузера.",
    );
    m.insert(
        "welcome.center2",
        "Хранилище браузера может быть случайно очищено.",
    );
    m.insert(
        "welcome.center3",
        "Периодически сохраняйте работу в файл, чтобы ничего не потерять.",
    );
    m.insert("stats.title", "Свойства");
    m.insert("stats.shapes", "Фигуры");
    m.insert("stats.selectedType", "Тип");
    m.insert("status.saved", "Сохранено");
    m.insert(
        "status.unavailable",
        "Требуется бэкенд Excalidraw — недоступно в этой сборке",
    );
    m.insert("labels.convertToCode", "Преобразовать в код");
    m.insert("labels.copySource", "Скопировать источник в буфер обмена");
    m.insert("fileDrop.replace", "Отпустите, чтобы заменить содержимое");
    m.insert("fileDrop.add", "Отпустите, чтобы добавить на холст");
    m.insert(
        "fileDrop.importLibrary",
        "Отпустите, чтобы импортировать библиотеку",
    );
    m.insert(
        "fileDrop.replaceHint",
        "Удерживайте {{Shift}}, чтобы сохранить текущее содержимое",
    );
    m.insert("fileDrop.keepHint", "Текущее содержимое будет сохранено");
    m.insert("fileDrop.libraryHint", "Файлы библиотеки будут добавлены");
    m.insert(
        "fileDrop.replacedToast",
        "Содержимое заменено. {{Ctrl+Z}} для отмены",
    );
    m.insert("colorPicker.colors", "Цвета");
    m.insert("colorPicker.shades", "Оттенки");
    m.insert(
        "colorPicker.noShades",
        "Нет доступных оттенков для этого цвета",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Часто используемые пользовательские цвета",
    );
    m.insert("colorPicker.hexCode", "Шестнадцатеричный код");
    m.insert("fontList.sceneFonts", "В этой сцене");
    m.insert("fontList.availableFonts", "Доступные шрифты");
    m.insert("fontList.empty", "Шрифты не найдены");
    m.insert("quickSearch.placeholder", "Быстрый поиск");
    m.insert("elementLink.title", "Ссылка на объект");
    m.insert(
        "elementLink.desc",
        "Нажмите на фигуру на холсте или вставьте ссылку.",
    );
    m.insert("buttons.remove", "Удалить");
    m.insert("buttons.cancel", "Отменить");
    m.insert("buttons.confirm", "Подтвердить");
    m
}

fn ar_sa() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "اختر");
    m.insert("toolbar.hand", "يد (أداة الإزاحة)");
    m.insert("toolbar.rectangle", "مستطيل");
    m.insert("toolbar.diamond", "مضلع");
    m.insert("toolbar.ellipse", "دائرة");
    m.insert("toolbar.arrow", "سهم");
    m.insert("toolbar.line", "خط");
    m.insert("toolbar.freedraw", "رسم");
    m.insert("toolbar.text", "نص");
    m.insert("toolbar.image", "إدراج صورة");
    m.insert("toolbar.frame", "أداة الإطار");
    m.insert("toolbar.eraser", "ممحاة");
    m.insert("toolbar.laser", "مؤشر ليزر");
    m.insert("toolbar.lock", "قفل");
    m.insert("toolbar.library", "مكتبة");
    m.insert("action.undo", "تراجع");
    m.insert("action.redo", "إعادة تنفيذ");
    m.insert("action.delete", "حذف");
    m.insert("action.duplicate", "تكرار");
    m.insert("action.exportPng", "تصدير بصيغة PNG");
    m.insert("action.exportSvg", "تصدير بصيغة SVG");
    m.insert("action.copyToClipboard", "نسخ إلى الحافظة");
    m.insert("action.zoomIn", "تكبير");
    m.insert("action.zoomOut", "تصغير");
    m.insert("action.resetZoom", "إعادة تعيين التكبير");
    m.insert("action.toggleTheme", "تبديل الوضع الفاتح/الداكن");
    m.insert("action.toggleGrid", "تبديل الشبكة");
    m.insert("panel.layers", "الطبقات");
    m.insert("panel.properties", "الخصائص");
    m.insert("panel.library", "مكتبة");
    m.insert("menu.language", "اللغة");
    m.insert("menu.open", "فتح");
    m.insert("menu.save", "حفظ إلى...");
    m.insert("menu.exportImage", "تصدير الصورة...");
    m.insert("menu.exportSvg", "تصدير بصيغة SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "تبديل الشبكة");
    m.insert("menu.layers", "الطبقات");
    m.insert("menu.reset", "إعادة تعيين اللوحة");
    m.insert("menu.help", "المساعدة");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "القائمة");
    m.insert("menu.new", "New");
    m.insert("menu.load", "فتح");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "تحديد مجموعة");
    m.insert("action.ungroup", "إلغاء تحديد مجموعة");
    m.insert("ctx.duplicate", "تكرار");
    m.insert("ctx.delete", "حذف");
    m.insert("ctx.copy", "نسخ");
    m.insert("ctx.cut", "قص");
    m.insert("ctx.paste", "لصق");
    m.insert("ctx.selectAll", "تحديد الكل");
    m.insert("ctx.bringFront", "أحضر للأمام");
    m.insert("ctx.sendBack", "أرسل للخلف");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "لا شيء");
    m.insert("ah.arrow", "سهم");
    m.insert("ah.bar", "شريط");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "مثلث");
    m.insert("ah.diamond", "ألماسة");
    m.insert("theme.light", "الوضع المضيء");
    m.insert("theme.dark", "الوضع المظلم");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "لتحريك القماشة، اضغط مطولًا على {{Scroll wheel}} أو {{Space}} أثناء السحب، أو استخدم الأداة اليدوية");
    m.insert("library.addSelected", "أضف إلى المكتبة");
    m.insert("library.empty", "لا توجد عناصر أضيفت بعد...");
    m.insert("color.stroke", "الخط");
    m.insert("color.background", "الخلفية");
    m.insert("prop.strokeWidth", "سُمك الخط");
    m.insert("prop.fillStyle", "التعبئة");
    m.insert("prop.opacity", "الشفافية");
    m.insert("prop.roughness", "الإمالة");
    m.insert("prop.fontSize", "حجم الخط");
    m.insert("prop.fontFamily", "نوع الخط");
    m.insert("prop.textAlign", "محاذاة النص");
    m.insert("prop.strokeStyle", "نمط الخط");
    m.insert("prop.arrowheads", "رؤوس الأسهم");
    m.insert("prop.roundness", "الحواف");
    m.insert("prop.lock", "قفل");
    m.insert("prop.unlock", "فتح");
    m.insert("prop.group", "مجموعة");
    m.insert("prop.ungroup", "إلغاء تحديد مجموعة");
    m.insert("prop.align", "محاذاة");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "محاذاة إلى اليسار");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "محاذاة إلى اليمين");
    m.insert("align.top", "محاذاة إلى الأعلى");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "محاذاة إلى الأسفل");
    m.insert("fill.hachure", "خطوط");
    m.insert("fill.solid", "كامل");
    m.insert("fill.zigzag", "متعرج");
    m.insert("fill.crossHatch", "خطوط متقطعة");
    m.insert("stroke.solid", "متصل");
    m.insert("stroke.dashed", "متقطع");
    m.insert("stroke.dotted", "منقط");
    m.insert("round.none", "لا شيء");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "الـيسار");
    m.insert("textalign.center", "وسط");
    m.insert("textalign.right", "يمين");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "خلفية اللوحة");
    m.insert("menu.followUs", "تابعنا");
    m.insert("menu.discordGroup", "دردشة ديسكورد");
    m.insert("menu.collab", "التعاون المباشر...");
    m.insert("menu.commandPalette", "لوحة الأوامر");
    m.insert("menu.findOnCanvas", "البحث على اللوحة");
    m.insert("action.zoomToFit", "تكبير لتلائم جميع العناصر");
    m.insert(
        "palette.placeholder",
        "ابحث في القوائم، الأوامر، واكتشف الجواهر المخفية",
    );
    m.insert("palette.noResults", "لا توجد أوامر مطابقة...");
    m.insert("find.placeholder", "ابحث عن نص على اللوحة...");
    m.insert("find.noResults", "لم يتم العثور على تطابقات...");
    m.insert("help.tools", "الأدوات");
    m.insert("help.actions", "الإجراءات");
    m.insert("help.view", "عرض");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferences");
    m.insert("menu.theme", "السمة");
    m.insert("menu.toolLock", "Tool lock");
    m.insert("menu.snapToObjects", "التقاط إلى العناصر");
    m.insert("menu.zenMode", "وضع التأمل");
    m.insert("menu.viewMode", "نمط العرض");
    m.insert("menu.canvasStats", "خصائص اللوحة والأشكال");
    m.insert("menu.arrowBinding", "Arrow binding");
    m.insert("menu.snapToMidpoints", "Snap to midpoints");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "تضمين ويب");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "اختيار لاسو");
    m.insert("toolbar.textToDiagram", "نص إلى رسم بياني");
    m.insert("toolbar.mermaid", "من Mermaid إلى Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe إلى كود");
    m.insert("toolbar.generate", "Generate");
    m.insert("welcome.center1", "رسوماتك محفوظة في تخزين متصفحك.");
    m.insert("welcome.center2", "يُمكن محُ تخزين المتصفح بشكل غير متوقع.");
    m.insert("welcome.center3", "احفظ عملك في ملف بانتظام لتجنب فقدانه.");
    m.insert("stats.title", "الخصائص");
    m.insert("stats.shapes", "الأشكال");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "تحويل إلى كود");
    m.insert("labels.copySource", "نسخ المصدر إلى الحافظة");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "الألوان");
    m.insert("colorPicker.shades", "الدرجات");
    m.insert("colorPicker.noShades", "لا تتوفر درجات لهذا اللون");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "الألوان المخصصة الأكثر استخدامًا",
    );
    m.insert("colorPicker.hexCode", "رمز Hex");
    m.insert("fontList.sceneFonts", "في هذا المشهد");
    m.insert("fontList.availableFonts", "الخطوط المتوفرة");
    m.insert("fontList.empty", "لم يتم العثور على خطوط");
    m.insert("quickSearch.placeholder", "بحث سريع");
    m.insert("elementLink.title", "رابط إلى الكائن");
    m.insert("elementLink.desc", "انقر على شكل على اللوحة أو الصق رابطًا.");
    m.insert("buttons.remove", "إزالة");
    m.insert("buttons.cancel", "إلغاء");
    m.insert("buttons.confirm", "تأكيد");
    m
}

fn eu_es() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Hautatu");
    m.insert("toolbar.hand", "Eskua (panoratze tresna)");
    m.insert("toolbar.rectangle", "Laukizuzena");
    m.insert("toolbar.diamond", "Erromboa");
    m.insert("toolbar.ellipse", "Elipsea");
    m.insert("toolbar.arrow", "Gezia");
    m.insert("toolbar.line", "Lerroa");
    m.insert("toolbar.freedraw", "Marraztu");
    m.insert("toolbar.text", "Testua");
    m.insert("toolbar.image", "Txertatu irudia");
    m.insert("toolbar.frame", "Marko tresna");
    m.insert("toolbar.eraser", "Borragoma");
    m.insert("toolbar.laser", "Laser punteroa");
    m.insert("toolbar.lock", "Blokeatu");
    m.insert("toolbar.library", "Liburutegia");
    m.insert("action.undo", "Desegin");
    m.insert("action.redo", "Berregin");
    m.insert("action.delete", "Ezabatu");
    m.insert("action.duplicate", "Bikoiztu");
    m.insert("action.exportPng", "Esportatu PNG gisa");
    m.insert("action.exportSvg", "Esportatu SVG gisa");
    m.insert("action.copyToClipboard", "Kopiatu arbelera");
    m.insert("action.zoomIn", "Handiagotu");
    m.insert("action.zoomOut", "Txikiagotu");
    m.insert("action.resetZoom", "Leheneratu zooma");
    m.insert("action.toggleTheme", "Gaia argia/iluna aktibatu");
    m.insert("action.toggleGrid", "Txandakatu sareta");
    m.insert("panel.layers", "Geruzak");
    m.insert("panel.properties", "Propietateak");
    m.insert("panel.library", "Liburutegia");
    m.insert("menu.language", "Hizkuntza");
    m.insert("menu.open", "Ireki");
    m.insert("menu.save", "Gorde hemen...");
    m.insert("menu.exportImage", "Esportatu irudia...");
    m.insert("menu.exportSvg", "Esportatu SVG gisa");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Txandakatu sareta");
    m.insert("menu.layers", "Geruzak");
    m.insert("menu.reset", "Garbitu oihala");
    m.insert("menu.help", "Laguntza");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menua");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Ireki");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Hautapena taldea bihurtu");
    m.insert("action.ungroup", "Desegin hautapenaren taldea");
    m.insert("ctx.duplicate", "Bikoiztu");
    m.insert("ctx.delete", "Ezabatu");
    m.insert("ctx.copy", "Kopiatu");
    m.insert("ctx.cut", "Ebaki");
    m.insert("ctx.paste", "Itsatsi");
    m.insert("ctx.selectAll", "Hautatu dena");
    m.insert("ctx.bringFront", "Ekarri aurrera");
    m.insert("ctx.sendBack", "Eraman atzera");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Bat ere ez");
    m.insert("ah.arrow", "Gezia");
    m.insert("ah.bar", "Barra");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Hirukia");
    m.insert("ah.diamond", "Erromboa");
    m.insert("theme.light", "Modu argia");
    m.insert("theme.dark", "Modu iluna");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Oihala mugitzeko heldu {{Scroll wheel}} edo {{Space}} arrastatu bitartean edo erabili esku-tresna");
    m.insert("library.addSelected", "Gehitu liburutegira");
    m.insert("library.empty", "Oraindik ez da elementurik gehitu...");
    m.insert("color.stroke", "Marra");
    m.insert("color.background", "Atzeko planoa");
    m.insert("prop.strokeWidth", "Marraren zabalera");
    m.insert("prop.fillStyle", "Bete");
    m.insert("prop.opacity", "Opakotasuna");
    m.insert("prop.roughness", "Marraren trazoa");
    m.insert("prop.fontSize", "Letra-tamaina");
    m.insert("prop.fontFamily", "Letra-tipoa");
    m.insert("prop.textAlign", "Testuaren lerrokapena");
    m.insert("prop.strokeStyle", "Marraren estiloa");
    m.insert("prop.arrowheads", "Gezi-puntak");
    m.insert("prop.roundness", "Ertzak");
    m.insert("prop.lock", "Blokeatu");
    m.insert("prop.unlock", "Desblokeatu");
    m.insert("prop.group", "Taldea");
    m.insert("prop.ungroup", "Desegin hautapenaren taldea");
    m.insert("prop.align", "Lerrokatu");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Lerrokatu ezkerrean");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Lerrokatu eskuinean");
    m.insert("align.top", "Lerrokatu goian");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Lerrokatu behean");
    m.insert("fill.hachure", "Itzalduna");
    m.insert("fill.solid", "Solidoa");
    m.insert("fill.zigzag", "Sigi-saga");
    m.insert("fill.crossHatch", "Marraduna");
    m.insert("stroke.solid", "Solidoa");
    m.insert("stroke.dashed", "Marratua");
    m.insert("stroke.dotted", "Puntukatua");
    m.insert("round.none", "Bat ere ez");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Ezkerrean");
    m.insert("textalign.center", "Erdian");
    m.insert("textalign.right", "Eskuinean");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Oihalaren atzeko planoa");
    m.insert("menu.followUs", "Jarraitu gaitzazu");
    m.insert("menu.discordGroup", "Discord txata");
    m.insert("menu.collab", "Zuzeneko elkarlana...");
    m.insert("menu.commandPalette", "Komandoen paleta");
    m.insert("menu.findOnCanvas", "Bilatu oihalean");
    m.insert("action.zoomToFit", "Egin zoom elementu guztiak ikusteko");
    m.insert(
        "palette.placeholder",
        "Bilatu menuak, komandoak, eta aurkitu ezkutuko harribitxiak",
    );
    m.insert("palette.noResults", "Ez da komandorik aurkitu...");
    m.insert("find.placeholder", "Bilatu testua oihalean...");
    m.insert("find.noResults", "Ez da ezer aurkitu...");
    m.insert("help.tools", "Tresnak");
    m.insert("help.actions", "Ekintzak");
    m.insert("help.view", "Bistaratu");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Hobespenak");
    m.insert("menu.theme", "Itxura");
    m.insert("menu.toolLock", "Erremintaren blokeoa");
    m.insert("menu.snapToObjects", "Atxiki objektuei");
    m.insert("menu.zenMode", "Zen modua");
    m.insert("menu.viewMode", "Ikuspegia");
    m.insert("menu.canvasStats", "Oihal eta formen propietateak");
    m.insert("menu.arrowBinding", "Gezi azala");
    m.insert("menu.snapToMidpoints", "Lotu erdiko puntuetara");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web kapsulatzea");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Lazo hautaketa");
    m.insert("toolbar.textToDiagram", "Testutik diagramara");
    m.insert("toolbar.mermaid", "Mermaid-etik Excalidraw-ra");
    m.insert("toolbar.wireframeToCode", "Wireframetik kodera");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Zure marrazkiak zure nabigatzailearen biltegian gordetzen dira.",
    );
    m.insert(
        "welcome.center2",
        "Nabigatzailearen biltegia ustekabean hustu daiteke.",
    );
    m.insert(
        "welcome.center3",
        "Gorde zure lana maiz fitxategi batean, ez galtzeko.",
    );
    m.insert("stats.title", "Propietateak");
    m.insert("stats.shapes", "Formak");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Kodera bihurtu");
    m.insert("labels.copySource", "Kopiatu jatorria arbelean");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Koloreak");
    m.insert("colorPicker.shades", "Ñabardurak");
    m.insert(
        "colorPicker.noShades",
        "Kolore honetarako ez dago ñabardurarik eskuragarri",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Gehien erabilitako kolore pertsonalizatuak",
    );
    m.insert("colorPicker.hexCode", "Hez kodea");
    m.insert("fontList.sceneFonts", "Eszena honetan");
    m.insert("fontList.availableFonts", "Letra mota erabilgarriak");
    m.insert("fontList.empty", "Ez da letra motarik aurkitu");
    m.insert("quickSearch.placeholder", "Bilaketa azkarra");
    m.insert("elementLink.title", "Estekatu objektura");
    m.insert(
        "elementLink.desc",
        "Egin klik forma batean edo itsatsi esteka bat.",
    );
    m.insert("buttons.remove", "Kendu");
    m.insert("buttons.cancel", "Utzi");
    m.insert("buttons.confirm", "Bai");
    m
}

fn fa_ir() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "انتخاب");
    m.insert("toolbar.hand", "دست (ابزار پانینگ)");
    m.insert("toolbar.rectangle", "مستطیل");
    m.insert("toolbar.diamond", "لوزی");
    m.insert("toolbar.ellipse", "بیضی");
    m.insert("toolbar.arrow", "پیکان");
    m.insert("toolbar.line", "خط");
    m.insert("toolbar.freedraw", "کشیدن");
    m.insert("toolbar.text", "متن");
    m.insert("toolbar.image", "وارد کردن تصویر");
    m.insert("toolbar.frame", "ابزار فریم");
    m.insert("toolbar.eraser", "پاک کن");
    m.insert("toolbar.laser", "اشاره گر لیزری");
    m.insert("toolbar.lock", "قفل");
    m.insert("toolbar.library", "کتابخانه");
    m.insert("action.undo", "بازگرد");
    m.insert("action.redo", "از سر");
    m.insert("action.delete", "حذف");
    m.insert("action.duplicate", "تکراری");
    m.insert("action.exportPng", "تبدیل به PNG");
    m.insert("action.exportSvg", "تبدیل به SVG");
    m.insert("action.copyToClipboard", "کپی در حافظه موقت");
    m.insert("action.zoomIn", "بزرگ نمایی");
    m.insert("action.zoomOut", "کوچک نمایی");
    m.insert("action.resetZoom", "اندازه اصلی");
    m.insert("action.toggleTheme", "تغییر وضعیت پوسته روشن/تیره");
    m.insert("action.toggleGrid", "تغییر وضعیت خطوط شطرنجی");
    m.insert("panel.layers", "لایه ها");
    m.insert("panel.properties", "ویژگی‌ها");
    m.insert("panel.library", "کتابخانه");
    m.insert("menu.language", "زبان");
    m.insert("menu.open", "باز کردن");
    m.insert("menu.save", "ذخیره در...");
    m.insert("menu.exportImage", "خروجی گرفتن از تصویر...");
    m.insert("menu.exportSvg", "تبدیل به SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "تغییر وضعیت خطوط شطرنجی");
    m.insert("menu.layers", "لایه ها");
    m.insert("menu.reset", "پاکسازی بوم نقاشی");
    m.insert("menu.help", "راهنما");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "فهرست");
    m.insert("menu.new", "New");
    m.insert("menu.load", "باز کردن");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "گروهبندی انتخابها");
    m.insert("action.ungroup", "حذف گروهبندی انتخابها");
    m.insert("ctx.duplicate", "تکراری");
    m.insert("ctx.delete", "حذف");
    m.insert("ctx.copy", "کپی");
    m.insert("ctx.cut", "بریدن");
    m.insert("ctx.paste", "جایگذاری");
    m.insert("ctx.selectAll", "انتخاب همه");
    m.insert("ctx.bringFront", "جلو آوردن");
    m.insert("ctx.sendBack", "ارسال به عقب");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "هیچ کدام");
    m.insert("ah.arrow", "پیکان");
    m.insert("ah.bar", "میله ای");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "مثلث");
    m.insert("ah.diamond", "الماس");
    m.insert("theme.light", "حالت روشن");
    m.insert("theme.dark", "حالت تیره");
    m.insert("stats.elements", "Elements");
    m.insert(
        "hint.moveCanvas",
        "To move canvas, hold {{Scroll wheel}} or {{Space}} while dragging, or use the hand tool",
    );
    m.insert("library.addSelected", "افزودن به کتابخانه");
    m.insert("library.empty", "آیتمی به اینجا اضافه نشده...");
    m.insert("color.stroke", "حاشیه");
    m.insert("color.background", "پس زمینه");
    m.insert("prop.strokeWidth", "ضخامت حاشیه");
    m.insert("prop.fillStyle", "رنگ آمیزی");
    m.insert("prop.opacity", "شفافیت");
    m.insert("prop.roughness", "دقت");
    m.insert("prop.fontSize", "اندازه قلم");
    m.insert("prop.fontFamily", "نوع قلم");
    m.insert("prop.textAlign", "تراز متن");
    m.insert("prop.strokeStyle", "استایل حاشیه");
    m.insert("prop.arrowheads", "سر پیکان");
    m.insert("prop.roundness", "لبه ها");
    m.insert("prop.lock", "قفل");
    m.insert("prop.unlock", "باز کردن");
    m.insert("prop.group", "گروه");
    m.insert("prop.ungroup", "حذف گروهبندی انتخابها");
    m.insert("prop.align", "تراز");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "تراز به چپ");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "تراز به راست");
    m.insert("align.top", "تراز به بالا");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "تراز به پایین");
    m.insert("fill.hachure", "هاشور");
    m.insert("fill.solid", "توپر");
    m.insert("fill.zigzag", "زیگزاگ");
    m.insert("fill.crossHatch", "هاشور متقاطع");
    m.insert("stroke.solid", "یکدست");
    m.insert("stroke.dashed", "خط چین");
    m.insert("stroke.dotted", "نقطه چین");
    m.insert("round.none", "هیچ کدام");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "چپ");
    m.insert("textalign.center", "وسط");
    m.insert("textalign.right", "راست");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "پس‌زمینه بوم");
    m.insert("menu.followUs", "ما را دنبال کنید");
    m.insert("menu.discordGroup", "چت دیسکورد");
    m.insert("menu.collab", "همکاری آنلاین...");
    m.insert("menu.commandPalette", "تخته فرمان‌ها");
    m.insert("menu.findOnCanvas", "جستجو در بوم");
    m.insert("action.zoomToFit", "بزرگنمایی برای دیدن تمام آیتم ها");
    m.insert(
        "palette.placeholder",
        "در فهرست‌ها و دستورها را جستجو کنید هم سنگهای پنهان را کشف کنید",
    );
    m.insert("palette.noResults", "دستوری یافت نشد...");
    m.insert("find.placeholder", "جستجوی متن در بوم...");
    m.insert("find.noResults", "هیچ موردی یافت نشد...");
    m.insert("help.tools", "ابزار");
    m.insert("help.actions", "عملیات");
    m.insert("help.view", "مشاهده");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "ثبت‌نام");
    m.insert("menu.preferences", "ترجیحات");
    m.insert("menu.theme", "پوسته");
    m.insert("menu.toolLock", "قفل ابزار");
    m.insert("menu.snapToObjects", "به اشیاء بچسبد");
    m.insert("menu.zenMode", "حالت ذن");
    m.insert("menu.viewMode", "حالت نمایش");
    m.insert("menu.canvasStats", "ویژگی‌های بوم و شکل");
    m.insert("menu.arrowBinding", "اتصال پیکان");
    m.insert("menu.snapToMidpoints", "چسبیدن به نقاط میانی");
    m.insert("prefs.selectOn", "انتخاب بر اساس");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "افزونه وب");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Lasso selection");
    m.insert("toolbar.textToDiagram", "متن به دیاگرام");
    m.insert("toolbar.mermaid", "مرمید به excalidraw");
    m.insert("toolbar.wireframeToCode", "وایرفریم به کد");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Your drawings are saved in your browser storage.",
    );
    m.insert(
        "welcome.center2",
        "Your browser storage might be accidentally cleared.",
    );
    m.insert(
        "welcome.center3",
        "Save your work to a file from time to time to avoid losing it.",
    );
    m.insert("stats.title", "ویژگی‌ها");
    m.insert("stats.shapes", "اشکال");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "تبدیل به کد");
    m.insert("labels.copySource", "کپی منبع در حافظه موقت");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "رنگ‌ها");
    m.insert("colorPicker.shades", "جلوه‌ها");
    m.insert(
        "colorPicker.noShades",
        "هیچ سایه ای برای این رنگ در دسترس نیست",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "رنگ های به‌تازگی به‌کار گرفته شده",
    );
    m.insert("colorPicker.hexCode", "کدِ هگز");
    m.insert("fontList.sceneFonts", "در این صحنه");
    m.insert("fontList.availableFonts", "قلم های در دسترس");
    m.insert("fontList.empty", "قلمی یافت نشد");
    m.insert("quickSearch.placeholder", "جستجو فوری");
    m.insert("elementLink.title", "لینک به آیتم");
    m.insert(
        "elementLink.desc",
        "روی شکل یا بوم کلیک کنید، یا لینک را جایگذاری کنید.",
    );
    m.insert("buttons.remove", "پاک کردن");
    m.insert("buttons.cancel", "لغو");
    m.insert("buttons.confirm", "تایید");
    m
}

fn he_il() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "בחירה");
    m.insert("toolbar.hand", "יד (כלי הזזה)");
    m.insert("toolbar.rectangle", "מלבן");
    m.insert("toolbar.diamond", "יהלום");
    m.insert("toolbar.ellipse", "אליפסה");
    m.insert("toolbar.arrow", "חץ");
    m.insert("toolbar.line", "קו");
    m.insert("toolbar.freedraw", "צייר");
    m.insert("toolbar.text", "טקסט");
    m.insert("toolbar.image", "הוספת תמונה");
    m.insert("toolbar.frame", "כלי מסגרת");
    m.insert("toolbar.eraser", "מחק");
    m.insert("toolbar.laser", "סמן לייזר");
    m.insert("toolbar.lock", "נעילה");
    m.insert("toolbar.library", "ספריה");
    m.insert("action.undo", "ביטול");
    m.insert("action.redo", "ביצוע מחדש");
    m.insert("action.delete", "מחיקה");
    m.insert("action.duplicate", "שכפול");
    m.insert("action.exportPng", "ייצוא ל־PNG");
    m.insert("action.exportSvg", "ייצוא ל־SVG");
    m.insert("action.copyToClipboard", "העתקה ללוח");
    m.insert("action.zoomIn", "התקרבות");
    m.insert("action.zoomOut", "התרחקות");
    m.insert("action.resetZoom", "איפוס מרחק התצוגה");
    m.insert("action.toggleTheme", "החלפת מצב מואר/חשוך");
    m.insert("action.toggleGrid", "הצגת/הסתרת רשת");
    m.insert("panel.layers", "שכבות");
    m.insert("panel.properties", "מאפיינים");
    m.insert("panel.library", "ספריה");
    m.insert("menu.language", "שפה");
    m.insert("menu.open", "פתיחה");
    m.insert("menu.save", "שמירה אל…");
    m.insert("menu.exportImage", "ייצוא התמונה…");
    m.insert("menu.exportSvg", "ייצוא ל־SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "הצגת/הסתרת רשת");
    m.insert("menu.layers", "שכבות");
    m.insert("menu.reset", "איפוס לוח הציור");
    m.insert("menu.help", "עזרה");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "תפריט");
    m.insert("menu.new", "New");
    m.insert("menu.load", "פתיחה");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "קיבוץ הבחירה");
    m.insert("action.ungroup", "פירוק קבוצה");
    m.insert("ctx.duplicate", "שכפול");
    m.insert("ctx.delete", "מחיקה");
    m.insert("ctx.copy", "העתקה");
    m.insert("ctx.cut", "גזירה");
    m.insert("ctx.paste", "הדבקה");
    m.insert("ctx.selectAll", "בחירה בהכול");
    m.insert("ctx.bringFront", "קירוב לחזית");
    m.insert("ctx.sendBack", "הרחקה אחורה");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "ללא");
    m.insert("ah.arrow", "חץ");
    m.insert("ah.bar", "קצה אנכי");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "משולש");
    m.insert("ah.diamond", "יהלום");
    m.insert("theme.light", "מצב בהיר");
    m.insert("theme.dark", "מצב כהה");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "כדי להזיז את הקנבס, החזק את {{Scroll wheel}} או {{Space}} תוך כדי גרירה, או השתמש בכלי היד");
    m.insert("library.addSelected", "הוספה לספרייה");
    m.insert("library.empty", "עוד לא הוספת דברים…");
    m.insert("color.stroke", "קו מתאר");
    m.insert("color.background", "רקע");
    m.insert("prop.strokeWidth", "עובי קו מתאר");
    m.insert("prop.fillStyle", "מילוי");
    m.insert("prop.opacity", "אטימות");
    m.insert("prop.roughness", "רישול");
    m.insert("prop.fontSize", "גודל גופן");
    m.insert("prop.fontFamily", "גופן");
    m.insert("prop.textAlign", "יישור טקסט");
    m.insert("prop.strokeStyle", "סגנון קו המתאר");
    m.insert("prop.arrowheads", "ראשי חצים");
    m.insert("prop.roundness", "קצוות");
    m.insert("prop.lock", "נעילה");
    m.insert("prop.unlock", "ביטול נעילה");
    m.insert("prop.group", "קבוצה");
    m.insert("prop.ungroup", "פירוק קבוצה");
    m.insert("prop.align", "יישור");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "יישור לשמאל");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "יישור לימין");
    m.insert("align.top", "יישור למעלה");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "יישור למטה");
    m.insert(
        "fill.hachure",
        "קווים מקבילים קצרים להצגת כיוון וחדות שיפוע במפה",
    );
    m.insert("fill.solid", "אחיד");
    m.insert("fill.zigzag", "זיגזג");
    m.insert("fill.crossHatch", "קווים מוצלבים");
    m.insert("stroke.solid", "מלא");
    m.insert("stroke.dashed", "מקווקו");
    m.insert("stroke.dotted", "מנוקד");
    m.insert("round.none", "ללא");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "שמאל");
    m.insert("textalign.center", "מרכז");
    m.insert("textalign.right", "ימין");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "רקע קנבס");
    m.insert("menu.followUs", "לעקוב אחרינו");
    m.insert("menu.discordGroup", "צ׳אט ב־Discord");
    m.insert("menu.collab", "התחלת שיתוף חי…");
    m.insert("menu.commandPalette", "לוח פקודות");
    m.insert("menu.findOnCanvas", "איתור בלוח הציור");
    m.insert("action.zoomToFit", "שינוי מרחק התצוגה כך שיוצגו כל הרכיבים");
    m.insert(
        "palette.placeholder",
        "תפריטי חיפוש, פקודות ואבני חן נעלמות",
    );
    m.insert("palette.noResults", "אין פקודות תואמות…");
    m.insert("find.placeholder", "איתור טקסט בלוח הציור…");
    m.insert("find.noResults", "לא נמצאו תוצאות…");
    m.insert("help.tools", "כלים");
    m.insert("help.actions", "פעולות");
    m.insert("help.view", "תצוגה");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "העדפות");
    m.insert("menu.theme", "ערכת עיצוב");
    m.insert("menu.toolLock", "נעילת כלים");
    m.insert("menu.snapToObjects", "הצמדה לעצמים");
    m.insert("menu.zenMode", "מצב ריכוז");
    m.insert("menu.viewMode", "מצב תצוגה");
    m.insert("menu.canvasStats", "הגדרות הקנבס והצורות");
    m.insert("menu.arrowBinding", "קשירת חצים");
    m.insert("menu.snapToMidpoints", "הצמדה לנקודות ביניים");
    m.insert("prefs.selectOn", "בחר");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "הטמעה באתר");
    m.insert("toolbar.drawToShape", "ציור אל צורה");
    m.insert("toolbar.bucketFill", "מילוי דלי");
    m.insert("toolbar.lasso", "בחירת לאסו");
    m.insert("toolbar.textToDiagram", "טקסט לתרשים");
    m.insert("toolbar.mermaid", "Mermaid ל־Excalidraw");
    m.insert("toolbar.wireframeToCode", "תרשים קווי מתאר לקוד");
    m.insert("toolbar.generate", "Generate");
    m.insert("welcome.center1", "השרטוטים שלך נשמרים באחסון הדפדפן שלך.");
    m.insert(
        "welcome.center2",
        "זיכרון הדפדפן יכול להתנקות באופן לא צפוי.",
    );
    m.insert(
        "welcome.center3",
        "כדאי לשמור את העבודה שלך תדיר בתור קובץ בשביל שהיא לא תיאבד.",
    );
    m.insert("stats.title", "מאפיינים");
    m.insert("stats.shapes", "צורות");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "המרה לקוד");
    m.insert("labels.copySource", "העתקת המקור ללוח הגזירים");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "צבעים");
    m.insert("colorPicker.shades", "הצללות");
    m.insert("colorPicker.noShades", "אין הצללות זמינות לצבע הזה");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "הצבעים הייחודיים שהכי נפוצים",
    );
    m.insert("colorPicker.hexCode", "קוד הקסדצימלי");
    m.insert("fontList.sceneFonts", "בסצינה הנוכחית");
    m.insert("fontList.availableFonts", "הפונטים הקיימים");
    m.insert("fontList.empty", "לא נמצאו פונטים");
    m.insert("quickSearch.placeholder", "חיפוש מהיר");
    m.insert("elementLink.title", "קישור לפריט");
    m.insert(
        "elementLink.desc",
        "יש ללחוץ על צורה בקנבס או להדביק קישור.",
    );
    m.insert("buttons.remove", "הסרה");
    m.insert("buttons.cancel", "ביטול");
    m.insert("buttons.confirm", "אישור");
    m
}

fn id_id() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Pilih");
    m.insert("toolbar.hand", "Tangan (alat panning)");
    m.insert("toolbar.rectangle", "Persegi");
    m.insert("toolbar.diamond", "Wajik");
    m.insert("toolbar.ellipse", "Elips");
    m.insert("toolbar.arrow", "Panah");
    m.insert("toolbar.line", "Garis");
    m.insert("toolbar.freedraw", "Gambar");
    m.insert("toolbar.text", "Teks");
    m.insert("toolbar.image", "Sisipkan gambar");
    m.insert("toolbar.frame", "Alat bingkai");
    m.insert("toolbar.eraser", "Penghapus");
    m.insert("toolbar.laser", "Penunjuk laser");
    m.insert("toolbar.lock", "Kuncikan");
    m.insert("toolbar.library", "Pustaka");
    m.insert("action.undo", "Urungkan");
    m.insert("action.redo", "Ulangi");
    m.insert("action.delete", "Hapus");
    m.insert("action.duplicate", "Duplikat");
    m.insert("action.exportPng", "Ekspor ke PNG");
    m.insert("action.exportSvg", "Ekspor ke SVG");
    m.insert("action.copyToClipboard", "Salin ke Papan Klip");
    m.insert("action.zoomIn", "Besarkan");
    m.insert("action.zoomOut", "Kecilkan");
    m.insert("action.resetZoom", "Reset Pembesaran");
    m.insert("action.toggleTheme", "Toggle tema terang/gelap");
    m.insert("action.toggleGrid", "Toggle grid");
    m.insert("panel.layers", "Lapisan");
    m.insert("panel.properties", "Properti");
    m.insert("panel.library", "Pustaka");
    m.insert("menu.language", "Bahasa");
    m.insert("menu.open", "Buka");
    m.insert("menu.save", "Simpan ke...");
    m.insert("menu.exportImage", "Ekspor gambar...");
    m.insert("menu.exportSvg", "Ekspor ke SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Toggle grid");
    m.insert("menu.layers", "Lapisan");
    m.insert("menu.reset", "Setel Ulang Kanvas");
    m.insert("menu.help", "Bantuan");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Buka");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Kelompokan pilihan");
    m.insert("action.ungroup", "Pisahkan pilihan");
    m.insert("ctx.duplicate", "Duplikat");
    m.insert("ctx.delete", "Hapus");
    m.insert("ctx.copy", "Salin");
    m.insert("ctx.cut", "Potong");
    m.insert("ctx.paste", "Tempel");
    m.insert("ctx.selectAll", "Pilih semua");
    m.insert("ctx.bringFront", "Bawa ke depan");
    m.insert("ctx.sendBack", "Kirim ke belakang");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Tidak ada");
    m.insert("ah.arrow", "Panah");
    m.insert("ah.bar", "Batang");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Segitiga");
    m.insert("ah.diamond", "Diamond");
    m.insert("theme.light", "Mode terang");
    m.insert("theme.dark", "Mode gelap");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Untuk menggeser kanvas, tahan {{Scroll wheel}} atau {{Space}} sambil menyeret, atau gunakan alat tangan");
    m.insert("library.addSelected", "Tambahkan ke pustaka");
    m.insert("library.empty", "Belum ada item yang ditambahkan...");
    m.insert("color.stroke", "Guratan");
    m.insert("color.background", "Latar");
    m.insert("prop.strokeWidth", "Lebar guratan");
    m.insert("prop.fillStyle", "Isian");
    m.insert("prop.opacity", "Keburaman");
    m.insert("prop.roughness", "Kecerobohan");
    m.insert("prop.fontSize", "Ukuran font");
    m.insert("prop.fontFamily", "Jenis font");
    m.insert("prop.textAlign", "Perataan teks");
    m.insert("prop.strokeStyle", "Gaya guratan");
    m.insert("prop.arrowheads", "Mata panah");
    m.insert("prop.roundness", "Tepi");
    m.insert("prop.lock", "Kuncikan");
    m.insert("prop.unlock", "Lepas");
    m.insert("prop.group", "Kelompok");
    m.insert("prop.ungroup", "Pisahkan pilihan");
    m.insert("prop.align", "Perataan");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Rata kiri");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Rata kanan");
    m.insert("align.top", "Rata atas");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Rata bawah");
    m.insert("fill.hachure", "Garis-garis");
    m.insert("fill.solid", "Padat");
    m.insert("fill.zigzag", "Bentuk Z");
    m.insert("fill.crossHatch", "Asiran silang");
    m.insert("stroke.solid", "Padat");
    m.insert("stroke.dashed", "Putus-putus");
    m.insert("stroke.dotted", "Titik-titik");
    m.insert("round.none", "Tidak ada");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Kiri");
    m.insert("textalign.center", "Tengah");
    m.insert("textalign.right", "Kanan");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Latar Kanvas");
    m.insert("menu.followUs", "Ikuti kami");
    m.insert("menu.discordGroup", "Obrolan Discord");
    m.insert("menu.collab", "Kolaborasi langsung...");
    m.insert("menu.commandPalette", "Daftar Perintah");
    m.insert("menu.findOnCanvas", "Temukan di kanvas");
    m.insert("action.zoomToFit", "Memperbesar agar unsur pas semua");
    m.insert(
        "palette.placeholder",
        "Cari menu, perintah dan temukan hidden gems",
    );
    m.insert("palette.noResults", "Tidak ada perintah yang sesuai...");
    m.insert("find.placeholder", "Temukan teks di kanvas...");
    m.insert("find.noResults", "Tidak ada kecocokan yang ditemukan...");
    m.insert("help.tools", "Alat");
    m.insert("help.actions", "Aksi");
    m.insert("help.view", "Tampilan");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Daftar");
    m.insert("menu.preferences", "Preferensi");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Kunci alat");
    m.insert("menu.snapToObjects", "Snap to objects");
    m.insert("menu.zenMode", "Mode zen");
    m.insert("menu.viewMode", "Mode tampilan");
    m.insert("menu.canvasStats", "Properti Kanvas & Bentuk");
    m.insert("menu.arrowBinding", "Pengikatan anak panah");
    m.insert("menu.snapToMidpoints", "Rapatkan ke titik tengah");
    m.insert("prefs.selectOn", "Pilih pada");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Sematan Web");
    m.insert("toolbar.drawToShape", "Gambar ke bentuk");
    m.insert("toolbar.bucketFill", "Isi warna (Ember)");
    m.insert("toolbar.lasso", "Seleksi lasso");
    m.insert("toolbar.textToDiagram", "Teks menjadi diagram");
    m.insert("toolbar.mermaid", "Mermaid menjadi Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe menjadi code");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Gambar anda disimpan di penyimpanan browser anda.",
    );
    m.insert(
        "welcome.center2",
        "Penyimpanan browser dapat terhapus secara tidak terduga.",
    );
    m.insert(
        "welcome.center3",
        "Simpan pekerjaan anda ke file secara berkala agar tidak hilang.",
    );
    m.insert("stats.title", "Properti");
    m.insert("stats.shapes", "Bentuk");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Konversi ke kode");
    m.insert("labels.copySource", "Salin sumber ke papan klip");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Warna");
    m.insert("colorPicker.shades", "Nuansa");
    m.insert("colorPicker.noShades", "Tidak ada nuansa untuk warna ini");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Warna yang sering dipakai",
    );
    m.insert("colorPicker.hexCode", "Kode hexa");
    m.insert("fontList.sceneFonts", "Pada layar ini");
    m.insert("fontList.availableFonts", "Font yang tersedia");
    m.insert("fontList.empty", "Tidak ada font yang ditemukan");
    m.insert("quickSearch.placeholder", "Pencarian Cepat");
    m.insert("elementLink.title", "Taut ke objek");
    m.insert(
        "elementLink.desc",
        "Pencet bentuk pada kanvas atau tempel tautan.",
    );
    m.insert("buttons.remove", "Hapus");
    m.insert("buttons.cancel", "Batal");
    m.insert("buttons.confirm", "Konfirmasi");
    m
}

fn nl_nl() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Selecteer");
    m.insert("toolbar.hand", "Hand (panning tool)");
    m.insert("toolbar.rectangle", "Rechthoek");
    m.insert("toolbar.diamond", "Diamant");
    m.insert("toolbar.ellipse", "Ovaal");
    m.insert("toolbar.arrow", "Pijl");
    m.insert("toolbar.line", "Lijn");
    m.insert("toolbar.freedraw", "Teken");
    m.insert("toolbar.text", "Tekst");
    m.insert("toolbar.image", "Voeg afbeelding in");
    m.insert("toolbar.frame", "Frame tool");
    m.insert("toolbar.eraser", "Gum");
    m.insert("toolbar.laser", "Laseraanwijzer");
    m.insert("toolbar.lock", "Vergrendel");
    m.insert("toolbar.library", "Bibliotheek");
    m.insert("action.undo", "Ongedaan maken");
    m.insert("action.redo", "Herstel ongedaan maken");
    m.insert("action.delete", "Verwijderen");
    m.insert("action.duplicate", "Dupliceer");
    m.insert("action.exportPng", "Exporteer naar PNG");
    m.insert("action.exportSvg", "Exporteer naar SVG");
    m.insert("action.copyToClipboard", "Kopieer");
    m.insert("action.zoomIn", "Inzoomen");
    m.insert("action.zoomOut", "Uitzoomen");
    m.insert("action.resetZoom", "Zoom terugzetten");
    m.insert("action.toggleTheme", "Licht-/donkerthema in-/uitschakelen");
    m.insert("action.toggleGrid", "Raster in-/uitschakelen");
    m.insert("panel.layers", "Lagen");
    m.insert("panel.properties", "Eigenschappen");
    m.insert("panel.library", "Bibliotheek");
    m.insert("menu.language", "Taal");
    m.insert("menu.open", "Open");
    m.insert("menu.save", "Sla op...");
    m.insert("menu.exportImage", "Exporteer afbeelding...");
    m.insert("menu.exportSvg", "Exporteer naar SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Raster in-/uitschakelen");
    m.insert("menu.layers", "Lagen");
    m.insert("menu.reset", "Canvas opnieuw instellen");
    m.insert("menu.help", "Help");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Open");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Groeperen");
    m.insert("action.ungroup", "Groep opheffen");
    m.insert("ctx.duplicate", "Dupliceer");
    m.insert("ctx.delete", "Verwijderen");
    m.insert("ctx.copy", "Kopiëren");
    m.insert("ctx.cut", "Knip");
    m.insert("ctx.paste", "Plakken");
    m.insert("ctx.selectAll", "Alles selecteren");
    m.insert("ctx.bringFront", "Breng naar voorgrond");
    m.insert("ctx.sendBack", "Stuur naar achtergrond");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Geen");
    m.insert("ah.arrow", "Pijl");
    m.insert("ah.bar", "Balk");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Driehoek");
    m.insert("ah.diamond", "Diamant");
    m.insert("theme.light", "Lichte modus");
    m.insert("theme.dark", "Donkere modus");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Om canvas te verplaatsen, houd {{Scroll wheel}} of {{Space}} ingedrukt tijdens slepen, of gebruik de hand tool");
    m.insert("library.addSelected", "Voeg toe aan bibliotheek");
    m.insert("library.empty", "Nog geen items toegevoegd...");
    m.insert("color.stroke", "Lijn");
    m.insert("color.background", "Achtergrond");
    m.insert("prop.strokeWidth", "Lijnbreedte");
    m.insert("prop.fillStyle", "Invulling");
    m.insert("prop.opacity", "Doorzichtigheid");
    m.insert("prop.roughness", "Slordigheid");
    m.insert("prop.fontSize", "Tekstgrootte");
    m.insert("prop.fontFamily", "Lettertype");
    m.insert("prop.textAlign", "Uitlijning");
    m.insert("prop.strokeStyle", "Lijnstijl");
    m.insert("prop.arrowheads", "Pijlpunten");
    m.insert("prop.roundness", "Randen");
    m.insert("prop.lock", "Vergrendel");
    m.insert("prop.unlock", "Ontgrendel");
    m.insert("prop.group", "Groep");
    m.insert("prop.ungroup", "Groep opheffen");
    m.insert("prop.align", "Uitlijnen");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Links uitlijnen");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Rechts uitlijnen");
    m.insert("align.top", "Boven uitlijnen");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Onder uitlijnen");
    m.insert("fill.hachure", "Arcering");
    m.insert("fill.solid", "Ingekleurd");
    m.insert("fill.zigzag", "Zigzag");
    m.insert("fill.crossHatch", "Tweemaal gearceerd");
    m.insert("stroke.solid", "Ononderbroken");
    m.insert("stroke.dashed", "Gestreept");
    m.insert("stroke.dotted", "Gestippeld");
    m.insert("round.none", "Geen");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Links");
    m.insert("textalign.center", "Midden");
    m.insert("textalign.right", "Rechts");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Canvas achtergrond");
    m.insert("menu.followUs", "Volg ons");
    m.insert("menu.discordGroup", "Discord chat");
    m.insert("menu.collab", "Live Samenwerking...");
    m.insert("menu.commandPalette", "Opdrachtenpalet");
    m.insert("menu.findOnCanvas", "Zoek op canvas");
    m.insert("action.zoomToFit", "Zoom om alle element te laten passen");
    m.insert(
        "palette.placeholder",
        "Zoek in menu's, commando's en ontdek geheimpjes",
    );
    m.insert("palette.noResults", "Geen overeenkomende opdrachten...");
    m.insert("find.placeholder", "Zoek text op canvas...");
    m.insert("find.noResults", "Geen overeenkomsten gevonden...");
    m.insert("help.tools", "Tools");
    m.insert("help.actions", "Acties");
    m.insert("help.view", "Weergave");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Voorkeuren");
    m.insert("menu.theme", "Thema");
    m.insert("menu.toolLock", "Tool vergrendelen");
    m.insert("menu.snapToObjects", "Uitlijnen op objecten");
    m.insert("menu.zenMode", "Zen modus");
    m.insert("menu.viewMode", "Weergavemodus");
    m.insert("menu.canvasStats", "Canvas & Vorm eigenschappen");
    m.insert("menu.arrowBinding", "Pijlverbinding");
    m.insert("menu.snapToMidpoints", "Vastklikken op middelpunt");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web Embed");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Lasso selectie");
    m.insert("toolbar.textToDiagram", "Tekst naar diagram");
    m.insert("toolbar.mermaid", "Mermaid naar Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe naar code");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Je tekeningen worden bewaard in de opslag van je browser.",
    );
    m.insert(
        "welcome.center2",
        "Browser opslag kan onverwacht gewist worden.",
    );
    m.insert(
        "welcome.center3",
        "Sla je werk regelmatig op in een bestand om te voorkomen dat je het kwijtraakt.",
    );
    m.insert("stats.title", "Eigenschappen");
    m.insert("stats.shapes", "Vormen");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Zet om naar code");
    m.insert("labels.copySource", "Bron kopiëren naar klembord");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Kleuren");
    m.insert("colorPicker.shades", "Tinten");
    m.insert(
        "colorPicker.noShades",
        "Geen tinten beschikbaar voor deze kleur",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Meest gebruikte aangepaste kleur",
    );
    m.insert("colorPicker.hexCode", "HEX-code");
    m.insert("fontList.sceneFonts", "In deze scène");
    m.insert("fontList.availableFonts", "Beschikbare lettertypen");
    m.insert("fontList.empty", "Geen lettertypes gevonden");
    m.insert("quickSearch.placeholder", "Snel Zoeken");
    m.insert("elementLink.title", "Link naar object");
    m.insert(
        "elementLink.desc",
        "Klik op een vorm op canvas of plak een link.",
    );
    m.insert("buttons.remove", "Verwijderen");
    m.insert("buttons.cancel", "Annuleren");
    m.insert("buttons.confirm", "Bevestigen");
    m
}

fn pl_pl() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Wybierz");
    m.insert("toolbar.hand", "Ręka (narzędzie do przesuwania)");
    m.insert("toolbar.rectangle", "Prostokąt");
    m.insert("toolbar.diamond", "Romb");
    m.insert("toolbar.ellipse", "Elipsa");
    m.insert("toolbar.arrow", "Strzałka");
    m.insert("toolbar.line", "Linia");
    m.insert("toolbar.freedraw", "Rysuj");
    m.insert("toolbar.text", "Tekst");
    m.insert("toolbar.image", "Wstaw obraz");
    m.insert("toolbar.frame", "Ramka");
    m.insert("toolbar.eraser", "Gumka");
    m.insert("toolbar.laser", "Wskaźnik laserowy");
    m.insert("toolbar.lock", "Zablokuj");
    m.insert("toolbar.library", "Biblioteka");
    m.insert("action.undo", "Cofnij");
    m.insert("action.redo", "Przywróć");
    m.insert("action.delete", "Usuń");
    m.insert("action.duplicate", "Powiel");
    m.insert("action.exportPng", "Zapisz jako PNG");
    m.insert("action.exportSvg", "Zapisz jako SVG");
    m.insert("action.copyToClipboard", "Skopiuj do schowka");
    m.insert("action.zoomIn", "Powiększ");
    m.insert("action.zoomOut", "Pomniejsz");
    m.insert("action.resetZoom", "Zresetuj powiększenie");
    m.insert("action.toggleTheme", "Przełącz motyw jasny / ciemny");
    m.insert("action.toggleGrid", "Przełącz widoczność siatki");
    m.insert("panel.layers", "Warstwy");
    m.insert("panel.properties", "Właściwości");
    m.insert("panel.library", "Biblioteka");
    m.insert("menu.language", "Język");
    m.insert("menu.open", "Otwórz");
    m.insert("menu.save", "Zapisz jako...");
    m.insert("menu.exportImage", "Eksportuj obraz...");
    m.insert("menu.exportSvg", "Zapisz jako SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Przełącz widoczność siatki");
    m.insert("menu.layers", "Warstwy");
    m.insert("menu.reset", "Wyczyść dokument i zresetuj kolor dokumentu");
    m.insert("menu.help", "Pomoc");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Otwórz");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Zgrupuj wybrane");
    m.insert("action.ungroup", "Rozgrupuj wybrane");
    m.insert("ctx.duplicate", "Powiel");
    m.insert("ctx.delete", "Usuń");
    m.insert("ctx.copy", "Kopiuj");
    m.insert("ctx.cut", "Wytnij");
    m.insert("ctx.paste", "Wklej");
    m.insert("ctx.selectAll", "Zaznacz wszystko");
    m.insert("ctx.bringFront", "Przenieś na wierzch");
    m.insert("ctx.sendBack", "Przenieś na spód");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Brak");
    m.insert("ah.arrow", "Strzałka");
    m.insert("ah.bar", "Kreska");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Trójkąt");
    m.insert("ah.diamond", "Romb");
    m.insert("theme.light", "Jasny motyw");
    m.insert("theme.dark", "Ciemny motyw");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Aby przesunąć płótno, przytrzymaj {{Scroll wheel}} lub {{Space}} podczas przeciągania, albo użyj narzędzia ręki");
    m.insert("library.addSelected", "Dodaj do biblioteki");
    m.insert("library.empty", "Nie dodano jeszcze żadnych elementów...");
    m.insert("color.stroke", "Kreska");
    m.insert("color.background", "Tło");
    m.insert("prop.strokeWidth", "Szerokość kreski");
    m.insert("prop.fillStyle", "Wypełnienie");
    m.insert("prop.opacity", "Przezroczystość");
    m.insert("prop.roughness", "Styl kreski");
    m.insert("prop.fontSize", "Rozmiar tekstu");
    m.insert("prop.fontFamily", "Krój pisma");
    m.insert("prop.textAlign", "Wyrównanie tekstu");
    m.insert("prop.strokeStyle", "Styl kreski");
    m.insert("prop.arrowheads", "Groty");
    m.insert("prop.roundness", "Krawędzie");
    m.insert("prop.lock", "Zablokuj");
    m.insert("prop.unlock", "Odblokuj");
    m.insert("prop.group", "Grupa");
    m.insert("prop.ungroup", "Rozgrupuj wybrane");
    m.insert("prop.align", "Wyrównaj");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Wyrównaj do lewej");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Wyrównaj do prawej");
    m.insert("align.top", "Wyrównaj do góry");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Wyrównaj do dołu");
    m.insert("fill.hachure", "Linie");
    m.insert("fill.solid", "Pełne");
    m.insert("fill.zigzag", "Zygzak");
    m.insert("fill.crossHatch", "Zakreślone");
    m.insert("stroke.solid", "Pełny");
    m.insert("stroke.dashed", "Linia przerywana");
    m.insert("stroke.dotted", "Kropkowany");
    m.insert("round.none", "Brak");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Do lewej");
    m.insert("textalign.center", "Do środka");
    m.insert("textalign.right", "Do prawej");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Kolor dokumentu");
    m.insert("menu.followUs", "Obserwuj nas");
    m.insert("menu.discordGroup", "Czat Discorda");
    m.insert("menu.collab", "Współpraca w czasie rzeczywistym...");
    m.insert("menu.commandPalette", "Paleta poleceń");
    m.insert("menu.findOnCanvas", "Znajdź na płótnie");
    m.insert("action.zoomToFit", "Dopasuj widok do całej zawartości");
    m.insert(
        "palette.placeholder",
        "Szukaj w menu, poleceniach i odkrywaj ukryte funkcje",
    );
    m.insert("palette.noResults", "Brak pasujących komend...");
    m.insert("find.placeholder", "Znajdź tekst na płótnie...");
    m.insert("find.noResults", "Nie znaleziono pasujących elementów...");
    m.insert("help.tools", "Narzędzia");
    m.insert("help.actions", "Akcje");
    m.insert("help.view", "Widok");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferences");
    m.insert("menu.theme", "Motyw");
    m.insert("menu.toolLock", "Tool lock");
    m.insert("menu.snapToObjects", "Przyciąganie do obiektów");
    m.insert("menu.zenMode", "Tryb Zen");
    m.insert("menu.viewMode", "Tryb widoku");
    m.insert("menu.canvasStats", "Właściwości płótna i kształtu");
    m.insert("menu.arrowBinding", "Arrow binding");
    m.insert("menu.snapToMidpoints", "Snap to midpoints");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Osadzenie z internetu");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Wybór lassem");
    m.insert("toolbar.textToDiagram", "Generuj diagram z opisu");
    m.insert("toolbar.mermaid", "Konwertuj diagram Mermaid do Excalidraw");
    m.insert("toolbar.wireframeToCode", "Wireframe do kodu");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Your drawings are saved in your browser storage.",
    );
    m.insert(
        "welcome.center2",
        "Your browser storage might be accidentally cleared.",
    );
    m.insert(
        "welcome.center3",
        "Save your work to a file from time to time to avoid losing it.",
    );
    m.insert("stats.title", "Właściwości");
    m.insert("stats.shapes", "Kształty");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Skonwertuj na kod");
    m.insert("labels.copySource", "Kopiuj źródło do schowka");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Kolory");
    m.insert("colorPicker.shades", "Odcienie");
    m.insert(
        "colorPicker.noShades",
        "Brak dostępnych odcieni dla tego koloru",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Najczęściej używane kolory",
    );
    m.insert("colorPicker.hexCode", "Kod HEX");
    m.insert("fontList.sceneFonts", "W tej scenie");
    m.insert("fontList.availableFonts", "Dostępne czcionki");
    m.insert("fontList.empty", "Nie znaleziono czcionek");
    m.insert("quickSearch.placeholder", "Szybkie wyszukiwanie");
    m.insert("elementLink.title", "Link do obiektu");
    m.insert(
        "elementLink.desc",
        "Kliknij kształt na obszarze roboczym lub wklej link.",
    );
    m.insert("buttons.remove", "Usuń");
    m.insert("buttons.cancel", "Anuluj");
    m.insert("buttons.confirm", "Zatwierdź");
    m
}

fn pt_pt() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Selecionar");
    m.insert("toolbar.hand", "Mão (ferramenta de deslocar vista)");
    m.insert("toolbar.rectangle", "Retângulo");
    m.insert("toolbar.diamond", "Diamante");
    m.insert("toolbar.ellipse", "Elipse");
    m.insert("toolbar.arrow", "Seta");
    m.insert("toolbar.line", "Linha");
    m.insert("toolbar.freedraw", "Desenhar");
    m.insert("toolbar.text", "Texto");
    m.insert("toolbar.image", "Inserir imagem");
    m.insert("toolbar.frame", "Ferramenta de moldura");
    m.insert("toolbar.eraser", "Borracha");
    m.insert("toolbar.laser", "Ponteiro laser");
    m.insert("toolbar.lock", "Bloquear");
    m.insert("toolbar.library", "Biblioteca");
    m.insert("action.undo", "Desfazer");
    m.insert("action.redo", "Refazer");
    m.insert("action.delete", "Apagar");
    m.insert("action.duplicate", "Duplicar");
    m.insert("action.exportPng", "Exportar em PNG");
    m.insert("action.exportSvg", "Exportar em SVG");
    m.insert(
        "action.copyToClipboard",
        "Copiar para a área de transferência",
    );
    m.insert("action.zoomIn", "Aproximar");
    m.insert("action.zoomOut", "Afastar");
    m.insert("action.resetZoom", "Repor ampliação");
    m.insert("action.toggleTheme", "Alternar tema claro/escuro");
    m.insert("action.toggleGrid", "Ativar/desativar grelha");
    m.insert("panel.layers", "Camadas");
    m.insert("panel.properties", "Propriedades");
    m.insert("panel.library", "Biblioteca");
    m.insert("menu.language", "Idioma");
    m.insert("menu.open", "Abrir");
    m.insert("menu.save", "Guardar para...");
    m.insert("menu.exportImage", "Exportar imagem...");
    m.insert("menu.exportSvg", "Exportar em SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Ativar/desativar grelha");
    m.insert("menu.layers", "Camadas");
    m.insert("menu.reset", "Limpar a área de desenho");
    m.insert("menu.help", "Ajuda");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Abrir");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Agrupar seleção");
    m.insert("action.ungroup", "Desagrupar seleção");
    m.insert("ctx.duplicate", "Duplicar");
    m.insert("ctx.delete", "Apagar");
    m.insert("ctx.copy", "Copiar");
    m.insert("ctx.cut", "Cortar");
    m.insert("ctx.paste", "Colar");
    m.insert("ctx.selectAll", "Selecionar tudo");
    m.insert("ctx.bringFront", "Subir");
    m.insert("ctx.sendBack", "Baixar");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Nenhuma");
    m.insert("ah.arrow", "Seta");
    m.insert("ah.bar", "Barra");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Triângulo");
    m.insert("ah.diamond", "Losango");
    m.insert("theme.light", "Modo claro");
    m.insert("theme.dark", "Modo escuro");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Para mover a área de desenho, pressione {{Scroll wheel}} ou {{Space}} enquanto arrasta, ou use a ferramenta da mãozinha");
    m.insert("library.addSelected", "Adicionar à biblioteca");
    m.insert("library.empty", "Ainda não foram adicionados itens...");
    m.insert("color.stroke", "Contorno");
    m.insert("color.background", "Fundo");
    m.insert("prop.strokeWidth", "Espessura do traço");
    m.insert("prop.fillStyle", "Preenchimento");
    m.insert("prop.opacity", "Opacidade");
    m.insert("prop.roughness", "Precisão do traço");
    m.insert("prop.fontSize", "Tamanho do tipo de letra");
    m.insert("prop.fontFamily", "Família do tipo de letra");
    m.insert("prop.textAlign", "Alinhamento do texto");
    m.insert("prop.strokeStyle", "Estilo do traço");
    m.insert("prop.arrowheads", "Pontas");
    m.insert("prop.roundness", "Arestas");
    m.insert("prop.lock", "Bloquear");
    m.insert("prop.unlock", "Desbloquear");
    m.insert("prop.group", "Grupo");
    m.insert("prop.ungroup", "Desagrupar seleção");
    m.insert("prop.align", "Alinhamento");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Alinhar à esquerda");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Alinhar à direita");
    m.insert("align.top", "Alinhar ao topo");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Alinhar ao fundo");
    m.insert("fill.hachure", "Sombreado");
    m.insert("fill.solid", "Sólido");
    m.insert("fill.zigzag", "Ziguezague");
    m.insert("fill.crossHatch", "Xadrez");
    m.insert("stroke.solid", "Sólido");
    m.insert("stroke.dashed", "Tracejado");
    m.insert("stroke.dotted", "Pontilhado");
    m.insert("round.none", "Nenhuma");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Esquerda");
    m.insert("textalign.center", "Centro");
    m.insert("textalign.right", "Direita");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Fundo da área de desenho");
    m.insert("menu.followUs", "Siga-nos");
    m.insert("menu.discordGroup", "Conversação no Discord");
    m.insert("menu.collab", "Colaboração ao vivo...");
    m.insert("menu.commandPalette", "Paleta de comandos");
    m.insert("menu.findOnCanvas", "Pesquisar na área de desenho");
    m.insert(
        "action.zoomToFit",
        "Aproximar para enquadrar todos os elementos",
    );
    m.insert(
        "palette.placeholder",
        "Procure menus, comandos e descubra pedras preciosas ocultas",
    );
    m.insert("palette.noResults", "Nenhum comando correspondente...");
    m.insert("find.placeholder", "Pesquisar texto na área de desenho...");
    m.insert("find.noResults", "Não foram encontrados resultados...");
    m.insert("help.tools", "Ferramentas");
    m.insert("help.actions", "Ações");
    m.insert("help.view", "Visualização");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferências");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Bloqueio de ferramenta");
    m.insert("menu.snapToObjects", "Atrair aos objetos");
    m.insert("menu.zenMode", "Modo zen");
    m.insert("menu.viewMode", "Modo de visualização");
    m.insert(
        "menu.canvasStats",
        "Área de desenho e propriedades da forma",
    );
    m.insert("menu.arrowBinding", "Ligar automaticamente as setas");
    m.insert(
        "menu.snapToMidpoints",
        "Ligar automaticamente aos pontos médios",
    );
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Incorporar web");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Seleção de laçarote");
    m.insert("toolbar.textToDiagram", "Texto para diagrama");
    m.insert("toolbar.mermaid", "Mermaid para Excalidraw");
    m.insert("toolbar.wireframeToCode", "Esquema para código");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Os seus desenhos estão guardados no armazenamento do seu navegador.",
    );
    m.insert(
        "welcome.center2",
        "O armazenamento do navegador pode ser apagado inesperadamente.",
    );
    m.insert(
        "welcome.center3",
        "Para não perder os seus desenhos, guarde-os num ficheiro.",
    );
    m.insert("stats.title", "Propriedades");
    m.insert("stats.shapes", "Formas");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Converter para código");
    m.insert(
        "labels.copySource",
        "Copiar origem para área de transferência",
    );
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Cores");
    m.insert("colorPicker.shades", "Tons");
    m.insert("colorPicker.noShades", "Sem tons disponíveis para esta cor");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Cores personalizadas mais usadas",
    );
    m.insert("colorPicker.hexCode", "Código hex");
    m.insert("fontList.sceneFonts", "Nesta cena");
    m.insert("fontList.availableFonts", "Fontes disponíveis");
    m.insert("fontList.empty", "Nenhuma fonte encontrada");
    m.insert("quickSearch.placeholder", "Pesquisa rápida");
    m.insert("elementLink.title", "Hiperligação para o objeto");
    m.insert(
        "elementLink.desc",
        "Clique numa forma na tela ou cole uma hiperligação.",
    );
    m.insert("buttons.remove", "Remover");
    m.insert("buttons.cancel", "Cancelar");
    m.insert("buttons.confirm", "Confirmar");
    m
}

fn ro_ro() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Selectare");
    m.insert("toolbar.hand", "Mână (instrument de panoramare)");
    m.insert("toolbar.rectangle", "Dreptunghi");
    m.insert("toolbar.diamond", "Romb");
    m.insert("toolbar.ellipse", "Elipsă");
    m.insert("toolbar.arrow", "Săgeată");
    m.insert("toolbar.line", "Linie");
    m.insert("toolbar.freedraw", "Desenare");
    m.insert("toolbar.text", "Text");
    m.insert("toolbar.image", "Introducere imagine");
    m.insert("toolbar.frame", "Instrument cadru");
    m.insert("toolbar.eraser", "Radieră");
    m.insert("toolbar.laser", "Indicator laser");
    m.insert("toolbar.lock", "Blocare");
    m.insert("toolbar.library", "Bibliotecă");
    m.insert("action.undo", "Anulare");
    m.insert("action.redo", "Refacere");
    m.insert("action.delete", "Ștergere");
    m.insert("action.duplicate", "Duplicare");
    m.insert("action.exportPng", "Exportare ca PNG");
    m.insert("action.exportSvg", "Exportare ca SVG");
    m.insert("action.copyToClipboard", "Copiere în memoria temporară");
    m.insert("action.zoomIn", "Apropiere");
    m.insert("action.zoomOut", "Depărtare");
    m.insert("action.resetZoom", "Resetare transfocare");
    m.insert("action.toggleTheme", "Comutare temă luminoasă/întunecată");
    m.insert("action.toggleGrid", "Comutare grilă");
    m.insert("panel.layers", "Straturi");
    m.insert("panel.properties", "Proprietăți");
    m.insert("panel.library", "Bibliotecă");
    m.insert("menu.language", "Limbă");
    m.insert("menu.open", "Deschidere");
    m.insert("menu.save", "Salvare în...");
    m.insert("menu.exportImage", "Exportare imagine...");
    m.insert("menu.exportSvg", "Exportare ca SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Comutare grilă");
    m.insert("menu.layers", "Straturi");
    m.insert("menu.reset", "Resetare pânză");
    m.insert("menu.help", "Ajutor");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Meniu");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Deschidere");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Grupare selecție");
    m.insert("action.ungroup", "Degrupare selecție");
    m.insert("ctx.duplicate", "Duplicare");
    m.insert("ctx.delete", "Ștergere");
    m.insert("ctx.copy", "Copiere");
    m.insert("ctx.cut", "Decupare");
    m.insert("ctx.paste", "Lipire");
    m.insert("ctx.selectAll", "Selectare totală");
    m.insert("ctx.bringFront", "Aducere în prim plan");
    m.insert("ctx.sendBack", "Trimitere în ultimul plan");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Niciunul");
    m.insert("ah.arrow", "Săgeată");
    m.insert("ah.bar", "Bară");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Triunghi");
    m.insert("ah.diamond", "Romb");
    m.insert("theme.light", "Mod luminos");
    m.insert("theme.dark", "Mod întunecat");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Pentru a muta pânză, ține {{Scroll wheel}} sau {{Space}} în timp ce glisezi sau folosește instrumentul în formă de mână");
    m.insert("library.addSelected", "Adăugare la bibliotecă");
    m.insert("library.empty", "Niciun element adăugat încă...");
    m.insert("color.stroke", "Contur");
    m.insert("color.background", "Fundal");
    m.insert("prop.strokeWidth", "Lățimea conturului");
    m.insert("prop.fillStyle", "Umplere");
    m.insert("prop.opacity", "Opacitate");
    m.insert("prop.roughness", "Aspectul trasării");
    m.insert("prop.fontSize", "Dimensiune font");
    m.insert("prop.fontFamily", "Familia de fonturi");
    m.insert("prop.textAlign", "Alinierea textului");
    m.insert("prop.strokeStyle", "Stilul conturului");
    m.insert("prop.arrowheads", "Vârfuri de săgeată");
    m.insert("prop.roundness", "Margini");
    m.insert("prop.lock", "Blocare");
    m.insert("prop.unlock", "Deblocare");
    m.insert("prop.group", "Grup");
    m.insert("prop.ungroup", "Degrupare selecție");
    m.insert("prop.align", "Aliniere");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Aliniere la stânga");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Aliniere la dreapta");
    m.insert("align.top", "Aliniere sus");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Aliniere jos");
    m.insert("fill.hachure", "Hașură");
    m.insert("fill.solid", "Plină");
    m.insert("fill.zigzag", "Zigzag");
    m.insert("fill.crossHatch", "Hașură transversală");
    m.insert("stroke.solid", "Neîntrerupt");
    m.insert("stroke.dashed", "Liniuțe");
    m.insert("stroke.dotted", "Punctat");
    m.insert("round.none", "Niciunul");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Stânga");
    m.insert("textalign.center", "Centru");
    m.insert("textalign.right", "Dreapta");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Fundalul pânzei");
    m.insert("menu.followUs", "Urmărește-ne");
    m.insert("menu.discordGroup", "Conversații pe Discord");
    m.insert("menu.collab", "Colaborare în direct...");
    m.insert("menu.commandPalette", "Paletă de comenzi");
    m.insert("menu.findOnCanvas", "Găsire pe pânză");
    m.insert(
        "action.zoomToFit",
        "Transfocare pentru a cuprinde toate elementele",
    );
    m.insert(
        "palette.placeholder",
        "Caută meniuri, comenzi și descoperă nestemate ascunse",
    );
    m.insert("palette.noResults", "Nicio comandă potrivită...");
    m.insert("find.placeholder", "Găsire text pe pânză...");
    m.insert("find.noResults", "Nicio potrivire găsită...");
    m.insert("help.tools", "Instrumente");
    m.insert("help.actions", "Acțiuni");
    m.insert("help.view", "Vizualizare");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Înscrie-te");
    m.insert("menu.preferences", "Preferințe");
    m.insert("menu.theme", "Temă");
    m.insert("menu.toolLock", "Blocare instrument");
    m.insert("menu.snapToObjects", "Ancorare la obiecte");
    m.insert("menu.zenMode", "Mod zen");
    m.insert("menu.viewMode", "Mod de vizualizare");
    m.insert("menu.canvasStats", "Proprietăți pentru pânză și forme");
    m.insert("menu.arrowBinding", "Legare săgeată");
    m.insert("menu.snapToMidpoints", "Ancorare la punctele medii");
    m.insert("prefs.selectOn", "Selectare pe");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Încorporare web");
    m.insert("toolbar.drawToShape", "Desenează la formă");
    m.insert("toolbar.bucketFill", "Umplere cu culoare");
    m.insert("toolbar.lasso", "Selecție lasou");
    m.insert("toolbar.textToDiagram", "Text la diagramă");
    m.insert("toolbar.mermaid", "Mermaid la Excalidraw");
    m.insert("toolbar.wireframeToCode", "Structură-de-fire la cod");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Desenele tale sunt salvate în spațiul de stocare al navigatorului tău.",
    );
    m.insert(
        "welcome.center2",
        "Spațiul de stocare al navigatorului poate fi șters în mod neașteptat.",
    );
    m.insert(
        "welcome.center3",
        "Salvează-ți periodic munca într-un fișier pentru a evita pierderea acesteia.",
    );
    m.insert("stats.title", "Proprietăți");
    m.insert("stats.shapes", "Forme");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Convertire în cod");
    m.insert("labels.copySource", "Copiere sursă în memoria temporară");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Culori");
    m.insert("colorPicker.shades", "Nuanțe");
    m.insert(
        "colorPicker.noShades",
        "Nu este disponibilă nicio nuanță pentru această culoare",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Cele mai utilizate culori personalizate",
    );
    m.insert("colorPicker.hexCode", "Cod hexa");
    m.insert("fontList.sceneFonts", "În această scenă");
    m.insert("fontList.availableFonts", "Fonturi disponibile");
    m.insert("fontList.empty", "Niciun font găsit");
    m.insert("quickSearch.placeholder", "Căutare rapidă");
    m.insert("elementLink.title", "Link către obiect");
    m.insert(
        "elementLink.desc",
        "Dă clic pe o formă pe pânză sau lipește un link.",
    );
    m.insert("buttons.remove", "Eliminare");
    m.insert("buttons.cancel", "Anulare");
    m.insert("buttons.confirm", "Confirmare");
    m
}

fn sk_sk() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Vybrať");
    m.insert("toolbar.hand", "Ruka (nástroj pre pohyb plátna)");
    m.insert("toolbar.rectangle", "Obdĺžnik");
    m.insert("toolbar.diamond", "Diamant");
    m.insert("toolbar.ellipse", "Elipsa");
    m.insert("toolbar.arrow", "Šípka");
    m.insert("toolbar.line", "Čiara");
    m.insert("toolbar.freedraw", "Kresliť");
    m.insert("toolbar.text", "Text");
    m.insert("toolbar.image", "Vložiť obrázok");
    m.insert("toolbar.frame", "Nástroj rám");
    m.insert("toolbar.eraser", "Guma");
    m.insert("toolbar.laser", "Laserový ukazovateľ");
    m.insert("toolbar.lock", "Zamknúť");
    m.insert("toolbar.library", "Knižnica");
    m.insert("action.undo", "Späť");
    m.insert("action.redo", "Znova");
    m.insert("action.delete", "Vymazať");
    m.insert("action.duplicate", "Duplikovať");
    m.insert("action.exportPng", "Exportovať do PNG");
    m.insert("action.exportSvg", "Exportovať do SVG");
    m.insert("action.copyToClipboard", "Kopírovať do schránky");
    m.insert("action.zoomIn", "Priblížiť");
    m.insert("action.zoomOut", "Oddialiť");
    m.insert("action.resetZoom", "Obnoviť priblíženie");
    m.insert("action.toggleTheme", "Prepnúť svetlú/tmavú tému");
    m.insert("action.toggleGrid", "Prepnúť mriežku");
    m.insert("panel.layers", "Vrstvy");
    m.insert("panel.properties", "Vlastnosti");
    m.insert("panel.library", "Knižnica");
    m.insert("menu.language", "Jazyk");
    m.insert("menu.open", "Otvoriť");
    m.insert("menu.save", "Uložiť do...");
    m.insert("menu.exportImage", "Exportovať obrázok...");
    m.insert("menu.exportSvg", "Exportovať do SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Prepnúť mriežku");
    m.insert("menu.layers", "Vrstvy");
    m.insert("menu.reset", "Obnoviť plátno");
    m.insert("menu.help", "Pomocník");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Ponuka");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Otvoriť");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Zoskupiť");
    m.insert("action.ungroup", "Zrušiť zoskupenie");
    m.insert("ctx.duplicate", "Duplikovať");
    m.insert("ctx.delete", "Vymazať");
    m.insert("ctx.copy", "Kopírovať");
    m.insert("ctx.cut", "Vystrihnúť");
    m.insert("ctx.paste", "Vložiť");
    m.insert("ctx.selectAll", "Vybrať všetko");
    m.insert("ctx.bringFront", "Presunúť dopredu");
    m.insert("ctx.sendBack", "Presunúť dozadu");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Žiadne");
    m.insert("ah.arrow", "Šípka");
    m.insert("ah.bar", "Čiara");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Trojuholník");
    m.insert("ah.diamond", "Diamant");
    m.insert("theme.light", "Svetlý režim");
    m.insert("theme.dark", "Tmavý režim");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Pre pohyb plátna podržte {{Scroll wheel}} alebo {{Space}} počas ťahania, alebo použite nástroj ruka");
    m.insert("library.addSelected", "Pridať do knižnice");
    m.insert("library.empty", "Zatiaľ neboli pridané žiadne položky...");
    m.insert("color.stroke", "Obrys");
    m.insert("color.background", "Pozadie");
    m.insert("prop.strokeWidth", "Hrúbka obrysu");
    m.insert("prop.fillStyle", "Výplň");
    m.insert("prop.opacity", "Priehľadnosť");
    m.insert("prop.roughness", "Štylizácia");
    m.insert("prop.fontSize", "Veľkosť písma");
    m.insert("prop.fontFamily", "Písmo");
    m.insert("prop.textAlign", "Zarovnanie textu");
    m.insert("prop.strokeStyle", "Štýl obrysu");
    m.insert("prop.arrowheads", "Zakončenie šípky");
    m.insert("prop.roundness", "Okraje");
    m.insert("prop.lock", "Zamknúť");
    m.insert("prop.unlock", "Odomknúť");
    m.insert("prop.group", "Skupina");
    m.insert("prop.ungroup", "Zrušiť zoskupenie");
    m.insert("prop.align", "Zarovnanie");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Zarovnať doľava");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Zarovnať doprava");
    m.insert("align.top", "Zarovnať nahor");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Zarovnať nadol");
    m.insert("fill.hachure", "Šrafovaná");
    m.insert("fill.solid", "Plná");
    m.insert("fill.zigzag", "Cik-cak");
    m.insert("fill.crossHatch", "Mriežkovaná");
    m.insert("stroke.solid", "Plný");
    m.insert("stroke.dashed", "Čiarkovaný");
    m.insert("stroke.dotted", "Bodkovaný");
    m.insert("round.none", "Žiadne");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Doľava");
    m.insert("textalign.center", "Na stred");
    m.insert("textalign.right", "Doprava");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Pozadie plátna");
    m.insert("menu.followUs", "Sledujte nás");
    m.insert("menu.discordGroup", "Discord chat");
    m.insert("menu.collab", "Živá spolupráca...");
    m.insert("menu.commandPalette", "Paleta príkazov");
    m.insert("menu.findOnCanvas", "Vyhľadať v plátne");
    m.insert(
        "action.zoomToFit",
        "Priblížiť aby boli zahrnuté všetky prvky",
    );
    m.insert(
        "palette.placeholder",
        "Prehľadávajte menu, príkazy a objavte skryté poklady",
    );
    m.insert("palette.noResults", "Žiadne vyhovujúce príkazy...");
    m.insert("find.placeholder", "Vyhľadať text v plátne...");
    m.insert("find.noResults", "Neboli nájdené žiadne výsledky...");
    m.insert("help.tools", "Nástroje");
    m.insert("help.actions", "Akcie");
    m.insert("help.view", "Zobrazenie");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Nastavenia");
    m.insert("menu.theme", "Téma");
    m.insert("menu.toolLock", "Uzamknutie nástroja");
    m.insert("menu.snapToObjects", "Prichytiť k objektom");
    m.insert("menu.zenMode", "Režim zen");
    m.insert("menu.viewMode", "Režim zobrazenia");
    m.insert("menu.canvasStats", "Vlastnosti plátna a tvaru");
    m.insert("menu.arrowBinding", "Pripájanie šípky");
    m.insert("menu.snapToMidpoints", "Prichytiť k bodom");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web Embed");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Laso výber");
    m.insert("toolbar.textToDiagram", "Text na diagram");
    m.insert("toolbar.mermaid", "Mermaid do Excalidraw");
    m.insert("toolbar.wireframeToCode", "Drôtený model na kód");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Vaše kresby sú uložené v pamäti vášho prehliadača.",
    );
    m.insert(
        "welcome.center2",
        "Pamäť prehliadača môže byť neočakávane vymazaná.",
    );
    m.insert(
        "welcome.center3",
        "Pravidelne si ukladajte si vašu prácu do súboru, aby ste ju nestratili.",
    );
    m.insert("stats.title", "Vlastnosti");
    m.insert("stats.shapes", "Tvary");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Konvertovať na kód");
    m.insert("labels.copySource", "Kopírovať kód do schránky");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Farby");
    m.insert("colorPicker.shades", "Odtiene");
    m.insert(
        "colorPicker.noShades",
        "Pre túto farbu nie sú dostupné žiadne odtiene",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Najpoužívanejšie vlastné farby",
    );
    m.insert("colorPicker.hexCode", "Hex kód");
    m.insert("fontList.sceneFonts", "V tejto scéne");
    m.insert("fontList.availableFonts", "Dostupné písma");
    m.insert("fontList.empty", "Nenašli sa žiadne písma");
    m.insert("quickSearch.placeholder", "Rýchle vyhľadávanie");
    m.insert("elementLink.title", "Odkaz na objekt");
    m.insert(
        "elementLink.desc",
        "Kliknite na tvar na plátne alebo vložte odkaz.",
    );
    m.insert("buttons.remove", "Odstrániť");
    m.insert("buttons.cancel", "Zrušiť");
    m.insert("buttons.confirm", "Potvrdiť");
    m
}

fn sl_si() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Izberi");
    m.insert("toolbar.hand", "Roka (orodje za premikanje)");
    m.insert("toolbar.rectangle", "Pravokotnik");
    m.insert("toolbar.diamond", "Diamant");
    m.insert("toolbar.ellipse", "Elipsa");
    m.insert("toolbar.arrow", "Puščica");
    m.insert("toolbar.line", "Črta");
    m.insert("toolbar.freedraw", "Risanje");
    m.insert("toolbar.text", "Besedilo");
    m.insert("toolbar.image", "Vstavi sliko");
    m.insert("toolbar.frame", "Okvir");
    m.insert("toolbar.eraser", "Radirka");
    m.insert("toolbar.laser", "Laserski kazalec");
    m.insert("toolbar.lock", "Zakleni");
    m.insert("toolbar.library", "Knjižnica");
    m.insert("action.undo", "Razveljavi");
    m.insert("action.redo", "Ponovi");
    m.insert("action.delete", "Izbriši");
    m.insert("action.duplicate", "Podvoji");
    m.insert("action.exportPng", "Izvozi v PNG");
    m.insert("action.exportSvg", "Izvozi v SVG");
    m.insert("action.copyToClipboard", "Kopiraj v odložišče");
    m.insert("action.zoomIn", "Povečaj");
    m.insert("action.zoomOut", "Pomanjšaj");
    m.insert("action.resetZoom", "Ponastavi povečavo");
    m.insert("action.toggleTheme", "Preklapljanje med svetlo/temno temo");
    m.insert("action.toggleGrid", "Preklopi mrežo");
    m.insert("panel.layers", "Plasti");
    m.insert("panel.properties", "Lastnosti");
    m.insert("panel.library", "Knjižnica");
    m.insert("menu.language", "Jezik");
    m.insert("menu.open", "Odpri");
    m.insert("menu.save", "Shrani v...");
    m.insert("menu.exportImage", "Izvozi sliko...");
    m.insert("menu.exportSvg", "Izvozi v SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Preklopi mrežo");
    m.insert("menu.layers", "Plasti");
    m.insert("menu.reset", "Ponastavi platno");
    m.insert("menu.help", "Pomoč");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Meni");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Odpri");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Združi izbor");
    m.insert("action.ungroup", "Razdruži izbor");
    m.insert("ctx.duplicate", "Podvoji");
    m.insert("ctx.delete", "Izbriši");
    m.insert("ctx.copy", "Kopiraj");
    m.insert("ctx.cut", "Izreži");
    m.insert("ctx.paste", "Prilepi");
    m.insert("ctx.selectAll", "Izberi vse");
    m.insert("ctx.bringFront", "Pomakni v ospredje");
    m.insert("ctx.sendBack", "Pomakni v ozadje");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Brez");
    m.insert("ah.arrow", "Puščica");
    m.insert("ah.bar", "Palica");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Trikotnik");
    m.insert("ah.diamond", "Diamant");
    m.insert("theme.light", "Svetli način");
    m.insert("theme.dark", "Temni način");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Za premikanje platna med vlečenjem držite tipko {{Scroll wheel}} ali {{Space}} ali uporabite orodje z roko");
    m.insert("library.addSelected", "Dodaj v knjižnico");
    m.insert("library.empty", "Dodan še ni noben element...");
    m.insert("color.stroke", "Poteza");
    m.insert("color.background", "Ozadje");
    m.insert("prop.strokeWidth", "Debelina poteze");
    m.insert("prop.fillStyle", "Polnilo");
    m.insert("prop.opacity", "Prekrivnost");
    m.insert("prop.roughness", "Površnost");
    m.insert("prop.fontSize", "Velikost pisave");
    m.insert("prop.fontFamily", "Družina pisave");
    m.insert("prop.textAlign", "Poravnava besedila");
    m.insert("prop.strokeStyle", "Slog poteze");
    m.insert("prop.arrowheads", "Puščice");
    m.insert("prop.roundness", "Robovi");
    m.insert("prop.lock", "Zakleni");
    m.insert("prop.unlock", "Odkleni");
    m.insert("prop.group", "Skupina");
    m.insert("prop.ungroup", "Razdruži izbor");
    m.insert("prop.align", "Poravnava");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Poravnaj levo");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Poravnaj desno");
    m.insert("align.top", "Poravnaj na vrh");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Poravnaj na dno");
    m.insert("fill.hachure", "Šrafura");
    m.insert("fill.solid", "Polno");
    m.insert("fill.zigzag", "Cikcak");
    m.insert("fill.crossHatch", "Križno");
    m.insert("stroke.solid", "Polna");
    m.insert("stroke.dashed", "Črtkana");
    m.insert("stroke.dotted", "Pikasta");
    m.insert("round.none", "Brez");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Levo");
    m.insert("textalign.center", "Sredina");
    m.insert("textalign.right", "Desno");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Ozadje platna");
    m.insert("menu.followUs", "Sledi nas");
    m.insert("menu.discordGroup", "Klepet v Discordu");
    m.insert("menu.collab", "Sodelovanje v živo...");
    m.insert("menu.commandPalette", "Paleta ukazov");
    m.insert("menu.findOnCanvas", "Poišči na platnu");
    m.insert("action.zoomToFit", "Približaj na vse elemente");
    m.insert(
        "palette.placeholder",
        "Išči po menijih, ukazih in odkrij skrite funkcije",
    );
    m.insert("palette.noResults", "Ni ustreznih ukazov ...");
    m.insert("find.placeholder", "Poišči besedilo na platnu ...");
    m.insert("find.noResults", "Ni zadetkov ...");
    m.insert("help.tools", "Orodja");
    m.insert("help.actions", "Dejanja");
    m.insert("help.view", "Pogled");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Registriraj se");
    m.insert("menu.preferences", "Nastavitve");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Zaklep orodja");
    m.insert("menu.snapToObjects", "Pripenjanje na predmete");
    m.insert("menu.zenMode", "Način Zen");
    m.insert("menu.viewMode", "Način ogleda");
    m.insert("menu.canvasStats", "Lastnosti platna in oblike");
    m.insert("menu.arrowBinding", "Vezava puščic");
    m.insert("menu.snapToMidpoints", "Pripni na srednje točke");
    m.insert("prefs.selectOn", "Izberi na");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Spletna vdelava");
    m.insert("toolbar.drawToShape", "Risanje v obliko");
    m.insert("toolbar.bucketFill", "Zapolnjevanje z vedrom");
    m.insert("toolbar.lasso", "Izbira z lassom");
    m.insert("toolbar.textToDiagram", "Besedilo v diagram");
    m.insert("toolbar.mermaid", "Mermaid v Excalidraw");
    m.insert("toolbar.wireframeToCode", "Žični okvir v kodo");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Vaše risbe so shranjene v pomnilniku vašega brskalnika.",
    );
    m.insert(
        "welcome.center2",
        "Pomnilnik brskalnika se lahko nepričakovano izprazni.",
    );
    m.insert(
        "welcome.center3",
        "Redno shranjujte svoje delo v datoteko, da ga ne izgubite.",
    );
    m.insert("stats.title", "Lastnosti");
    m.insert("stats.shapes", "Oblike");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Pretvori v kodo");
    m.insert("labels.copySource", "Kopiraj vir v odložišče");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Barve");
    m.insert("colorPicker.shades", "Odtenki");
    m.insert("colorPicker.noShades", "Odtenki za to barvo niso na voljo");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Najpogosteje uporabljene barve po meri",
    );
    m.insert("colorPicker.hexCode", "Hex koda");
    m.insert("fontList.sceneFonts", "V tej sceni");
    m.insert("fontList.availableFonts", "Razpoložljive pisave");
    m.insert("fontList.empty", "Ni pisav");
    m.insert("quickSearch.placeholder", "Hitro iskanje");
    m.insert("elementLink.title", "Povezava do objekta");
    m.insert(
        "elementLink.desc",
        "Kliknite obliko na platnu ali prilepite povezavo.",
    );
    m.insert("buttons.remove", "Odstrani");
    m.insert("buttons.cancel", "Prekliči");
    m.insert("buttons.confirm", "Potrdi");
    m
}

fn sv_se() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Välj");
    m.insert("toolbar.hand", "Hand (panoreringsverktyg)");
    m.insert("toolbar.rectangle", "Rektangel");
    m.insert("toolbar.diamond", "Romb");
    m.insert("toolbar.ellipse", "Ellips");
    m.insert("toolbar.arrow", "Pil");
    m.insert("toolbar.line", "Linje");
    m.insert("toolbar.freedraw", "Rita");
    m.insert("toolbar.text", "Text");
    m.insert("toolbar.image", "Infoga bild");
    m.insert("toolbar.frame", "Rutverktyg");
    m.insert("toolbar.eraser", "Radergummi");
    m.insert("toolbar.laser", "Laserpekare");
    m.insert("toolbar.lock", "Lås");
    m.insert("toolbar.library", "Bibliotek");
    m.insert("action.undo", "Ångra");
    m.insert("action.redo", "Gör om");
    m.insert("action.delete", "Ta bort");
    m.insert("action.duplicate", "Duplicera");
    m.insert("action.exportPng", "Exportera till PNG");
    m.insert("action.exportSvg", "Exportera till SVG");
    m.insert("action.copyToClipboard", "Kopiera till urklipp");
    m.insert("action.zoomIn", "Zooma in");
    m.insert("action.zoomOut", "Zooma ut");
    m.insert("action.resetZoom", "Återställ zoom");
    m.insert("action.toggleTheme", "Växla ljus/mörkt tema");
    m.insert("action.toggleGrid", "Visa/dölj rutnät");
    m.insert("panel.layers", "Lager");
    m.insert("panel.properties", "Egenskaper");
    m.insert("panel.library", "Bibliotek");
    m.insert("menu.language", "Språk");
    m.insert("menu.open", "Öppna");
    m.insert("menu.save", "Spara till...");
    m.insert("menu.exportImage", "Exportera bild...");
    m.insert("menu.exportSvg", "Exportera till SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Visa/dölj rutnät");
    m.insert("menu.layers", "Lager");
    m.insert("menu.reset", "Återställ canvasen");
    m.insert("menu.help", "Hjälp");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Meny");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Öppna");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Gruppera markering");
    m.insert("action.ungroup", "Avgruppera markering");
    m.insert("ctx.duplicate", "Duplicera");
    m.insert("ctx.delete", "Ta bort");
    m.insert("ctx.copy", "Kopiera");
    m.insert("ctx.cut", "Klipp ut");
    m.insert("ctx.paste", "Klistra in");
    m.insert("ctx.selectAll", "Markera alla");
    m.insert("ctx.bringFront", "Flytta främst");
    m.insert("ctx.sendBack", "Flytta underst");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Inga");
    m.insert("ah.arrow", "Pil");
    m.insert("ah.bar", "Stolpe");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Triangel");
    m.insert("ah.diamond", "Romb");
    m.insert("theme.light", "Ljust läge");
    m.insert("theme.dark", "Mörkt läge");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "För att flytta canvasen, håll {{Scroll wheel}} eller {{Space}} medan du drar eller använd handverktyget");
    m.insert("library.addSelected", "Lägg till i biblioteket");
    m.insert("library.empty", "Inga objekt tillagda ännu...");
    m.insert("color.stroke", "Linje");
    m.insert("color.background", "Bakgrund");
    m.insert("prop.strokeWidth", "Linjebredd");
    m.insert("prop.fillStyle", "Fyllnad");
    m.insert("prop.opacity", "Genomskinlighet");
    m.insert("prop.roughness", "Slarvighet");
    m.insert("prop.fontSize", "Teckenstorlek");
    m.insert("prop.fontFamily", "Teckensnitt");
    m.insert("prop.textAlign", "Textjustering");
    m.insert("prop.strokeStyle", "Linjestil");
    m.insert("prop.arrowheads", "Pilhuvuden");
    m.insert("prop.roundness", "Kanter");
    m.insert("prop.lock", "Lås");
    m.insert("prop.unlock", "Lås upp");
    m.insert("prop.group", "Grupp");
    m.insert("prop.ungroup", "Avgruppera markering");
    m.insert("prop.align", "Justera");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Justera vänster");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Justera höger");
    m.insert("align.top", "Justera överkant");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Justera underkant");
    m.insert("fill.hachure", "Skraffering");
    m.insert("fill.solid", "Solid");
    m.insert("fill.zigzag", "Sicksack");
    m.insert("fill.crossHatch", "Skraffera med kors");
    m.insert("stroke.solid", "Solid");
    m.insert("stroke.dashed", "Streckad");
    m.insert("stroke.dotted", "Punktad");
    m.insert("round.none", "Inga");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Vänster");
    m.insert("textalign.center", "Centrera");
    m.insert("textalign.right", "Höger");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Canvas-bakgrund");
    m.insert("menu.followUs", "Följ oss");
    m.insert("menu.discordGroup", "Discord chat");
    m.insert("menu.collab", "Samarbeta live...");
    m.insert("menu.commandPalette", "Kommandopalett");
    m.insert("menu.findOnCanvas", "Hitta på canvas");
    m.insert("action.zoomToFit", "Zooma för att rymma alla element");
    m.insert(
        "palette.placeholder",
        "Sök menyer, kommandon och upptäck dolda pärlor",
    );
    m.insert("palette.noResults", "Inga matchande kommandon...");
    m.insert("find.placeholder", "Hitta text på canvas...");
    m.insert("find.noResults", "Inga träffar hittades...");
    m.insert("help.tools", "Verktyg");
    m.insert("help.actions", "Åtgärder");
    m.insert("help.view", "Visa");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Registrera dig");
    m.insert("menu.preferences", "Inställningar");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Verktygslås");
    m.insert("menu.snapToObjects", "Fäst mot objekt");
    m.insert("menu.zenMode", "Zen-läge");
    m.insert("menu.viewMode", "Visningsläge");
    m.insert("menu.canvasStats", "Egenskaper för Canvas & Form");
    m.insert("menu.arrowBinding", "Pilbindning");
    m.insert("menu.snapToMidpoints", "Fäst mot mittpunkter");
    m.insert("prefs.selectOn", "Välj på");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Bädda in (web)");
    m.insert("toolbar.drawToShape", "Rita utefter form");
    m.insert("toolbar.bucketFill", "Färglägg");
    m.insert("toolbar.lasso", "Lassomarkering");
    m.insert("toolbar.textToDiagram", "Text till diagram");
    m.insert("toolbar.mermaid", "Mermaid till Excalidraw");
    m.insert("toolbar.wireframeToCode", "Trådram till kod");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Dina teckningar sparas i webbläsarens lagring.",
    );
    m.insert(
        "welcome.center2",
        "Webbläsarens lagringsutrymme kan rensas oväntat.",
    );
    m.insert(
        "welcome.center3",
        "Spara ditt arbete till en fil regelbundet för att undvika att förlora det.",
    );
    m.insert("stats.title", "Egenskaper");
    m.insert("stats.shapes", "Former");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Konvertera till kod");
    m.insert("labels.copySource", "Kopiera källa till urklipp");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Färger");
    m.insert("colorPicker.shades", "Nyanser");
    m.insert(
        "colorPicker.noShades",
        "Inga nyanser tillgängliga för denna färg",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Mest frekvent använda anpassade färger",
    );
    m.insert("colorPicker.hexCode", "Hex-kod");
    m.insert("fontList.sceneFonts", "I den här scenen");
    m.insert("fontList.availableFonts", "Tillgängliga typsnitt");
    m.insert("fontList.empty", "Inga typsnitt hittades");
    m.insert("quickSearch.placeholder", "Snabbsök");
    m.insert("elementLink.title", "Länk till objektet");
    m.insert(
        "elementLink.desc",
        "Klicka på en figur på canvas eller klistra in en länk.",
    );
    m.insert("buttons.remove", "Ta bort");
    m.insert("buttons.cancel", "Avbryt");
    m.insert("buttons.confirm", "Bekräfta");
    m
}

fn tr_tr() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Seç");
    m.insert("toolbar.hand", "El (hareket aracı)");
    m.insert("toolbar.rectangle", "Dikdörtgen");
    m.insert("toolbar.diamond", "Elmas");
    m.insert("toolbar.ellipse", "Elips");
    m.insert("toolbar.arrow", "Ok");
    m.insert("toolbar.line", "Çizgi");
    m.insert("toolbar.freedraw", "Çiz");
    m.insert("toolbar.text", "Yazı");
    m.insert("toolbar.image", "Görsel ekle");
    m.insert("toolbar.frame", "Çerçeve aracı");
    m.insert("toolbar.eraser", "Silgi");
    m.insert("toolbar.laser", "Lazer işaretçisi");
    m.insert("toolbar.lock", "Kilitle");
    m.insert("toolbar.library", "Kütüphane");
    m.insert("action.undo", "Geri Al");
    m.insert("action.redo", "Yeniden yap");
    m.insert("action.delete", "Sil");
    m.insert("action.duplicate", "Çoğalt");
    m.insert("action.exportPng", "PNG olarak dışa aktar");
    m.insert("action.exportSvg", "SVG olarak dışa aktar");
    m.insert("action.copyToClipboard", "Panoya kopyala");
    m.insert("action.zoomIn", "Yakınlaştır");
    m.insert("action.zoomOut", "Uzaklaştır");
    m.insert("action.resetZoom", "Yakınlaştırmayı sıfırla");
    m.insert("action.toggleTheme", "Aydınlık/karanlık mod");
    m.insert("action.toggleGrid", "Izgarayı aç/kapat");
    m.insert("panel.layers", "Katmanlar");
    m.insert("panel.properties", "Özellikler");
    m.insert("panel.library", "Kütüphane");
    m.insert("menu.language", "Dil");
    m.insert("menu.open", "Aç");
    m.insert("menu.save", "Şuraya kaydet...");
    m.insert("menu.exportImage", "Resimleri dışa aktar...");
    m.insert("menu.exportSvg", "SVG olarak dışa aktar");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Izgarayı aç/kapat");
    m.insert("menu.layers", "Katmanlar");
    m.insert("menu.reset", "Tuvali sıfırla");
    m.insert("menu.help", "Yardım");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Menü");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Aç");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Seçimi grup yap");
    m.insert("action.ungroup", "Seçilen grubu dağıt");
    m.insert("ctx.duplicate", "Çoğalt");
    m.insert("ctx.delete", "Sil");
    m.insert("ctx.copy", "Kopyala");
    m.insert("ctx.cut", "Kes");
    m.insert("ctx.paste", "Yapıştır");
    m.insert("ctx.selectAll", "Tümünü seç");
    m.insert("ctx.bringFront", "En öne getir");
    m.insert("ctx.sendBack", "Arkaya gönder");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Yok");
    m.insert("ah.arrow", "Ok");
    m.insert("ah.bar", "Çizgi");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Üçgen");
    m.insert("ah.diamond", "Elmas");
    m.insert("theme.light", "Açık tema");
    m.insert("theme.dark", "Koyu tema");
    m.insert("stats.elements", "Elements");
    m.insert("hint.moveCanvas", "Tuvali taşımak için {{Scroll wheel}} kısayolunu basılı tutun, sürüklerken {{Space}} kısayolunu veya el aracını kullanın");
    m.insert("library.addSelected", "Kütüphaneye ekle");
    m.insert("library.empty", "Öğe eklenmedi...");
    m.insert("color.stroke", "Kontur");
    m.insert("color.background", "Arka plan");
    m.insert("prop.strokeWidth", "Kontur genişliği");
    m.insert("prop.fillStyle", "Doldur");
    m.insert("prop.opacity", "Opaklık");
    m.insert("prop.roughness", "Üstün körülük");
    m.insert("prop.fontSize", "Yazı tipi boyutu");
    m.insert("prop.fontFamily", "Yazı tipi ailesi");
    m.insert("prop.textAlign", "Metin hizala");
    m.insert("prop.strokeStyle", "Kontur stili");
    m.insert("prop.arrowheads", "Ok uçları");
    m.insert("prop.roundness", "Kenarlar");
    m.insert("prop.lock", "Kilitle");
    m.insert("prop.unlock", "Kilidi Kaldır");
    m.insert("prop.group", "Grup");
    m.insert("prop.ungroup", "Seçilen grubu dağıt");
    m.insert("prop.align", "Hizala");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Sola hizala");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Sağa hizala");
    m.insert("align.top", "Yukarı hizala");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Aşağı hizala");
    m.insert("fill.hachure", "Taralı");
    m.insert("fill.solid", "Dolu");
    m.insert("fill.zigzag", "Zikzak");
    m.insert("fill.crossHatch", "Çapraz-taralı");
    m.insert("stroke.solid", "Dolu");
    m.insert("stroke.dashed", "Kesik çizgili");
    m.insert("stroke.dotted", "Noktalı");
    m.insert("round.none", "Yok");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Sol");
    m.insert("textalign.center", "Ortala");
    m.insert("textalign.right", "Sağ");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Tuval arka planı");
    m.insert("menu.followUs", "Bizi takip edin");
    m.insert("menu.discordGroup", "Discord sohbeti");
    m.insert("menu.collab", "Canlı ortak çalışma alanı...");
    m.insert("menu.commandPalette", "Komut paleti");
    m.insert("menu.findOnCanvas", "Tuvalde bul");
    m.insert(
        "action.zoomToFit",
        "Tüm öğeleri sığdırmak için yakınlaştırın",
    );
    m.insert(
        "palette.placeholder",
        "Menülerde, komutlarda arama yapın ve saklı kalmış hazineleri keşfedin",
    );
    m.insert("palette.noResults", "Eşleşen komut yok...");
    m.insert("find.placeholder", "Tuvalde metin bul...");
    m.insert("find.noResults", "Sonuç bulunamadı...");
    m.insert("help.tools", "Araçlar");
    m.insert("help.actions", "Eylemler");
    m.insert("help.view", "Görünüm");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Tercihler");
    m.insert("menu.theme", "Tema");
    m.insert("menu.toolLock", "Aracı Kilitle");
    m.insert("menu.snapToObjects", "Nesnelere hizala");
    m.insert("menu.zenMode", "Zen modu");
    m.insert("menu.viewMode", "Görünüm modu");
    m.insert("menu.canvasStats", "Tuval ve şekil özellikleri");
    m.insert("menu.arrowBinding", "Arrow binding");
    m.insert("menu.snapToMidpoints", "Snap to midpoints");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Web Yerleştirme");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Bucket fill");
    m.insert("toolbar.lasso", "Serbest seçim");
    m.insert("toolbar.textToDiagram", "Yazıdan diyagrama");
    m.insert("toolbar.mermaid", "Mermaid'den Excalidraw'a");
    m.insert("toolbar.wireframeToCode", "Wireframe'den koda");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Çizimleriniz tarayıcınızın depolama alanına kaydedilir.",
    );
    m.insert(
        "welcome.center2",
        "Tarayıcı depolama alanı beklenmedik bir şekilde silinebilir.",
    );
    m.insert(
        "welcome.center3",
        "Veri kaybını önlemek için çalışmanızı düzenli olarak bir dosyaya kaydedin.",
    );
    m.insert("stats.title", "Özellikler");
    m.insert("stats.shapes", "Şekiller");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Koda dönüştür");
    m.insert("labels.copySource", "Kaynağı panoya kopyala");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Renkler");
    m.insert("colorPicker.shades", "Tonlar");
    m.insert("colorPicker.noShades", "Bu renk için ton mevcut değil");
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "En çok kullanılan özel renkler",
    );
    m.insert("colorPicker.hexCode", "Hex kodu");
    m.insert("fontList.sceneFonts", "Bu sahnede");
    m.insert("fontList.availableFonts", "Kullanılabilir yazı tipleri");
    m.insert("fontList.empty", "Yazı tipi bulunamadı");
    m.insert("quickSearch.placeholder", "Hızlı arama");
    m.insert("elementLink.title", "Nesneye bağlantıla");
    m.insert(
        "elementLink.desc",
        "Tuvaldeki bir nesneye tıklayın veya bir bağlantıyı yapıştırın.",
    );
    m.insert("buttons.remove", "Kaldır");
    m.insert("buttons.cancel", "İptal");
    m.insert("buttons.confirm", "Onayla");
    m
}

fn uk_ua() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("toolbar.select", "Вибрати");
    m.insert("toolbar.hand", "Рука (інструмент для панорамування)");
    m.insert("toolbar.rectangle", "Прямокутник");
    m.insert("toolbar.diamond", "Ромб");
    m.insert("toolbar.ellipse", "Еліпс");
    m.insert("toolbar.arrow", "Стрілка");
    m.insert("toolbar.line", "Лінія");
    m.insert("toolbar.freedraw", "Малювати");
    m.insert("toolbar.text", "Текст");
    m.insert("toolbar.image", "Вставити зображення");
    m.insert("toolbar.frame", "Інструмент фрейму");
    m.insert("toolbar.eraser", "Гумка");
    m.insert("toolbar.laser", "Лазерний вказівник");
    m.insert("toolbar.lock", "Блокувати");
    m.insert("toolbar.library", "Бібліотека");
    m.insert("action.undo", "Відмінити");
    m.insert("action.redo", "Повторити");
    m.insert("action.delete", "Видалити");
    m.insert("action.duplicate", "Дублювати");
    m.insert("action.exportPng", "Експортувати в PNG");
    m.insert("action.exportSvg", "Експортувати у SVG");
    m.insert("action.copyToClipboard", "Скопіювати до буферу обміну");
    m.insert("action.zoomIn", "Збільшити");
    m.insert("action.zoomOut", "Зменшити");
    m.insert("action.resetZoom", "Скинути масштаб");
    m.insert("action.toggleTheme", "Перемикати світлу/темну теми");
    m.insert("action.toggleGrid", "Перемикати сітку");
    m.insert("panel.layers", "Шари");
    m.insert("panel.properties", "Властивості");
    m.insert("panel.library", "Бібліотека");
    m.insert("menu.language", "Мова");
    m.insert("menu.open", "Відкрити");
    m.insert("menu.save", "Зберегти як...");
    m.insert("menu.exportImage", "Експорт зображення...");
    m.insert("menu.exportSvg", "Експортувати у SVG");
    m.insert("menu.snap", "Snap to grid");
    m.insert("menu.grid", "Перемикати сітку");
    m.insert("menu.layers", "Шари");
    m.insert("menu.reset", "Очистити полотно");
    m.insert("menu.help", "Допомога");
    m.insert(
        "menu.helpHint",
        "Drag to draw. Pick a tool from the toolbar, or press its key.",
    );
    m.insert("menu.file", "Меню");
    m.insert("menu.new", "New");
    m.insert("menu.load", "Відкрити");
    m.insert("action.snap", "Snap to grid");
    m.insert("action.group", "Групувати виділене");
    m.insert("action.ungroup", "Розгрупувати виділене");
    m.insert("ctx.duplicate", "Дублювати");
    m.insert("ctx.delete", "Видалити");
    m.insert("ctx.copy", "Копіювати");
    m.insert("ctx.cut", "Вирізати");
    m.insert("ctx.paste", "Вставити");
    m.insert("ctx.selectAll", "Вибрати все");
    m.insert("ctx.bringFront", "На передній план");
    m.insert("ctx.sendBack", "На задній план");
    m.insert("prop.start", "Start");
    m.insert("prop.end", "End");
    m.insert("ah.none", "Жоден");
    m.insert("ah.arrow", "Стрілка");
    m.insert("ah.bar", "Колона");
    m.insert("ah.dot", "Dot");
    m.insert("ah.triangle", "Трикутник");
    m.insert("ah.diamond", "Ромб");
    m.insert("theme.light", "Світла тема");
    m.insert("theme.dark", "Темна тема");
    m.insert("stats.elements", "Elements");
    m.insert(
        "hint.moveCanvas",
        "To move canvas, hold {{Scroll wheel}} or {{Space}} while dragging, or use the hand tool",
    );
    m.insert("library.addSelected", "Додати до бібліотеки");
    m.insert("library.empty", "Тут поки пусто...");
    m.insert("color.stroke", "Контур");
    m.insert("color.background", "Тло");
    m.insert("prop.strokeWidth", "Товщина контуру");
    m.insert("prop.fillStyle", "Заповнити");
    m.insert("prop.opacity", "Прозорість");
    m.insert("prop.roughness", "Охайність");
    m.insert("prop.fontSize", "Розмір шрифту");
    m.insert("prop.fontFamily", "Шрифт");
    m.insert("prop.textAlign", "Вирівнювання тексту");
    m.insert("prop.strokeStyle", "Стиль контуру");
    m.insert("prop.arrowheads", "Закінчення стрілки");
    m.insert("prop.roundness", "Краї");
    m.insert("prop.lock", "Блокувати");
    m.insert("prop.unlock", "Розблокувати");
    m.insert("prop.group", "Група");
    m.insert("prop.ungroup", "Розгрупувати виділене");
    m.insert("prop.align", "Вирівнювання");
    m.insert("prop.distribute", "Distribute");
    m.insert("align.left", "Вирівняти по лівому краю");
    m.insert("align.centerH", "Align center");
    m.insert("align.right", "Вирівнювання по правому краю");
    m.insert("align.top", "Вирівняти по верхньому краю");
    m.insert("align.centerV", "Align middle");
    m.insert("align.bottom", "Вирівняти по нижньому краю");
    m.insert("fill.hachure", "Штриховка");
    m.insert("fill.solid", "Суцільна");
    m.insert("fill.zigzag", "Зиґзаґ");
    m.insert("fill.crossHatch", "Перехресна штриховка");
    m.insert("stroke.solid", "Суцільний");
    m.insert("stroke.dashed", "Пунктир");
    m.insert("stroke.dotted", "Крапки");
    m.insert("round.none", "Жоден");
    m.insert("round.small", "S");
    m.insert("round.medium", "M");
    m.insert("round.large", "L");
    m.insert("font.virgil", "Virgil");
    m.insert("font.helvetica", "Helvetica");
    m.insert("font.cascadia", "Cascadia");
    m.insert("font.comic", "Comic");
    m.insert("textalign.left", "Зліва");
    m.insert("textalign.center", "По центру");
    m.insert("textalign.right", "Справа");
    m.insert("welcome.title", "Excalidraw");
    m.insert(
        "welcome.subtitle",
        "Virtual whiteboard for sketching hand-drawn like diagrams",
    );
    m.insert("menu.canvasBackground", "Тло полотна");
    m.insert("menu.followUs", "Підписатися");
    m.insert("menu.discordGroup", "Чат в Discord");
    m.insert("menu.collab", "Спільна робота наживо...");
    m.insert("menu.commandPalette", "Командна палітра");
    m.insert("menu.findOnCanvas", "Знайти на полотні");
    m.insert("action.zoomToFit", "Збільшити, щоб умістити всі елементи");
    m.insert(
        "palette.placeholder",
        "Шукайте меню, команди та відкривайте приховані перлини",
    );
    m.insert("palette.noResults", "Немає відповідних команд...");
    m.insert("find.placeholder", "Знайти текст на полотні...");
    m.insert("find.noResults", "Збігів не знайдено...");
    m.insert("help.tools", "Інструменти");
    m.insert("help.actions", "Дії");
    m.insert("help.view", "Вигляд");
    m.insert("help.files", "Files");
    m.insert("menu.signUp", "Sign up");
    m.insert("menu.preferences", "Preferences");
    m.insert("menu.theme", "Тема");
    m.insert("menu.toolLock", "Tool lock");
    m.insert("menu.snapToObjects", "Прив'язати до об'єктів");
    m.insert("menu.zenMode", "Режим Дзен");
    m.insert("menu.viewMode", "Режим перегляду");
    m.insert("menu.canvasStats", "Властивості полотна та форми");
    m.insert("menu.arrowBinding", "Arrow binding");
    m.insert("menu.snapToMidpoints", "Snap to midpoints");
    m.insert("prefs.selectOn", "Select on");
    m.insert("prefs.input", "Input");
    m.insert("toolbar.embed", "Веб вкладення");
    m.insert("toolbar.drawToShape", "Draw to shape");
    m.insert("toolbar.bucketFill", "Заливка");
    m.insert("toolbar.lasso", "Ласо");
    m.insert("toolbar.textToDiagram", "Діаграма з тексту");
    m.insert("toolbar.mermaid", "Mermaid у Excalidraw");
    m.insert("toolbar.wireframeToCode", "Перетворити у код");
    m.insert("toolbar.generate", "Generate");
    m.insert(
        "welcome.center1",
        "Ваші малюнки зберігаються у сховищі браузера.",
    );
    m.insert(
        "welcome.center2",
        "Дані, збережені в браузері, можуть бути несподівано видалені.",
    );
    m.insert(
        "welcome.center3",
        "Регулярно зберігайте свою роботу у файл, щоб не втратити її.",
    );
    m.insert("stats.title", "Властивості");
    m.insert("stats.shapes", "Форми");
    m.insert("stats.selectedType", "Type");
    m.insert("status.saved", "Saved");
    m.insert(
        "status.unavailable",
        "Needs the Excalidraw backend — not available in this build",
    );
    m.insert("labels.convertToCode", "Перетворити у код");
    m.insert("labels.copySource", "Скопіювати джерело в буфер обміну");
    m.insert("fileDrop.replace", "Drop to replace content");
    m.insert("fileDrop.add", "Drop to add to canvas");
    m.insert("fileDrop.importLibrary", "Drop to import library");
    m.insert(
        "fileDrop.replaceHint",
        "Hold {{Shift}} to keep existing content",
    );
    m.insert("fileDrop.keepHint", "Your existing content will be kept");
    m.insert("fileDrop.libraryHint", "Library files will append");
    m.insert(
        "fileDrop.replacedToast",
        "Content replaced. {{Ctrl+Z}} to undo",
    );
    m.insert("colorPicker.colors", "Кольори");
    m.insert("colorPicker.shades", "Тіні");
    m.insert(
        "colorPicker.noShades",
        "Немає доступних відтінків цього кольору",
    );
    m.insert(
        "colorPicker.mostUsedCustomColors",
        "Найбільш використовувані користувацькі кольори",
    );
    m.insert("colorPicker.hexCode", "Hex-код");
    m.insert("fontList.sceneFonts", "На цій сцені");
    m.insert("fontList.availableFonts", "Доступні шрифти");
    m.insert("fontList.empty", "Шрифти не знайдено");
    m.insert("quickSearch.placeholder", "Швидкий пошук");
    m.insert("elementLink.title", "Посилання на об’єкт");
    m.insert(
        "elementLink.desc",
        "Натисніть на фігуру на полотні або вставте посилання.",
    );
    m.insert("buttons.remove", "Видалити");
    m.insert("buttons.cancel", "Скасувати");
    m.insert("buttons.confirm", "Підтвердити");
    m
}

#[cfg(test)]
mod locale_tests {
    use super::*;

    #[test]
    fn locale_tags_map_like_the_site_language_picker() {
        assert_eq!(Language::from_locale("zh_CN.UTF-8"), Language::ZhCn);
        assert_eq!(Language::from_locale("zh_TW.UTF-8"), Language::ZhTw);
        assert_eq!(Language::from_locale("ja_JP"), Language::Ja);
        assert_eq!(Language::from_locale("ko_KR.eucKR"), Language::Ko);
        assert_eq!(Language::from_locale("de_DE@euro"), Language::De);
        assert_eq!(Language::from_locale("pt_BR"), Language::PtBr);
        assert_eq!(Language::from_locale("pt_PT"), Language::PtPt);
        assert_eq!(Language::from_locale("he_IL"), Language::He);
    }

    #[test]
    fn undetermined_locales_fall_back_to_english() {
        assert_eq!(Language::from_locale("C"), Language::En);
        assert_eq!(Language::from_locale("C.UTF-8"), Language::En);
        assert_eq!(Language::from_locale("POSIX"), Language::En);
        assert_eq!(Language::from_locale(""), Language::En);
        assert_eq!(Language::from_locale("fr_FR.UTF-8"), Language::Fr);
    }

    #[test]
    fn the_picker_offers_upstreams_locale_list() {
        let expected = [
            ("en", "English"),
            ("id-ID", "Bahasa Indonesia"),
            ("de-DE", "Deutsch"),
            ("es-ES", "Español"),
            ("eu-ES", "Euskara"),
            ("fr-FR", "Français"),
            ("it-IT", "Italiano"),
            ("nl-NL", "Nederlands"),
            ("pl-PL", "Polski"),
            ("pt-PT", "Português"),
            ("pt-BR", "Português Brasileiro"),
            ("ro-RO", "Română"),
            ("sk-SK", "Slovenčina"),
            ("sl-SI", "Slovenščina"),
            ("sv-SE", "Svenska"),
            ("tr-TR", "Türkçe"),
            ("ru-RU", "Русский"),
            ("uk-UA", "Українська"),
            ("he-IL", "עברית"),
            ("ar-SA", "العربية"),
            ("fa-IR", "فارسی"),
            ("ja-JP", "日本語"),
            ("zh-CN", "简体中文"),
            ("zh-TW", "繁體中文"),
            ("ko-KR", "한국어"),
        ];
        let offered: Vec<(&str, &str)> = Language::all()
            .into_iter()
            .map(|language| (language.code(), language.label()))
            .collect();
        assert_eq!(offered, expected);

        let tables = translations();
        for (code, _) in expected {
            assert!(tables.contains_key(code), "{code} ships no table");
        }
    }

    #[test]
    fn every_locale_actually_translates_most_of_its_table() {
        let tables = translations();
        let en = &tables["en"];
        for (code, table) in &tables {
            if *code == "en" {
                continue;
            }
            let translated = table
                .iter()
                .filter(|(key, value)| en.get(*key).is_some_and(|english| english != *value))
                .count();
            assert!(
                translated >= 100,
                "{code} only translates {translated} of its {} entries",
                table.len()
            );
        }
    }
}
