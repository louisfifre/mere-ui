//! The native world preview. Step 13 builds its shell: the toolbar, the three columns with the real settings
//! sections, and static values where the later steps will bring data (the map is step 14).

pub mod app;
pub mod bands;
pub mod colour;
pub mod cut;
pub mod figures;
pub mod json;
pub mod map;
pub mod paint;
pub mod run;
pub mod scene;
pub mod section;
pub mod settings;
pub mod shell;
pub mod text;
pub mod world;

pub use app::PreviewApp;
pub use shell::{Columns, PreviewState, ShellIds, build, picture};
