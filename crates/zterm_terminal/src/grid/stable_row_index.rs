//! 稳定行索引（StableRowIndex）
//!
//! 对应 WezTerm 的 StableRowIndex
//! 提供跨 scrollback 的稳定行索引

use serde::{Deserialize, Serialize};

/// 稳定行索引
///
/// 对应 WezTerm 的 StableRowIndex
/// 即使 scrollback 滚动，StableRowIndex 也保持不变
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StableRowIndex(pub usize);

impl StableRowIndex {
    /// 创建新的稳定索引
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// 获取索引值
    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }

    /// 下一个索引
    #[inline]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// 上一个索引（如果 > 0）
    #[inline]
    pub const fn prev(self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }

    /// 偏移索引
    #[inline]
    pub const fn offset(self, delta: isize) -> Self {
        Self((self.0 as isize + delta) as usize)
    }

    /// 尝试偏移索引（如果结果 < 0 返回 None）
    #[inline]
    pub fn checked_offset(self, delta: isize) -> Option<Self> {
        let result = self.0 as isize + delta;
        if result >= 0 {
            Some(Self(result as usize))
        } else {
            None
        }
    }
}

impl From<usize> for StableRowIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<StableRowIndex> for usize {
    fn from(index: StableRowIndex) -> usize {
        index.0
    }
}

impl std::fmt::Display for StableRowIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StableRowIndex({})", self.0)
    }
}

/// 物理行索引
///
/// 对应 WezTerm 的 PhysRowIndex
/// 表示在 VecDeque 中的实际位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhysRowIndex(pub usize);

impl PhysRowIndex {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for PhysRowIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<PhysRowIndex> for usize {
    fn from(index: PhysRowIndex) -> usize {
        index.0
    }
}

/// 可见行索引
///
/// 对应 WezTerm 的 VisibleRowIndex
/// 表示在当前视口中的位置（0 = 第一行可见行）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VisibleRowIndex(pub isize);

impl VisibleRowIndex {
    #[inline]
    pub const fn new(index: isize) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn get(self) -> isize {
        self.0
    }

    /// 是否在视口内（0..rows）
    #[inline]
    pub fn is_visible(self, rows: usize) -> bool {
        self.0 >= 0 && self.0 < rows as isize
    }
}

impl From<isize> for VisibleRowIndex {
    fn from(index: isize) -> Self {
        Self(index)
    }
}

impl From<VisibleRowIndex> for isize {
    fn from(index: VisibleRowIndex) -> isize {
        index.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_row_index() {
        let idx = StableRowIndex::new(10);
        assert_eq!(idx.get(), 10);

        let next = idx.next();
        assert_eq!(next.get(), 11);

        let prev = idx.prev();
        assert_eq!(prev.unwrap().get(), 9);

        let zero = StableRowIndex::new(0);
        assert!(zero.prev().is_none());
    }

    #[test]
    fn test_stable_row_index_offset() {
        let idx = StableRowIndex::new(10);

        let plus = idx.offset(5);
        assert_eq!(plus.get(), 15);

        let minus = idx.offset(-3);
        assert_eq!(minus.get(), 7);
    }

    #[test]
    fn test_stable_row_index_checked_offset() {
        let idx = StableRowIndex::new(5);

        assert_eq!(idx.checked_offset(3).unwrap().get(), 8);
        assert_eq!(idx.checked_offset(-2).unwrap().get(), 3);
        assert!(idx.checked_offset(-10).is_none());
    }

    #[test]
    fn test_stable_row_index_ordering() {
        let idx1 = StableRowIndex::new(5);
        let idx2 = StableRowIndex::new(10);

        assert!(idx1 < idx2);
        assert!(idx2 > idx1);
        assert_eq!(idx1, idx1);
    }

    #[test]
    fn test_phys_row_index() {
        let idx = PhysRowIndex::new(42);
        assert_eq!(idx.get(), 42);

        let from_usize: PhysRowIndex = 100.into();
        assert_eq!(from_usize.get(), 100);
    }

    #[test]
    fn test_visible_row_index() {
        let idx = VisibleRowIndex::new(5);
        assert_eq!(idx.get(), 5);

        assert!(idx.is_visible(10)); // 5 在 [0, 10) 内
        assert!(!idx.is_visible(3)); // 5 不在 [0, 3) 内

        let negative = VisibleRowIndex::new(-1);
        assert!(!negative.is_visible(10)); // -1 不可见
    }

    #[test]
    fn test_visible_row_index_boundary() {
        let rows = 24;

        let first = VisibleRowIndex::new(0);
        assert!(first.is_visible(rows));

        let last = VisibleRowIndex::new(23);
        assert!(last.is_visible(rows));

        let out_of_bounds = VisibleRowIndex::new(24);
        assert!(!out_of_bounds.is_visible(rows));
    }

    #[test]
    fn test_display() {
        let idx = StableRowIndex::new(42);
        assert_eq!(format!("{}", idx), "StableRowIndex(42)");
    }
}
