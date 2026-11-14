//! The native world preview. Step 13 builds its shell: the toolbar, the three columns with the real settings
//! sections, and static values where the later steps will bring data (the map is step 14).

pub mod bands;
pub mod settings;
pub mod shell;
pub mod text;

pub use shell::{Columns, PreviewState, ShellIds, build, picture};
