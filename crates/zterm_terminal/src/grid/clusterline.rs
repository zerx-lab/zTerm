//! ClusteredLine 结构
//!
//! 内存优化的行存储，使用属性聚类压缩相同属性的连续字符
//! 对应 WezTerm 的 ClusteredLine

use serde::{Deserialize, Serialize};
use std::num::NonZeroU8;

use super::cell::Cell;
use super::cluster::Cluster;

/// 固定大小的位集合（简化版，用于标记双宽字符）
///
/// WezTerm 使用 fixedbitset crate，这里使用 Vec<bool> 作为简化实现
/// 但仍保持相同的内存布局目标
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedBitSet {
    bits: Vec<bool>,
}

impl FixedBitSet {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bits: vec![false; capacity],
        }
    }

    pub fn insert(&mut self, index: usize) {
        if index < self.bits.len() {
            self.bits[index] = true;
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        index < self.bits.len() && self.bits[index]
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }
}

/// 内存优化的聚类行存储
///
/// 对应 WezTerm 的 ClusteredLine
/// 将相同属性的连续字符压缩存储，减少内存占用
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusteredLine {
    /// 所有字符的文本内容（UTF-8）
    text: String,
    /// 双宽字符标记（如果存在）
    is_double_wide: Option<Box<FixedBitSet>>,
    /// 属性聚类
    clusters: Vec<Cluster>,
    /// 行长度（单元格数）
    len: u32,
    /// 最后一个单元格的宽度（用于优化）
    last_cell_width: Option<NonZeroU8>,
}

impl ClusteredLine {
    /// 创建空行
    pub fn new() -> Self {
        Self {
            text: String::new(),
            is_double_wide: None,
            clusters: Vec::new(),
            len: 0,
            last_cell_width: None,
        }
    }

    /// 创建指定容量的空行
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            text: String::with_capacity(capacity),
            is_double_wide: None,
            clusters: Vec::with_capacity(capacity / 4), // 假设平均每个聚类 4 个字符
            len: 0,
            last_cell_width: None,
        }
    }

    /// 获取行长度（单元格数）
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// 是否为空行
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 追加单个单元格
    pub fn append_cell(&mut self, cell: &Cell) {
        let cell_width = cell.width() as u16;
        let text = cell.text();

        // 添加文本
        let char_idx = self.text.chars().count(); // 当前字符索引
        self.text.push_str(text);

        // 标记双宽字符位置
        if cell_width == 2 {
            let bitset = self
                .is_double_wide
                .get_or_insert_with(|| Box::new(FixedBitSet::with_capacity(128)));
            bitset.insert(char_idx);
        }

        // 尝试与最后一个聚类合并
        if let Some(last_cluster) = self.clusters.last_mut() {
            if last_cluster.attrs() == cell.attrs() {
                last_cluster.add_width(1); // Cluster.width 表示字符数
                self.len += cell_width as u32; // len 表示屏幕列数
                self.last_cell_width = NonZeroU8::new(cell_width as u8);
                return;
            }
        }

        // 创建新聚类
        self.clusters
            .push(Cluster::new(1, cell.attrs().clone())); // Cluster.width 表示字符数
        self.len += cell_width as u32; // len 表示屏幕列数
        self.last_cell_width = NonZeroU8::new(cell_width as u8);
    }

    /// 从 Vec<Cell> 创建
    pub fn from_cells(cells: &[Cell]) -> Self {
        let mut line = Self::with_capacity(cells.len());
        for cell in cells {
            line.append_cell(cell);
        }
        line
    }

    /// 转换为 Vec<Cell>
    pub fn to_cells(&self) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(self.len as usize);
        let mut char_idx = 0;

        for cluster in &self.clusters {
            let char_count = cluster.width() as usize; // cluster.width 现在表示字符数
            let attrs = cluster.attrs();

            for _ in 0..char_count {
                // 获取第 char_idx 个字符
                if let Some(ch) = self.text.chars().nth(char_idx) {
                    let text = ch.to_string();

                    // 检查是否是双宽字符
                    let width = if self
                        .is_double_wide
                        .as_ref()
                        .map(|bs| bs.contains(char_idx))
                        .unwrap_or(false)
                    {
                        2
                    } else {
                        1
                    };

                    cells.push(Cell::new_with_attrs(&text, attrs.clone(), width as u8));
                    char_idx += 1;
                } else {
                    // 如果文本不足，填充空白单元格
                    cells.push(Cell::blank_with_attrs(attrs.clone()));
                    char_idx += 1;
                }
            }
        }

        cells
    }

    /// 获取文本内容
    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 获取聚类数量
    #[inline]
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    /// 获取聚类引用
    #[inline]
    pub fn clusters(&self) -> &[Cluster] {
        &self.clusters
    }

    /// 估算内存使用（字节）
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.text.capacity()
            + self
                .is_double_wide
                .as_ref()
                .map(|bs| std::mem::size_of_val(bs.as_ref()))
                .unwrap_or(0)
            + self.clusters.capacity() * std::mem::size_of::<Cluster>()
    }
}

impl Default for ClusteredLine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CellAttributes;

    #[test]
    fn test_clustered_line_new() {
        let line = ClusteredLine::new();
        assert!(line.is_empty());
        assert_eq!(line.len(), 0);
        assert_eq!(line.cluster_count(), 0);
    }

    #[test]
    fn test_clustered_line_append_single_cell() {
        let mut line = ClusteredLine::new();
        let cell = Cell::new("a");

        line.append_cell(&cell);
        assert_eq!(line.len(), 1);
        assert_eq!(line.cluster_count(), 1);
        assert_eq!(line.text(), "a");
    }

    #[test]
    fn test_clustered_line_append_same_attrs() {
        let mut line = ClusteredLine::new();
        let cell1 = Cell::new("a");
        let cell2 = Cell::new("b");
        let cell3 = Cell::new("c");

        line.append_cell(&cell1);
        line.append_cell(&cell2);
        line.append_cell(&cell3);

        assert_eq!(line.len(), 3);
        assert_eq!(line.cluster_count(), 1); // 应该合并为一个聚类
        assert_eq!(line.text(), "abc");
    }

    #[test]
    fn test_clustered_line_append_different_attrs() {
        let mut line = ClusteredLine::new();

        let cell1 = Cell::new("a");

        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);
        let cell2 = Cell::new_with_attrs("b", attrs, 1);

        line.append_cell(&cell1);
        line.append_cell(&cell2);

        assert_eq!(line.len(), 2);
        assert_eq!(line.cluster_count(), 2); // 不同属性，两个聚类
        assert_eq!(line.text(), "ab");
    }

    #[test]
    fn test_clustered_line_double_width() {
        let mut line = ClusteredLine::new();
        let cell = Cell::new_with_attrs("中", CellAttributes::default(), 2);

        line.append_cell(&cell);
        assert_eq!(line.len(), 2); // 双宽字符占 2 个单元格
        assert!(line.is_double_wide.is_some());
    }

    #[test]
    fn test_clustered_line_from_cells() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let line = ClusteredLine::from_cells(&cells);

        assert_eq!(line.len(), 3);
        assert_eq!(line.cluster_count(), 1);
        assert_eq!(line.text(), "abc");
    }

    #[test]
    fn test_clustered_line_to_cells() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let line = ClusteredLine::from_cells(&cells);
        let restored = line.to_cells();

        assert_eq!(cells.len(), restored.len());
        for (original, restored) in cells.iter().zip(restored.iter()) {
            assert_eq!(original.text(), restored.text());
            assert_eq!(original.attrs(), restored.attrs());
        }
    }

    #[test]
    fn test_clustered_line_round_trip() {
        let mut cells = Vec::new();

        // 普通字符
        cells.push(Cell::new("a"));
        cells.push(Cell::new("b"));

        // 粗体字符
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);
        cells.push(Cell::new_with_attrs("c", attrs, 1));

        // 双宽字符
        cells.push(Cell::new_with_attrs("中", CellAttributes::default(), 2));

        let line = ClusteredLine::from_cells(&cells);
        let restored = line.to_cells();

        assert_eq!(cells.len(), restored.len());
    }

    #[test]
    fn test_clustered_line_memory_efficiency() {
        // 创建 100 个相同属性的单元格
        let cells: Vec<_> = (0..100).map(|_| Cell::new("x")).collect();
        let line = ClusteredLine::from_cells(&cells);

        // 应该只有一个聚类
        assert_eq!(line.cluster_count(), 1);
        assert_eq!(line.clusters()[0].width(), 100);
    }

    #[test]
    fn test_clustered_line_alternating_attrs() {
        let mut cells = Vec::new();
        let mut attrs1 = CellAttributes::default();
        let mut attrs2 = CellAttributes::default();
        attrs2.set_bold(true);

        for i in 0..10 {
            if i % 2 == 0 {
                cells.push(Cell::new_with_attrs("a", attrs1.clone(), 1));
            } else {
                cells.push(Cell::new_with_attrs("b", attrs2.clone(), 1));
            }
        }

        let line = ClusteredLine::from_cells(&cells);
        assert_eq!(line.cluster_count(), 10); // 交替属性，10 个聚类
    }

    #[test]
    fn test_clustered_line_memory_usage() {
        let line = ClusteredLine::new();
        let usage = line.memory_usage();

        // 应该至少等于结构体大小
        assert!(usage >= std::mem::size_of::<ClusteredLine>());
    }

    #[test]
    fn test_clustered_line_with_capacity() {
        let line = ClusteredLine::with_capacity(100);
        assert!(line.is_empty());
        assert_eq!(line.len(), 0);
    }

    #[test]
    fn test_fixed_bitset() {
        let mut bitset = FixedBitSet::with_capacity(10);
        assert_eq!(bitset.len(), 10);
        assert!(!bitset.contains(0));

        bitset.insert(0);
        bitset.insert(5);
        bitset.insert(9);

        assert!(bitset.contains(0));
        assert!(!bitset.contains(1));
        assert!(bitset.contains(5));
        assert!(bitset.contains(9));
    }

    #[test]
    fn test_clustered_line_mixed_widths() {
        let mut cells = Vec::new();
        cells.push(Cell::new("a")); // 宽度 1
        cells.push(Cell::new_with_attrs("中", CellAttributes::default(), 2)); // 宽度 2
        cells.push(Cell::new("b")); // 宽度 1

        let line = ClusteredLine::from_cells(&cells);
        assert_eq!(line.len(), 4); // 1 + 2 + 1 = 4
    }

    #[test]
    fn test_clustered_line_empty_text() {
        let mut line = ClusteredLine::new();
        let cell = Cell::blank();
        line.append_cell(&cell);

        assert_eq!(line.len(), 1);
        assert_eq!(line.text(), " "); // 空白单元格应该有空格
    }
}
