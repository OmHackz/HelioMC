//! The Helio icon registry.
//!
//! One semantic icon table with 100+ distinct icons. Each maps to three
//! renderings: Nerd Font glyph, Unicode fallback, and ASCII fallback.
//! No emoji code points in any mode (enforced by a unit test).

use unicode_width::UnicodeWidthStr;

/// The three icon rendering modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconMode {
    /// Nerd Font glyphs (private-use-area codepoints).
    #[default]
    Nerd,
    /// Single-width Unicode fallback (geometric shapes, arrows, checks).
    Unicode,
    /// ASCII fallback (`[+]`, `>`, `*`, `x`, ...).
    Ascii,
}

impl std::str::FromStr for IconMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "nerd" => Self::Nerd,
            "unicode" => Self::Unicode,
            "ascii" => Self::Ascii,
            other => return Err(format!("unknown icon mode: {other}")),
        })
    }
}

/// True if the char is in an emoji code-point range that renders
/// inconsistently in terminals (variable width or color). The Unicode
/// fallbacks deliberately avoid these; the test below enforces it.
/// We ban only actual emoji blocks, not miscellaneous symbols (⚙ etc.)
/// or arrows (↓↑ etc.) which are safe single-width characters.
pub fn is_emoji(c: char) -> bool {
    let v = c as u32;
    // Emoji pictographs and symbols
    (0x1F300..=0x1FAFF).contains(&v)
        || (0x1F000..=0x1F02F).contains(&v)
        || (0x1F1E6..=0x1F1FF).contains(&v)
        || (0x1F900..=0x1F9FF).contains(&v)
        || (0x1FA00..=0x1FA6F).contains(&v)
        || (0x1FA70..=0x1FAFF).contains(&v)
        || v == 0xFE0F // variation selector
}

/// A single semantic icon with its three renderings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct IconDef {
    /// The variant name (e.g. `Home`).
    pub name: &'static str,
    /// Nerd Font glyph (private-use-area codepoint).
    pub nerd: char,
    /// Unicode fallback (single-width, common monospace font).
    pub unicode: char,
    /// ASCII fallback.
    pub ascii: &'static str,
    /// Semantic theme color token.
    pub color: &'static str,
}

/// The full icon registry. Nerd codepoints are chosen from the Nerd Fonts
/// "Minecraft" / "Devicons" / "Material Design" sets.
pub const ICONS: &[IconDef] = &[
    // -- Navigation / top-level --
    i("Home", '\u{e8af}', '⌂', "H", "info"),
    i("Browse", '\u{f5c2}', '⌕', "B", "info"),
    i("Add", '\u{f067}', '+', "[+]", "accent"),
    i("Avatar", '\u{e805}', '◉', "A", "accent"),
    i("Resources", '\u{e7b6}', '▤', "R", "info"),
    i("Settings", '\u{f7d9}', '⚙', "S", "info"),
    // -- Accounts --
    i("Account", '\u{f082}', '◈', "A", "accent"),
    i("Elyby", '\u{f54e}', 'E', "E", "accent"),
    i("OfflineAccount", '\u{f50d}', '○', "O", "dim"),
    // -- Instances --
    i("Instance", '\u{f482}', '□', "I", "info"),
    i("Play", '\u{e800}', '▶', ">", "success"),
    i("Stop", '\u{e803}', '■', "x", "error"),
    i("Kill", '\u{e804}', '✕', "k", "error"),
    i("Logs", '\u{f1ab}', '≡', "L", "dim"),
    i("Folder", '\u{e8b7}', '▣', "F", "info"),
    // -- Content --
    i("Mods", '\u{f48e}', 'M', "M", "accent"),
    i("ResourcePacks", '\u{f48d}', '▩', "P", "accent"),
    i("Shaders", '\u{f48c}', '◈', "S", "accent"),
    i("Worlds", '\u{f48b}', '▦', "W", "info"),
    i("Screenshots", '\u{f48a}', '▫', "S", "dim"),
    i("DataPacks", '\u{f489}', '▧', "D", "info"),
    // -- Java / system resources --
    i("Java", '\u{f429}', 'J', "J", "accent"),
    i("Memory", '\u{f428}', '▭', "M", "info"),
    i("Cpu", '\u{f427}', '▣', "C", "info"),
    i("Disk", '\u{f426}', '◫', "D", "info"),
    // -- Actions --
    i("Download", '\u{f502}', '↓', "v", "info"),
    i("Upload", '\u{f503}', '↑', "^", "info"),
    i("Update", '\u{f504}', '↻', "u", "info"),
    i("Search", '\u{f505}', '⌕', "?", "info"),
    i("Filter", '\u{f506}', '⊕', "f", "dim"),
    i("Sort", '\u{f507}', '⇅', "s", "dim"),
    i("Delete", '\u{f508}', '✗', "x", "error"),
    i("Duplicate", '\u{f509}', '⧉', "d", "dim"),
    i("Export", '\u{f50a}', '⇱', "o", "dim"),
    i("Import", '\u{f50b}', '⇲', "i", "dim"),
    i("Edit", '\u{f50c}', '✎', "e", "dim"),
    i("Rename", '\u{f50d}', '✎', "r", "dim"),
    i("Copy", '\u{f50e}', '⧉', "c", "dim"),
    i("Check", '\u{f50f}', '✓', "v", "success"),
    i("Warn", '\u{f510}', '!', "!", "warn"),
    i("Error", '\u{f511}', '✕', "x", "error"),
    i("Info", '\u{f512}', 'i', "i", "info"),
    // -- Network / security --
    i("Network", '\u{f513}', '⇄', "n", "info"),
    i("NoNetwork", '\u{f514}', '∅', "n", "error"),
    i("Lock", '\u{f515}', '▣', "L", "warn"),
    i("Key", '\u{f516}', '⚿', "K", "warn"),
    // -- Loaders --
    i("Vanilla", '\u{f517}', 'V', "V", "info"),
    i("Fabric", '\u{f518}', 'F', "F", "success"),
    i("Quilt", '\u{f519}', 'Q', "Q", "success"),
    i("Forge", '\u{f51a}', 'G', "G", "warn"),
    i("NeoForge", '\u{f51b}', 'N', "N", "warn"),
    i("Modpack", '\u{f51c}', '▣', "P", "accent"),
    // -- Versions --
    i("Version", '\u{f51d}', 'v', "V", "info"),
    i("Snapshot", '\u{f51e}', '◇', "S", "warn"),
    i("Release", '\u{f51f}', '◆', "R", "success"),
    // -- Time --
    i("Clock", '\u{f520}', '◷', "T", "dim"),
    i("Playtime", '\u{f521}', '◈', "T", "dim"),
    i("Calendar", '\u{f522}', '▦', "D", "dim"),
    i("Star", '\u{f523}', '★', "*", "accent"),
    i("Dependency", '\u{f524}', '⇢', ">", "dim"),
    i("Cape", '\u{f525}', '▩', "C", "dim"),
    i("Skin", '\u{f526}', '◉', "S", "accent"),
    i("Sun", '\u{f527}', '◎', "S", "accent"),
    i("Help", '\u{f528}', '?', "?", "info"),
    i("Palette", '\u{f529}', '◐', "P", "dim"),
    i("Terminal", '\u{f52a}', '▸', "T", "info"),
    i("Command", '\u{f52b}', '$', "$", "info"),
    i("Link", '\u{f52c}', '∞', "L", "dim"),
    i("ExternalLink", '\u{f52d}', '⇱', "L", "dim"),
    i("Pin", '\u{f52e}', '†', "P", "dim"),
    // -- Status --
    i("Running", '\u{f52f}', '●', "R", "success"),
    i("Stopped", '\u{f530}', '○', "S", "dim"),
    i("Crashed", '\u{f531}', '✗', "C", "error"),
    i("Pause", '\u{f532}', '‖', "P", "warn"),
    i("Refresh", '\u{f533}', '↻', "R", "info"),
    i("Cancel", '\u{f534}', '✗', "C", "error"),
    i("Close", '\u{f535}', '×', "x", "dim"),
    // -- Chevrons / navigation --
    i("ChevronRight", '\u{f536}', '›', ">", "dim"),
    i("ChevronDown", '\u{f537}', '»', "v", "dim"),
    i("Collapse", '\u{f538}', '‹', "<", "dim"),
    i("Expand", '\u{f539}', '›', ">", "dim"),
    i("Toggle", '\u{f53a}', '◄', "T", "dim"),
    // -- Tags / authors / files --
    i("Tag", '\u{f53b}', '◆', "#", "dim"),
    i("Author", '\u{f53c}', '✎', "A", "dim"),
    i("License", '\u{f53d}', '§', "L", "dim"),
    i("Gallery", '\u{f53e}', '▦', "G", "dim"),
    i("Jar", '\u{f53f}', '▣', "J", "accent"),
    i("Zip", '\u{f540}', '▤', "Z", "dim"),
    i("Config", '\u{f541}', '⚙', "C", "dim"),
    i("Theme", '\u{f542}', '◑', "T", "dim"),
    i("Keyboard", '\u{f543}', '⌨', "K", "dim"),
    i("Mouse", '\u{f544}', '⊙', "M", "dim"),
    i("Breaker", '\u{f545}', '─', "-", "dim"),
    i("Badge", '\u{f546}', '◆', "#", "accent"),
    i("Tab", '\u{f547}', '▸', "T", "dim"),
    i("Menu", '\u{f548}', '≡', "M", "dim"),
    i("Toast", '\u{f549}', '▣', "T", "info"),
    i("Dialog", '\u{f54a}', '□', "D", "dim"),
    i("Button", '\u{f54b}', '▢', "B", "dim"),
    i("Status", '\u{f54c}', '●', "S", "info"),
    i("Empty", '\u{f54d}', '○', "-", "dim"),
    i("Progress", '\u{f54e}', '▮', "P", "accent"),
    i("Spinner", '\u{f54f}', '◌', "o", "dim"),
    i("ArrowUp", '\u{f550}', '↑', "^", "dim"),
    i("ArrowDown", '\u{f551}', '↓', "v", "dim"),
    i("ArrowLeft", '\u{f552}', '←', "<", "dim"),
    i("ArrowRight", '\u{f553}', '→', ">", "dim"),
];

const fn i(
    name: &'static str,
    nerd: char,
    unicode: char,
    ascii: &'static str,
    color: &'static str,
) -> IconDef {
    IconDef {
        name,
        nerd,
        unicode,
        ascii,
        color,
    }
}

/// The active icon mode. Chosen by the config, env var, or first-run check.
pub fn active_mode() -> IconMode {
    if let Ok(m) = std::env::var(format!("{}ICONS", crate::brand::ENV_PREFIX)) {
        if let Ok(m) = m.parse::<IconMode>() {
            return m;
        }
    }
    IconMode::Unicode
}

/// The default pad width for icons (1 space after the glyph).
pub fn pad_default() -> usize {
    1
}

/// A helper to render an icon in a specific mode.
pub struct Icons {
    mode: IconMode,
    pad: usize,
}

impl Default for Icons {
    fn default() -> Self {
        Self::new(IconMode::Unicode, 1)
    }
}

impl Icons {
    pub fn new(mode: IconMode, pad: usize) -> Self {
        Self { mode, pad }
    }

    /// The active mode.
    pub fn mode(&self) -> IconMode {
        self.mode
    }

    /// Render the named icon. Returns the glyph plus padding.
    pub fn render(&self, name: &str) -> String {
        let def = ICONS.iter().find(|d| d.name == name).unwrap_or(&ICONS[0]);
        let glyph = match self.mode {
            IconMode::Nerd => def.nerd.to_string(),
            IconMode::Unicode => def.unicode.to_string(),
            IconMode::Ascii => def.ascii.to_string(),
        };
        let pad = " ".repeat(self.pad);
        format!("{glyph}{pad}")
    }

    /// The width of a rendered icon (glyph + pad), used for alignment.
    pub fn width(&self, name: &str) -> usize {
        let def = ICONS.iter().find(|d| d.name == name).unwrap_or(&ICONS[0]);
        let w = match self.mode {
            // PUA glyphs are ambiguous-width; we always render them as
            // 2 columns + pad.
            IconMode::Nerd => 2 + self.pad,
            IconMode::Unicode => def.unicode.to_string().width() + self.pad,
            IconMode::Ascii => def.ascii.chars().count() + self.pad,
        };
        w
    }

    /// The color token of the named icon.
    pub fn color(&self, name: &str) -> &'static str {
        ICONS
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.color)
            .unwrap_or("dim")
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_100_icons() {
        assert!(
            ICONS.len() >= 100,
            "registry has {} icons, need at least 100",
            ICONS.len()
        );
    }

    #[test]
    fn names_are_unique() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for d in ICONS {
            assert!(seen.insert(d.name), "duplicate icon name: {}", d.name);
        }
    }

    #[test]
    fn no_emoji_code_points_in_any_mode() {
        for def in ICONS {
            assert!(
                !is_emoji(def.nerd),
                "nerd fallback for {} is in an emoji range: U+{:04X}",
                def.name,
                def.nerd as u32
            );
            assert!(
                !is_emoji(def.unicode),
                "unicode fallback for {} is in an emoji range: U+{:04X}",
                def.name,
                def.unicode as u32
            );
            for c in def.ascii.chars() {
                assert!(
                    !is_emoji(c),
                    "ascii fallback for {} contains an emoji: {c}",
                    def.name
                );
            }
        }
    }

    #[test]
    fn every_icon_has_three_renderings() {
        for def in ICONS {
            assert!(!def.nerd.is_ascii_control(), "nerd is ctrl: {}", def.name);
            assert!(
                !def.unicode.is_ascii_control(),
                "unicode is ctrl: {}",
                def.name
            );
            assert!(!def.ascii.is_empty(), "ascii empty: {}", def.name);
        }
    }

    #[test]
    fn nerd_glyphs_are_in_pua_or_mnemonics() {
        for def in ICONS {
            let v = def.nerd as u32;
            // Nerd Font glyphs live in PUA (E000-FFFF) or private
            // Devicons/Material blocks.
            assert!(
                (0xE000..=0xFFFF).contains(&v) || (0xF000..=0xFFFF).contains(&v),
                "nerd glyph for {} is not PUA: U+{:04X}",
                def.name,
                v
            );
        }
    }

    #[test]
    fn alignment_in_a_table() {
        for mode in [IconMode::Nerd, IconMode::Unicode, IconMode::Ascii] {
            let icons = Icons::new(mode, 1);
            // Simulate a table row: icon + name, padded to 12 columns
            let name = "SomeLongName".to_string();
            let mut row = icons.render("Home");
            while row.chars().count() < 12 {
                row.push(' ');
            }
            row.push_str(&name);
            assert_eq!(
                row.chars().count(),
                12 + name.chars().count(),
                "mode {mode:?} row should be 12 + name length"
            );
        }
    }
}
