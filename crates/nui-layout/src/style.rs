//! What a page's CSS says about a box, as plain Rust values. Nothing here parses CSS: `calc()`, `min()`,
//! `ch` and `cq` units are arithmetic done by the caller before it builds a `Style`.

use nui_base::Vec2;

/// A length along one axis. `Pct` is a percentage (100.0 is the whole), taken of the parent's inner size
/// and treated as `Auto` while that size is not known.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Dim {
    #[default]
    Auto,
    Px(f32),
    Pct(f32),
}

impl Dim {
    /// The length in pixels, or `None` for `Auto` and for a percentage of an unknown size.
    pub fn resolve(self, basis: Option<f32>) -> Option<f32> {
        match self {
            Dim::Auto => None,
            Dim::Px(v) => Some(v),
            Dim::Pct(p) => basis.map(|b| b * p / 100.0),
        }
    }
}

/// `Block` is only what the page needs of it: a column of full-width boxes, with no gap.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Display {
    #[default]
    Flex,
    Block,
    None,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Edges {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Edges {
    pub const fn all(v: f32) -> Edges {
        Edges { left: v, top: v, right: v, bottom: v }
    }
    /// CSS order of the two-value form: vertical first, then horizontal.
    pub const fn vh(v: f32, h: f32) -> Edges {
        Edges { left: h, top: v, right: h, bottom: v }
    }
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Edges {
        Edges { left, top, right, bottom }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Side {
    Px(f32),
    Auto,
}

impl Side {
    /// What the margin takes in the sums before auto margins share the free space out.
    pub fn px(self) -> f32 {
        match self {
            Side::Px(v) => v,
            Side::Auto => 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Margin {
    pub left: Side,
    pub top: Side,
    pub right: Side,
    pub bottom: Side,
}

impl Default for Margin {
    fn default() -> Margin {
        Margin::all(0.0)
    }
}

impl Margin {
    pub const fn all(v: f32) -> Margin {
        Margin { left: Side::Px(v), top: Side::Px(v), right: Side::Px(v), bottom: Side::Px(v) }
    }
    /// `margin: 0 auto`: centred on the cross axis of a column, or on the main axis of a row.
    pub const fn horizontal_auto() -> Margin {
        Margin { left: Side::Auto, top: Side::Px(0.0), right: Side::Auto, bottom: Side::Px(0.0) }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    #[default]
    Stretch,
    /// Only in a row; in a column it behaves as `Start`, as in the CSS spec.
    Baseline,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
}

/// The flex subset of design.md section 1.2. `grow`, `shrink` and `basis` are the parts of `flex:`; the
/// defaults are CSS's own (`flex: 0 1 auto`, `align-items: stretch`, row direction, no wrap).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub display: Display,
    /// `flex-direction: column`.
    pub column: bool,
    pub wrap: bool,
    /// (column gap, row gap): the first lies between boxes side by side, the second between boxes stacked.
    pub gap: Vec2,
    pub grow: f32,
    pub shrink: f32,
    pub basis: Dim,
    pub order: i32,
    pub w: Dim,
    pub h: Dim,
    /// `Auto` is the automatic minimum of flex items (the content's min size); `Px(0.0)` switches it off,
    /// which is what the page's `min-width: 0` does.
    pub min_w: Dim,
    pub min_h: Dim,
    /// `Auto` means no maximum.
    pub max_w: Dim,
    pub max_h: Dim,
    /// width / height, filling the axis that is not set.
    pub aspect: Option<f32>,
    pub pad: Edges,
    pub margin: Margin,
    pub align_items: Align,
    pub align_self: Option<Align>,
    pub justify: Justify,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            display: Display::Flex,
            column: false,
            wrap: false,
            gap: Vec2::new(0.0, 0.0),
            grow: 0.0,
            shrink: 1.0,
            basis: Dim::Auto,
            order: 0,
            w: Dim::Auto,
            h: Dim::Auto,
            min_w: Dim::Auto,
            min_h: Dim::Auto,
            max_w: Dim::Auto,
            max_h: Dim::Auto,
            aspect: None,
            pad: Edges::default(),
            margin: Margin::default(),
            align_items: Align::Stretch,
            align_self: None,
            justify: Justify::Start,
        }
    }
}

/// What a leaf is told about the room it may take. `None` is no limit on that axis; `Some(0.0)` asks for the
/// narrowest the leaf can be (a text's longest word), which is how the automatic minimum is found.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Avail {
    pub w: Option<f32>,
    pub h: Option<f32>,
}

/// A leaf's answer. `baseline` is the distance from the top of the box to the first line's baseline.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Measured {
    pub size: Vec2,
    pub baseline: f32,
}
