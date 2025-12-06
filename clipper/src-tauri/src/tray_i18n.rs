use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    En,
    Zh,
}

impl Language {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zh" | "zh-cn" | "zh-hans" => Language::Zh,
            _ => Language::En,
        }
    }
}

/// Get translations for tray menu
pub fn get_translations(lang: Language) -> HashMap<&'static str, &'static str> {
    let mut translations = HashMap::new();

    match lang {
        Language::En => {
            translations.insert("tray.showHide", "Open Clipper");
            translations.insert("tray.settings", "Settings...");
            translations.insert("tray.about", "About Clipper");
            translations.insert("tray.checkUpdates", "Check for Updates...");
            translations.insert("tray.quit", "Quit Application");
        }
        Language::Zh => {
            translations.insert("tray.showHide", "打开 Clipper");
            translations.insert("tray.settings", "设置...");
            translations.insert("tray.about", "关于 Clipper");
            translations.insert("tray.checkUpdates", "检查更新...");
            translations.insert("tray.quit", "退出应用");
        }
    }

    translations
}

/// Get a specific translation
pub fn t(lang: Language, key: &'static str) -> &'static str {
    let translations = get_translations(lang);
    translations.get(key).copied().unwrap_or(key)
}
