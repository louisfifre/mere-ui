//! Pure layout: no OS code, nothing but `nui-base`'s geometry. A tree of styles and measured leaves goes in,
//! a rect per node comes out. Flexbox only for now (design.md section 3.3, flex steps 1 to 10); grid,
//! absolute positioning and overflow are build step 8.
//!
//! Drawing a layout as a picture needs a backend, so it lives outside this crate (`demos/shell-boxes`).

mod flex;
mod style;
mod tree;

pub use style::{Align, Avail, Dim, Display, Edges, Justify, Margin, Measured, Side, Style};
pub use tree::{NodeId, Tree};
