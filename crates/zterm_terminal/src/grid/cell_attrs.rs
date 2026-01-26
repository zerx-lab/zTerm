//! Cell 属性类型
//!
//! 参考 WezTerm 的 CellAttributes 设计，使用位字段优化

use serde::{Deserialize, Serialize};

/// 语义类型（用于 Shell Integration）
///
/// 参考 WezTerm 的 SemanticType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SemanticType {
    /// 输出（默认）
    Output = 0,
    /// 输入
    Input = 1,
    /// 提示符
    Prompt = 2,
}

impl Default for SemanticType {
    fn default() -> Self {
        Self::Output
    }
}

impl SemanticType {
    /// 从 u8 转换（用于位字段）
    pub const fn from_u8(value: u8) -> Self {
        match value & 0b11 {
            1 => Self::Input,
            2 => Self::Prompt,
            _ => Self::Output,
        }
    }

    /// 转换为 u8（用于位字段）
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Cell 标志位
///
/// 参考 WezTerm 的属性位字段设计
/// 使用 bitflags 宏定义标志位
use bitflags::bitflags;

bitflags! {
    /// Cell 文本属性标志
    ///
    /// 对应 VT100/ANSI SGR 序列
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct CellFlags: u16 {
        /// 粗体（SGR 1）
        const BOLD = 1 << 0;

        /// 斜体（SGR 3）
        const ITALIC = 1 << 1;

        /// 下划线（SGR 4）
        const UNDERLINE = 1 << 2;

        /// 反色（SGR 7）
        const REVERSE = 1 << 3;

        /// 删除线（SGR 9）
        const STRIKETHROUGH = 1 << 4;

        /// 闪烁（SGR 5）
        const BLINK = 1 << 5;

        /// 隐藏（SGR 8）
        const INVISIBLE = 1 << 6;

        /// 上划线（SGR 53）
        const OVERLINE = 1 << 7;

        /// 双下划线（SGR 21）
        const DOUBLE_UNDERLINE = 1 << 8;

        /// 暗淡（SGR 2）
        const DIM = 1 << 9;

        /// 快速闪烁（SGR 6）
        const RAPID_BLINK = 1 << 10;

        /// 行尾被包装（内部使用）
        const WRAPPED = 1 << 11;
    }
}

impl Default for CellFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// Cell 强度（用于 SGR 1/2/22）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Intensity {
    /// 正常强度
    Normal = 0,
    /// 粗体/高亮（SGR 1）
    Bold = 1,
    /// 暗淡（SGR 2）
    Dim = 2,
}

impl Default for Intensity {
    fn default() -> Self {
        Self::Normal
    }
}

impl Intensity {
    pub const fn from_u8(value: u8) -> Self {
        match value & 0b11 {
            1 => Self::Bold,
            2 => Self::Dim,
            _ => Self::Normal,
        }
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

/// 下划线类型（SGR 4/21/24）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum UnderlineStyle {
    /// 无下划线
    None = 0,
    /// 单下划线（SGR 4）
    Single = 1,
    /// 双下划线（SGR 21）
    Double = 2,
    /// 波浪下划线（SGR 4:3）
    Curly = 3,
    /// 点状下划线（SGR 4:4）
    Dotted = 4,
    /// 虚线下划线（SGR 4:5）
    Dashed = 5,
}

impl Default for UnderlineStyle {
    fn default() -> Self {
        Self::None
    }
}

impl UnderlineStyle {
    pub const fn from_u8(value: u8) -> Self {
        match value & 0b111 {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            _ => Self::None,
        }
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_type() {
        assert_eq!(SemanticType::default(), SemanticType::Output);
        assert_eq!(SemanticType::Output.to_u8(), 0);
        assert_eq!(SemanticType::Input.to_u8(), 1);
        assert_eq!(SemanticType::Prompt.to_u8(), 2);

        assert_eq!(SemanticType::from_u8(0), SemanticType::Output);
        assert_eq!(SemanticType::from_u8(1), SemanticType::Input);
        assert_eq!(SemanticType::from_u8(2), SemanticType::Prompt);
    }

    #[test]
    fn test_cell_flags() {
        let mut flags = CellFlags::empty();
        assert!(!flags.contains(CellFlags::BOLD));

        flags.insert(CellFlags::BOLD);
        assert!(flags.contains(CellFlags::BOLD));

        flags.insert(CellFlags::ITALIC);
        assert!(flags.contains(CellFlags::BOLD | CellFlags::ITALIC));

        flags.remove(CellFlags::BOLD);
        assert!(!flags.contains(CellFlags::BOLD));
        assert!(flags.contains(CellFlags::ITALIC));
    }

    #[test]
    fn test_intensity() {
        assert_eq!(Intensity::default(), Intensity::Normal);
        assert_eq!(Intensity::Bold.to_u8(), 1);
        assert_eq!(Intensity::from_u8(2), Intensity::Dim);
    }

    #[test]
    fn test_underline_style() {
        assert_eq!(UnderlineStyle::default(), UnderlineStyle::None);
        assert_eq!(UnderlineStyle::Curly.to_u8(), 3);
        assert_eq!(UnderlineStyle::from_u8(4), UnderlineStyle::Dotted);
    }
}
