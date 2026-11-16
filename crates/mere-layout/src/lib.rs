//! Pure layout: no OS code, nothing but `mere-base`'s geometry. A tree of styles and measured leaves goes in,
//! a rect per node comes out. Flexbox (design.md section 3.3, steps 1 to 10), a reduced grid, absolute
//! positioning against the parent, and the layout side of overflow (the automatic minimum and the content
//! size; scrolling itself belongs to the UI crate).
//!
//! Drawing a layout as a picture needs a backend, so it lives outside this crate (`demos/shell-boxes`).

mod absolute;
mod flex;
mod grid;
mod style;
mod tree;

pub use style::{Align, Avail, Dim, Display, Edges, Insets, Justify, Margin, Measured, Overflow, Position, Side, Style, Track, Tracks};
pub use tree::{NodeId, Tree};
