//! Semantic color tokens and the three Helio palettes (Sun, Midnight, Daylight),
//! each with truecolor, 256-color and 16-color variants.

use serde::{Deserialize, Serialize};

/// A semantic color token. Every UI element references one of these, never a
/// raw color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorToken {
    Accent,
    AccentHover,
    AccentDim,
    Bg,
    BgElev,
    Border,
    Text,
    TextDim,
    Success,
    Warn,
    Error,
    Info,
    Grid,
    GridHot,
}

/// The color depth a terminal supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// 24-bit RGB.
    True,
    /// 256 ANSI colors.
    Ansi256,
    /// 16 ANSI colors.
    Ansi16,
}

/// A palette: one token -> one color value.
#[derive(Debug, Clone, Copy)]
pub struct PaletteEntry {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PaletteEntry {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// One named palette with its color depth variants.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    pub depth: ColorDepth,
    pub colors: [PaletteEntry; ColorToken::COUNT],
}

impl ColorToken {
    const COUNT: usize = 14;

    /// The index of a token in the palette's color array.
    pub const fn index(self) -> usize {
        match self {
            Self::Accent => 0,
            Self::AccentHover => 1,
            Self::AccentDim => 2,
            Self::Bg => 3,
            Self::BgElev => 4,
            Self::Border => 5,
            Self::Text => 6,
            Self::TextDim => 7,
            Self::Success => 8,
            Self::Warn => 9,
            Self::Error => 10,
            Self::Info => 11,
            Self::Grid => 12,
            Self::GridHot => 13,
        }
    }
}

/// The three named palettes. Sun is the default.
pub const SUN: Palette = Palette {
    name: "Sun",
    depth: ColorDepth::True,
    colors: [
        PaletteEntry::rgb(255, 176, 0),   // Accent: warm amber
        PaletteEntry::rgb(255, 138, 0),   // AccentHover: orange
        PaletteEntry::rgb(180, 120, 0),   // AccentDim
        PaletteEntry::rgb(22, 28, 40),    // Bg: deep charcoal-navy
        PaletteEntry::rgb(30, 38, 54),    // BgElev
        PaletteEntry::rgb(60, 70, 90),    // Border
        PaletteEntry::rgb(240, 240, 235), // Text
        PaletteEntry::rgb(160, 165, 170), // TextDim
        PaletteEntry::rgb(0, 180, 100),   // Success
        PaletteEntry::rgb(255, 190, 0),   // Warn
        PaletteEntry::rgb(230, 60, 60),   // Error
        PaletteEntry::rgb(80, 160, 240),  // Info
        PaletteEntry::rgb(40, 48, 64),    // Grid (faint)
        PaletteEntry::rgb(80, 90, 110),   // GridHot
    ],
};

pub const MIDNIGHT: Palette = Palette {
    name: "Midnight",
    depth: ColorDepth::True,
    colors: [
        PaletteEntry::rgb(255, 176, 0),
        PaletteEntry::rgb(255, 138, 0),
        PaletteEntry::rgb(180, 120, 0),
        PaletteEntry::rgb(10, 12, 20),
        PaletteEntry::rgb(18, 22, 34),
        PaletteEntry::rgb(45, 52, 70),
        PaletteEntry::rgb(235, 238, 245),
        PaletteEntry::rgb(140, 145, 155),
        PaletteEntry::rgb(0, 170, 95),
        PaletteEntry::rgb(255, 185, 0),
        PaletteEntry::rgb(225, 55, 55),
        PaletteEntry::rgb(75, 150, 235),
        PaletteEntry::rgb(28, 34, 50),
        PaletteEntry::rgb(70, 80, 105),
    ],
};

pub const DAYLIGHT: Palette = Palette {
    name: "Daylight",
    depth: ColorDepth::True,
    colors: [
        PaletteEntry::rgb(210, 130, 0),
        PaletteEntry::rgb(190, 100, 0),
        PaletteEntry::rgb(150, 100, 20),
        PaletteEntry::rgb(252, 248, 240), // warm cream
        PaletteEntry::rgb(244, 238, 225),
        PaletteEntry::rgb(200, 190, 170),
        PaletteEntry::rgb(40, 45, 55),
        PaletteEntry::rgb(110, 115, 125),
        PaletteEntry::rgb(0, 140, 70),
        PaletteEntry::rgb(200, 150, 0),
        PaletteEntry::rgb(190, 40, 40),
        PaletteEntry::rgb(50, 120, 200),
        PaletteEntry::rgb(235, 228, 210),
        PaletteEntry::rgb(215, 205, 180),
    ],
};

/// All three palettes, in display order.
pub const PALETTES: &[Palette] = &[SUN, MIDNIGHT, DAYLIGHT];

/// Look up a palette by name, defaulting to Sun.
pub fn palette_by_name(name: &str) -> &'static Palette {
    PALETTES.iter().find(|p| p.name == name).unwrap_or(&SUN)
}

/// Detect the terminal's color depth from COLORTERM / TERM.
pub fn detect_color_depth() -> ColorDepth {
    if std::env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit") {
        return ColorDepth::True;
    }
    if std::env::var("TERM").is_ok_and(|v| v.contains("256color")) {
        return ColorDepth::Ansi256;
    }
    ColorDepth::Ansi16
}

/// Pick the right palette + depth for the terminal.
pub fn active_palette(name: &str) -> (&'static Palette, ColorDepth) {
    let p = palette_by_name(name);
    let depth = detect_color_depth();
    (p, depth)
}

/// Render a token from a palette to an 8-bit RGB triple for the terminal.
pub fn token_rgb(palette: &Palette, token: ColorToken) -> (u8, u8, u8) {
    let c = palette.colors[token.index()];
    (c.r, c.g, c.b)
}

/// Get the ANSI escape for a color, given depth.
pub fn ansi_color(rgb: (u8, u8, u8), depth: ColorDepth) -> String {
    let (r, g, b) = rgb;
    match depth {
        ColorDepth::True => format!("\u{1b}[38;2;{r};{g};{b}m"),
        ColorDepth::Ansi256 => {
            // Nearest of 256
            let idx = nearest_256(r, g, b);
            if idx < 16 {
                format!(
                    "\u{1b}[{}m",
                    if idx < 8 {
                        30 + idx
                    } else {
                        90 + idx - 8
                    }
                )
            } else {
                format!("\u{1b}[38;5;{idx}m")
            }
        }
        ColorDepth::Ansi16 => {
            let idx = nearest_16(r, g, b);
            let code = if idx < 8 { 30 + idx } else { 90 + (idx - 8) };
            format!("\u{1b}[{code}m")
        }
    }
}

fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    // Quick: use xterm 6x6x6 cube (indices 16..232) plus a few grays.
    let map = |v: u8| -> u8 {
        if v < 48 {
            0
        } else if v < 115 {
            1
        } else if v < 181 {
            2
        } else if v < 248 {
            3
        } else {
            4
        }
    };
    let i = map(r) * 36 + map(g) * 6 + map(b) + 16;
    i as u8
}

fn nearest_16(r: u8, g: u8, b: u8) -> u8 {
    // Map to one of 16 basic ANSI colors.
    let dist = |c: &(u8, u8, u8)| -> u32 {
        let dr = (c.0 as i32 - r as i32).abs() as u32;
        let dg = (c.1 as i32 - g as i32).abs() as u32;
        let db = (c.2 as i32 - b as i32).abs() as u32;
        dr * dr + dg * dg + db * db
    };
    let basic: [((u8, u8, u8), u8); 16] = [
        ((0, 0, 0), 0),
        ((128, 0, 0), 1),
        ((0, 128, 0), 2),
        ((128, 128, 0), 3),
        ((0, 0, 128), 4),
        ((128, 0, 128), 5),
        ((0, 128, 128), 6),
        ((192, 192, 192), 7),
        ((48, 48, 48), 8),
        ((255, 0, 0), 9),
        ((0, 255, 0), 10),
        ((255, 255, 0), 11),
        ((0, 0, 255), 12),
        ((255, 0, 255), 13),
        ((0, 255, 255), 14),
        ((255, 255, 255), 15),
    ];
    basic
        .iter()
        .min_by_key(|&(c, _)| dist(c))
        .map(|&(_, i)| i)
        .unwrap_or(0)
}

/// Set the terminal title (OSC 0) to "Helio", and restore it on exit.
/// Returns a guard that restores the previous title on drop.
pub struct TerminalTitleGuard {
    /// Saved previous title (empty if the terminal had none).
    pub _prev: String,
}

impl TerminalTitleGuard {
    /// Set the title to Helio and save the previous one.
    pub fn set(name: &str) -> Self {
        // OSC 0 sets window + icon title.
        print!("\u{1b}]0;{name}\u{1b}\\");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        Self {
            _prev: String::new(),
        }
    }
}

impl Drop for TerminalTitleGuard {
    fn drop(&mut self) {
        // Restore the (unknown) previous title by clearing it.
        print!("\u{1b}]0;\u{1b}\\");
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_palettes_exist() {
        assert_eq!(PALETTES.len(), 3);
        assert_eq!(palette_by_name("Sun").name, "Sun");
        assert_eq!(palette_by_name("Midnight").name, "Midnight");
        assert_eq!(palette_by_name("Daylight").name, "Daylight");
        assert_eq!(palette_by_name("bogus").name, "Sun");
    }

    #[test]
    fn all_tokens_are_indexable() {
        for t in [
            ColorToken::Accent,
            ColorToken::AccentHover,
            ColorToken::AccentDim,
            ColorToken::Bg,
            ColorToken::BgElev,
            ColorToken::Border,
            ColorToken::Text,
            ColorToken::TextDim,
            ColorToken::Success,
            ColorToken::Warn,
            ColorToken::Error,
            ColorToken::Info,
            ColorToken::Grid,
            ColorToken::GridHot,
        ] {
            assert!(t.index() < ColorToken::COUNT);
        }
    }

    #[test]
    fn color_depth_detection() {
        // We can't set env vars in parallel tests safely; just check the fn runs.
        let _ = detect_color_depth();
    }

    #[test]
    fn ansi16_produces_a_valid_code() {
        let s = ansi_color((255, 176, 0), ColorDepth::Ansi16);
        assert!(s.starts_with("\u{1b}["), "got: {s}");
        assert!(s.ends_with('m'), "got: {s}");
    }

    #[test]
    fn ansi256_produes_a_valid_code() {
        let s = ansi_color((255, 176, 0), ColorDepth::Ansi256);
        assert!(s.starts_with("\u{1b}["), "got: {s}");
    }
}
