//! Colours and fonts, after the Liquid Glass look of quiz-catch
//! (`frontend/src/aesthetics/glass.css`) in its light theme: a warm cream wallpaper,
//! controls and panes as tinted glass floating over it with a lit rim, and Inter
//! throughout (the face quiz-catch uses wherever San Francisco is not installed).

use std::sync::OnceLock;

use bevy::prelude::*;
use bevy::text::FontSource;

const fn hex(rgb: u32) -> Color {
    hexa(rgb, 1.0)
}

const fn hexa(rgb: u32, alpha: f32) -> Color {
    // Exact for 0..=255, so the casts below lose nothing.
    #[expect(clippy::cast_precision_loss, reason = "a byte fits in f32 exactly")]
    const fn byte(value: u32) -> f32 {
        (value & 0xFF) as f32 / 255.0
    }
    Color::srgba(byte(rgb >> 16), byte(rgb >> 8), byte(rgb), alpha)
}

// ------------------------------------------------------------------ the palette
// quiz-catch's light palette (`styles.css`, `prefers-color-scheme: light`), with the
// project's two accent colours: orange `#ff922f` and deep red `#b61b0d`. The discs
// keep their own red and yellow.

/// The wallpaper everything floats over.
pub const BG: Color = hex(0x00f7_f3e8);
pub const RAISED: Color = hex(0x00ef_e9d9);
pub const TEXT: Color = hex(0x0028_2820);
pub const MUTED: Color = hex(0x005f_5b49);
pub const FAINT: Color = hex(0x008f_8a75);
/// Accent one: orange.
pub const ACCENT: Color = hex(0x00ff_922f);
/// Accent two, deep red (`#b61b0d`): where orange runs to in the ember gradient, and
/// accent text on cream (orange is too light to read there).
pub const EMBER: Color = hex(0x00b6_1b0d);
pub const ACCENT_INK: Color = EMBER;

/// The ember gradient: orange at 0, deep red at 1.
pub fn heat(t: f32) -> Color {
    ACCENT.mix(&EMBER, t.clamp(0.0, 1.0))
}
pub const ON_ACCENT: Color = hex(0x0028_2820);
/// "success" and "rain": agent family colours on the slides.
pub const GREEN: Color = hex(0x006f_7a0a);
pub const AQUA: Color = hex(0x002f_6f86);

/// The two disc colours, bright enough to read as red and yellow plastic.
pub const RED: Color = hex(0x00e0_4a2f);
pub const YELLOW: Color = hex(0x00f6_b92b);
/// The same two as text on cream ("danger" and "star").
pub const RED_INK: Color = hex(0x00c8_331f);
pub const YELLOW_INK: Color = hex(0x00b0_7a10);
/// The board: the palette's "rain" blue.
pub const BOARD: Color = hex(0x002f_6f86);
/// The table under the board: a deeper cream.
pub const TABLE: Color = hex(0x00e6_dcc4);

// ------------------------------------------------------------------ glass
// The light theme's materials, thinnest to thickest, and the fills controls sit in.

pub const fn surface(alpha: f32) -> Color {
    hexa(0x00ff_fdf7, alpha)
}
pub const fn ink(alpha: f32) -> Color {
    hexa(0x0028_2820, alpha)
}
pub const fn white(alpha: f32) -> Color {
    Color::srgba(1.0, 1.0, 1.0, alpha)
}

pub const GLASS: Color = surface(0.62);
pub const THICK: Color = surface(0.84);
pub const FILL: Color = ink(0.05);
pub const FILL_2: Color = ink(0.08);
pub const SEP: Color = ink(0.10);
/// The flowing pill behind a selection.
pub const PILL: Color = surface(0.94);
/// Light on the rim: brightest where it enters and leaves (top left, bottom right).
pub const HI_A: Color = white(0.95);
pub const HI_B: Color = white(0.55);
pub const HI_C: Color = white(0.2);
pub const SHADOW: Color = Color::srgba(60.0 / 255.0, 50.0 / 255.0, 20.0 / 255.0, 0.16);

// ------------------------------------------------------------------ fonts

/// Inter in four weights, built into the binary so the app looks the same anywhere.
/// (Bevy's `system_font_discovery` leaked memory on every text change in 0.20, so
/// fonts are handed over as assets instead.)
struct Fonts {
    regular: Handle<Font>,
    medium: Handle<Font>,
    semibold: Handle<Font>,
    bold: Handle<Font>,
}

static FONTS: OnceLock<Fonts> = OnceLock::new();

/// Loads the UI fonts; runs before the UI is built.
pub fn load_fonts(mut fonts: ResMut<Assets<Font>>) {
    let mut load = |bytes: &[u8]| fonts.add(Font::from_bytes(bytes.to_vec()));
    FONTS
        .set(Fonts {
            regular: load(include_bytes!("../fonts/Inter-Regular.ttf")),
            medium: load(include_bytes!("../fonts/Inter-Medium.ttf")),
            semibold: load(include_bytes!("../fonts/Inter-SemiBold.ttf")),
            bold: load(include_bytes!("../fonts/Inter-Bold.ttf")),
        })
        .ok();
}

fn pick(face: fn(&Fonts) -> &Handle<Font>) -> FontSource {
    FONTS.get().map_or_else(FontSource::default, |fonts| FontSource::Handle(face(fonts).clone()))
}

/// Inter Regular: body text.
pub fn ui_font() -> FontSource {
    pick(|fonts| &fonts.regular)
}

/// Inter Medium: buttons and labels.
pub fn medium_font() -> FontSource {
    pick(|fonts| &fonts.medium)
}

/// Inter SemiBold: headlines and values.
pub fn bold_font() -> FontSource {
    pick(|fonts| &fonts.semibold)
}

/// Inter Bold: large titles and the score.
pub fn title_font() -> FontSource {
    pick(|fonts| &fonts.bold)
}
