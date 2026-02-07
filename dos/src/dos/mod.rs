mod common;
mod dos;
pub mod handle;
pub use common::{BColor, DrawCall, Pallete, Pos2, Rect, SysUiMode};
pub use dos::setup;
pub use handle::*;
pub mod tilemap;
