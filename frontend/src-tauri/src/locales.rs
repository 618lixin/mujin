#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    ZhCn,
}

impl Locale {
    pub fn from_tag(_value: &str) -> Self {
        Self::ZhCn
    }
}

pub fn app_name(_locale: Locale) -> &'static str {
    "槿年"
}

pub fn main_window_title(locale: Locale) -> &'static str {
    app_name(locale)
}

pub fn diary_window_title(_locale: Locale) -> &'static str {
    "快速记录"
}

pub fn pinboard_window_title(_locale: Locale) -> &'static str {
    "槿年 置顶"
}

pub fn tray_tooltip(locale: Locale) -> &'static str {
    app_name(locale)
}

pub fn tray_show_main_label(_locale: Locale) -> &'static str {
    "打开主窗口"
}

pub fn tray_quick_entry_label(_locale: Locale) -> &'static str {
    "快速记录"
}

pub fn tray_toggle_close_to_tray_label(_locale: Locale) -> &'static str {
    "关闭到托盘"
}

pub fn tray_toggle_autostart_label(_locale: Locale) -> &'static str {
    "开机自启动"
}

pub fn tray_quit_label(_locale: Locale) -> &'static str {
    "退出"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_locale_tag_as_source_locale() {
        assert_eq!(Locale::from_tag("zh-CN"), Locale::ZhCn);
        assert_eq!(Locale::from_tag("en-US"), Locale::ZhCn);
        assert_eq!(Locale::from_tag("zh-HK"), Locale::ZhCn);
        assert_eq!(Locale::from_tag("zh-TW"), Locale::ZhCn);
        assert_eq!(Locale::from_tag("fr-FR"), Locale::ZhCn);
    }

    #[test]
    fn native_shell_strings_use_simplified_chinese_only() {
        assert_eq!(app_name(Locale::ZhCn), "槿年");
        assert_eq!(diary_window_title(Locale::ZhCn), "快速记录");
        assert_eq!(pinboard_window_title(Locale::ZhCn), "槿年 置顶");
        assert_eq!(tray_tooltip(Locale::ZhCn), "槿年");
        assert_eq!(tray_show_main_label(Locale::ZhCn), "打开主窗口");
        assert_eq!(tray_quick_entry_label(Locale::ZhCn), "快速记录");
        assert_eq!(tray_toggle_close_to_tray_label(Locale::ZhCn), "关闭到托盘");
        assert_eq!(tray_toggle_autostart_label(Locale::ZhCn), "开机自启动");
        assert_eq!(tray_quit_label(Locale::ZhCn), "退出");
    }
}
