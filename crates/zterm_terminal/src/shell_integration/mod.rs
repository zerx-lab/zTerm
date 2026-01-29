//! Shell Integration 支持
//!
//! 实现 OSC 133/633 shell integration 协议

mod block;
mod json_types;
mod scanner;

pub use block::{BlockId, BlockManager, BlockState, CommandBlock, OutputBlock, OutputStream};
pub use json_types::{BlockMetadata, CommandMetadata, CommandResult, JsonDataType, OutputMetadata};
pub use scanner::{OscScanner, OscSequence};
