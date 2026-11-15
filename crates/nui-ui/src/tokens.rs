//! The design system's values, the same as its tokens: the palette, the spacing scale and the roles that use it,
//! the sizes on a 4 px grid and the four fades. A widget takes its measures from here and never types a number
//! of its own, so a change of the system is a change of this file.
//!
//! A padding in a widget's `Style` counts the box's 1 px frame, which is drawn inside the box: a framed control
//! pads `FRAME + INSET`, a bare one `INSET`.

/// The palette: raw colours with no meaning, as `0xrrggbb`. Greys step evenly in perceived lightness (OKLab L),
/// amber and red sit on the same steps; `grey_0` is white and `grey_1000` black.
pub mod palette {
    pub const GREY_0: u32 = 0xffffff;
    pub const GREY_50: u32 = 0xf5f5f5;
    pub const GREY_100: u32 = 0xebebeb;
    pub const GREY_200: u32 = 0xcecece;
    pub const GREY_300: u32 = 0xb1b1b1;
    pub const GREY_400: u32 = 0x959595;
    pub const GREY_500: u32 = 0x7a7a7a;
    pub const GREY_600: u32 = 0x606060;
    pub const GREY_700: u32 = 0x484848;
    pub const GREY_800: u32 = 0x303030;
    pub const GREY_900: u32 = 0x1b1b1b;
    pub const GREY_950: u32 = 0x0f0f0f;
    pub const GREY_1000: u32 = 0x000000;

    pub const AMBER_50: u32 = 0xfaf4ee;
    pub const AMBER_100: u32 = 0xf8e8d8;
    pub const AMBER_200: u32 = 0xefc596;
    pub const AMBER_300: u32 = 0xe2a356;
    pub const AMBER_400: u32 = 0xcc8305;
    pub const AMBER_500: u32 = 0xa86b07;
    pub const AMBER_600: u32 = 0x865401;
    pub const AMBER_700: u32 = 0x653e00;
    pub const AMBER_800: u32 = 0x452901;
    pub const AMBER_900: u32 = 0x271602;
    pub const AMBER_950: u32 = 0x140e08;

    pub const RED_50: u32 = 0xfef2f0;
    pub const RED_100: u32 = 0xfee5e0;
    pub const RED_200: u32 = 0xfebab0;
    pub const RED_300: u32 = 0xff8b7b;
    pub const RED_400: u32 = 0xf55949;
    pub const RED_500: u32 = 0xdb2e23;
    pub const RED_600: u32 = 0xb70304;
    pub const RED_700: u32 = 0x8b0102;
    pub const RED_800: u32 = 0x5f0503;
    pub const RED_900: u32 = 0x310e0a;
    pub const RED_950: u32 = 0x170c0a;
}

/// Every frame, rule and track.
pub const FRAME: f32 = 1.0;

// The spacing scale: the only values a space may take.
pub const SPACE_2: f32 = 2.0;
pub const SPACE_4: f32 = 4.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_18: f32 = 18.0;
pub const SPACE_24: f32 = 24.0;
pub const SPACE_32: f32 = 32.0;
pub const SPACE_48: f32 = 48.0;
pub const SPACE_64: f32 = 64.0;
pub const SPACE_96: f32 = 96.0;
pub const SPACE_128: f32 = 128.0;
pub const SPACE_192: f32 = 192.0;
pub const SPACE_256: f32 = 256.0;

// Spacing roles: what a space is for. Insets are inside a control, from its frame to its word.
pub const INSET_HAIR: f32 = SPACE_2;
pub const INSET_TIGHT: f32 = SPACE_4;
pub const INSET: f32 = SPACE_8;
pub const INSET_WIDE: f32 = SPACE_12;
pub const GAP_TIGHT: f32 = SPACE_4;
pub const GAP: f32 = SPACE_8;
pub const GAP_WIDE: f32 = SPACE_12;
pub const SECTION: f32 = SPACE_24;
pub const REGION: f32 = SPACE_48;

// Sizes, on a 4 px grid.
pub const SWATCH: f32 = 12.0;
pub const ICON: f32 = 16.0;
pub const CONTROL_COMPACT: f32 = 24.0;
pub const MENU_ROW: f32 = 24.0;
pub const CONTROL: f32 = 28.0;
pub const ICON_SQUARE: f32 = 32.0;
pub const CONTROL_LARGE: f32 = 36.0;
pub const BAR: f32 = 40.0;
pub const CAPTION_W: f32 = 48.0;
/// The widest a tooltip grows, frame included, before its text wraps.
pub const TIP_WIDTH: f32 = 384.0;

// Fades: a colour at this opacity over the ground (`Theme::faded`).
/// A filled control under the pointer.
pub const FADE_LIT: f32 = 0.85;
/// Rows set back while one is being chosen; an absent row's swatch.
pub const FADE_DIM: f32 = 0.5;
/// Every disabled control.
pub const FADE_DISABLED: f32 = 0.4;
/// The light fill under the pointer of a window button or a hover box.
pub const FADE_TINT: f32 = 0.08;
