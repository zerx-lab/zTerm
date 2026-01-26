//! Terminal Grid 模块
//!
//! 完整的终端网格实现，参考 WezTerm 设计

pub mod attributes;
pub mod cell;
pub mod cell_attrs;
pub mod cluster;
pub mod clusterline;
pub mod color;
pub mod cursor;
pub mod line;
pub mod linebits;
pub mod screen;
pub mod stable_row_index;
pub mod storage;
pub mod terminal_state;
pub mod zone_range;

// 重新导出常用类型
pub use attributes::CellAttributes;
pub use cell::Cell;
pub use cell_attrs::{CellFlags, Intensity, SemanticType, UnderlineStyle};
pub use cluster::Cluster;
pub use clusterline::ClusteredLine;
pub use color::Color;
pub use cursor::{Cursor, CursorShape};
pub use line::Line;
pub use linebits::LineBits;
pub use screen::Screen;
pub use stable_row_index::{PhysRowIndex, StableRowIndex, VisibleRowIndex};
pub use storage::CellStorage;
pub use terminal_state::{ScrollRegion, TerminalModes, TerminalState};
pub use zone_range::ZoneRange;
