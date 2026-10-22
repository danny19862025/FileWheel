use std::{
    fs,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim().strip_prefix('#').unwrap_or(value.trim());
        if value.len() != 6 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        Some(Self(
            u8::from_str_radix(&value[..2], 16).ok()?,
            u8::from_str_radix(&value[2..4], 16).ok()?,
            u8::from_str_radix(&value[4..], 16).ok()?,
        ))
    }
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
    pub fn rgb(self) -> (f64, f64, f64) {
        (
            self.0 as f64 / 255.,
            self.1 as f64 / 255.,
            self.2 as f64 / 255.,
        )
    }
    pub fn blend(self, other: Self, amount: f64) -> Self {
        let mix = |a: u8, b: u8| (a as f64 * (1. - amount) + b as f64 * amount).round() as u8;
        Self(
            mix(self.0, other.0),
            mix(self.1, other.1),
            mix(self.2, other.2),
        )
    }
    fn luminance(self) -> f64 {
        let linear = |n: u8| {
            let n = n as f64 / 255.;
            if n <= 0.04045 {
                n / 12.92
            } else {
                ((n + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
    }
    fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    fn ink(self) -> Self {
        let dark = Self(16, 20, 27);
        if self.contrast(dark) >= self.contrast(Self(255, 255, 255)) {
            dark
        } else {
            Self(255, 255, 255)
        }
    }
    fn readable_on(self, background: Self) -> Self {
        let ink = background.ink();
        for step in 0..=20 {
            let candidate = self.blend(ink, step as f64 / 20.);
            if candidate.contrast(background) >= 4.5 {
                return candidate;
            }
        }
        ink
    }
}

pub const ACCENTS: [(&str, Color); 8] = [
    ("Orange", Color(252, 146, 89)),
    ("Coral", Color(243, 111, 129)),
    ("Gold", Color(235, 181, 59)),
    ("Mint", Color(70, 190, 151)),
    ("Teal", Color(45, 184, 196)),
    ("Blue", Color(87, 157, 246)),
    ("Violet", Color(164, 133, 247)),
    ("Pink", Color(219, 122, 209)),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

impl Theme {
    pub fn index(self) -> u32 {
        match self {
            Self::Dark => 0,
            Self::Light => 1,
            Self::System => 2,
        }
    }
    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Light,
            2 => Self::System,
            _ => Self::Dark,
        }
    }
    pub fn is_dark(self, system_dark: bool) -> bool {
        match self {
            Self::Dark => true,
            Self::Light => false,
            Self::System => system_dark,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Theme,
    pub accent: Color,
    pub hover_sound: bool,
    pub hover_glow: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            accent: ACCENTS[0].1,
            hover_sound: true,
            hover_glow: true,
        }
    }
}

impl Preferences {
    pub fn path() -> Option<PathBuf> {
        let root = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|v| PathBuf::from(v).join(".config")))?;
        Some(root.join("filewheel/appearance.conf"))
    }
    pub fn parse(text: &str) -> Self {
        let mut prefs = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "theme" => {
                    prefs.theme = match value.trim() {
                        "light" => Theme::Light,
                        "system" => Theme::System,
                        _ => Theme::Dark,
                    }
                }
                "accent" => {
                    if let Some(color) = Color::parse(value) {
                        prefs.accent = color;
                    }
                }
                "hover_sound" => prefs.hover_sound = value.trim() != "false",
                "hover_glow" => prefs.hover_glow = value.trim() != "false",
                _ => {}
            }
        }
        prefs
    }
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|s| Self::parse(&s))
            .unwrap_or_default()
    }
    pub fn save(self, path: &Path) -> std::io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("No settings directory"))?;
        fs::create_dir_all(parent)?;
        let temp = parent.join(format!(".appearance-{}.tmp", std::process::id()));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(0o600)
                .open(&temp)?;
            writeln!(
                file,
                "theme={}\naccent={}\nhover_sound={}\nhover_glow={}",
                self.theme.name(),
                self.accent.hex(),
                self.hover_sound,
                self.hover_glow
            )?;
            file.sync_all()?;
            fs::rename(&temp, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub text: Color,
    pub muted: Color,
    pub surface: Color,
    pub button: Color,
    pub hover: Color,
    pub border: Color,
    pub disabled: Color,
    pub accent: Color,
    pub accent_ink: Color,
    pub accent_label: Color,
    pub accent_soft: Color,
    pub success: Color,
    pub error: Color,
}

impl Palette {
    pub fn new(prefs: Preferences, system_dark: bool) -> Self {
        let dark = prefs.theme.is_dark(system_dark);
        let (bg, text, muted, surface, button, hover, border, disabled, success, error) = if dark {
            (
                Color(23, 25, 28),
                Color(239, 240, 242),
                Color(158, 165, 177),
                Color(32, 35, 40),
                Color(43, 47, 55),
                Color(57, 63, 73),
                Color(65, 72, 84),
                Color(112, 120, 133),
                Color(145, 217, 173),
                Color(255, 162, 142),
            )
        } else {
            (
                Color(247, 248, 251),
                Color(28, 35, 47),
                Color(91, 103, 122),
                Color(255, 255, 255),
                Color(230, 234, 241),
                Color(214, 222, 234),
                Color(198, 207, 220),
                Color(131, 142, 159),
                Color(29, 117, 72),
                Color(174, 49, 46),
            )
        };
        let accent_soft = surface.blend(prefs.accent, if dark { 0.18 } else { 0.14 });
        Self {
            bg,
            text,
            muted,
            surface,
            button,
            hover,
            border,
            disabled,
            accent: prefs.accent,
            accent_ink: prefs.accent.ink(),
            accent_label: prefs.accent.readable_on(accent_soft).readable_on(bg),
            accent_soft,
            success,
            error,
        }
    }
    pub fn css(self) -> String {
        let colors = [
            ("bg", self.bg),
            ("text", self.text),
            ("muted", self.muted),
            ("surface", self.surface),
            ("button", self.button),
            ("hover", self.hover),
            ("border", self.border),
            ("disabled", self.disabled),
            ("accent", self.accent),
            ("accent_ink", self.accent_ink),
            ("accent_label", self.accent_label),
            ("accent_soft", self.accent_soft),
            ("success", self.success),
            ("error", self.error),
        ];
        let mut css = colors
            .iter()
            .map(|(name, color)| format!("@define-color fw_{name} {};\n", color.hex()))
            .collect::<String>();
        css.push_str(include_str!("style.css"));
        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_settings_cannot_inject_css_and_valid_fields_survive() {
        let prefs =
            Preferences::parse("theme=light\naccent=red; } window { color: red\nunknown=anything");
        assert_eq!(prefs.theme, Theme::Light);
        assert_eq!(prefs.accent, Preferences::default().accent);
        assert_eq!(Color::parse("#1a2B3c"), Some(Color(26, 43, 60)));
        for value in ["#123", "#💛💛", "#11223344", "#12xz56"] {
            assert!(Color::parse(value).is_none());
        }
    }
    #[test]
    fn preferences_survive_saving_and_reloading() {
        let work = crate::engine::WorkDir::new(&std::env::temp_dir()).unwrap();
        let path = work.path.join("config/appearance.conf");
        for theme in [Theme::Light, Theme::System, Theme::Dark] {
            let prefs = Preferences {
                theme,
                accent: Color(18, 52, 86),
                hover_sound: false,
                hover_glow: false,
            };
            prefs.save(&path).unwrap();
            assert_eq!(Preferences::load(&path), prefs);
        }
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }
    #[test]
    fn themes_and_extreme_custom_accents_keep_text_readable() {
        for dark in [false, true] {
            for color in ACCENTS.iter().map(|(_, c)| *c).chain([
                Color(0, 0, 0),
                Color(255, 255, 255),
                Color(127, 127, 127),
            ]) {
                let p = Palette::new(
                    Preferences {
                        theme: Theme::System,
                        accent: color,
                        ..Preferences::default()
                    },
                    dark,
                );
                assert!(p.text.contrast(p.bg) >= 7.);
                assert!(p.muted.contrast(p.surface) >= 4.5);
                assert!(p.accent_ink.contrast(p.accent) >= 4.5);
                assert!(p.accent_label.contrast(p.bg) >= 4.5);
                assert!(p.accent_label.contrast(p.accent_soft) >= 4.5);
            }
        }
        assert!(Theme::System.is_dark(true));
        assert!(!Theme::System.is_dark(false));
    }
}
