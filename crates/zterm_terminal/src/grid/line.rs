//! Line 结构
//!
//! 终端行的完整实现，支持双存储策略和语义区域
//! 对应 WezTerm 的 Line

use serde::{Deserialize, Serialize};

use super::cell::Cell;
use super::clusterline::ClusteredLine;
use super::linebits::LineBits;
use super::storage::CellStorage;
use super::zone_range::ZoneRange;

/// 终端行
///
/// 对应 WezTerm 的 Line
/// 支持：
/// - 双存储策略（Vec 或 ClusteredLine）
/// - 语义区域（Prompt/Input/Output）
/// - 序列号追踪（用于渲染优化）
/// - 行状态标志（换行、双宽等）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    /// 单元格存储
    cells: CellStorage,
    /// 语义区域
    zones: Vec<ZoneRange>,
    /// 序列号（用于脏检测）
    seqno: u64,
    /// 行标志位
    bits: LineBits,
}

impl Line {
    /// 创建空行
    pub fn new() -> Self {
        Self {
            cells: CellStorage::default(),
            zones: Vec::new(),
            seqno: 0,
            bits: LineBits::default(),
        }
    }

    /// 创建指定长度的空白行
    pub fn with_capacity(len: usize) -> Self {
        Self {
            cells: CellStorage::new_vec_with_capacity(len),
            zones: Vec::new(),
            seqno: 0,
            bits: LineBits::default(),
        }
    }

    /// 从 Vec<Cell> 创建
    pub fn from_cells(cells: Vec<Cell>) -> Self {
        Self {
            cells: CellStorage::new_vec(cells),
            zones: Vec::new(),
            seqno: 0,
            bits: LineBits::default(),
        }
    }

    /// 从 ClusteredLine 创建
    pub fn from_clustered(clustered: ClusteredLine) -> Self {
        Self {
            cells: CellStorage::new_clustered(clustered),
            zones: Vec::new(),
            seqno: 0,
            bits: LineBits::default(),
        }
    }

    /// 获取行长度
    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// 是否为空行
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// 获取单元格（如果是 Clustered 会自动转换为 Vec）
    pub fn get(&self, index: usize) -> Option<&Cell> {
        self.cells.get(index)
    }

    /// 获取可变单元格（会自动转换为 Vec 存储）
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Cell> {
        self.cells.ensure_vec();
        self.increment_seqno();
        self.cells.get_mut(index)
    }

    /// 设置单元格（会自动转换为 Vec 存储）
    pub fn set_cell(&mut self, index: usize, cell: Cell) {
        self.cells.ensure_vec();
        if let Some(existing) = self.cells.get_mut(index) {
            *existing = cell;
            self.increment_seqno();
        }
    }

    /// 追加单元格
    pub fn push(&mut self, cell: Cell) {
        self.cells.push(cell);
        self.increment_seqno();
    }

    /// 追加多个单元格
    pub fn extend<I: IntoIterator<Item = Cell>>(&mut self, cells: I) {
        for cell in cells {
            self.push(cell);
        }
    }

    /// 追加另一行到当前行尾
    pub fn append_line(&mut self, other: &Line) {
        // 先获取偏移量（追加前的长度）
        let offset = self.len();

        let other_cells = other.to_vec();
        self.extend(other_cells);

        // 合并语义区域（需要偏移 other 的区域）
        for zone in &other.zones {
            self.zones.push(zone.offset(offset as isize));
        }

        // 合并行标志
        self.bits.insert(other.bits);
        self.increment_seqno();
    }

    /// 调整行长度
    pub fn resize(&mut self, new_len: usize, fill: Cell) {
        self.cells.resize(new_len, fill);
        self.increment_seqno();
    }

    /// 清空行
    pub fn clear(&mut self) {
        self.cells.clear();
        self.zones.clear();
        self.bits = LineBits::default();
        self.increment_seqno();
    }

    /// 转换为 Vec<Cell>
    pub fn to_vec(&self) -> Vec<Cell> {
        self.cells.to_vec()
    }

    /// 转换为 Clustered 存储
    pub fn into_clustered(mut self) -> Self {
        self.cells = self.cells.into_clustered();
        self
    }

    /// 确保是 Vec 存储
    pub fn ensure_vec(&mut self) {
        self.cells.ensure_vec();
    }

    /// 是否是 Clustered 存储
    #[inline]
    pub fn is_clustered(&self) -> bool {
        self.cells.is_clustered()
    }

    /// 获取序列号
    #[inline]
    pub fn seqno(&self) -> u64 {
        self.seqno
    }

    /// 增加序列号（用于脏检测）
    #[inline]
    pub fn increment_seqno(&mut self) {
        self.seqno = self.seqno.wrapping_add(1);
    }

    /// 设置序列号
    #[inline]
    pub fn set_seqno(&mut self, seqno: u64) {
        self.seqno = seqno;
    }

    /// 获取行标志位
    #[inline]
    pub fn bits(&self) -> LineBits {
        self.bits
    }

    /// 获取可变行标志位
    #[inline]
    pub fn bits_mut(&mut self) -> &mut LineBits {
        &mut self.bits
    }

    /// 设置行标志位
    #[inline]
    pub fn set_bits(&mut self, bits: LineBits) {
        self.bits = bits;
    }

    /// 是否换行
    #[inline]
    pub fn is_wrapped(&self) -> bool {
        self.bits.is_wrapped()
    }

    /// 设置换行标志
    #[inline]
    pub fn set_wrapped(&mut self, wrapped: bool) {
        self.bits.set_wrapped(wrapped);
    }

    /// 是否包含超链接
    #[inline]
    pub fn has_hyperlink(&self) -> bool {
        self.bits.has_hyperlink()
    }

    /// 设置超链接标志
    #[inline]
    pub fn set_has_hyperlink(&mut self, value: bool) {
        self.bits.set_has_hyperlink(value);
    }

    /// 是否是双宽行
    #[inline]
    pub fn is_double_width(&self) -> bool {
        self.bits.is_double_width()
    }

    /// 设置双宽标志
    #[inline]
    pub fn set_double_width(&mut self, value: bool) {
        self.bits.set_double_width(value);
    }

    // ========== 语义区域管理 ==========

    /// 获取语义区域
    #[inline]
    pub fn zones(&self) -> &[ZoneRange] {
        &self.zones
    }

    /// 获取可变语义区域
    #[inline]
    pub fn zones_mut(&mut self) -> &mut Vec<ZoneRange> {
        &mut self.zones
    }

    /// 添加语义区域
    pub fn add_zone(&mut self, zone: ZoneRange) {
        self.zones.push(zone);
        self.increment_seqno();
    }

    /// 设置语义区域
    pub fn set_zones(&mut self, zones: Vec<ZoneRange>) {
        self.zones = zones;
        self.increment_seqno();
    }

    /// 清空语义区域
    pub fn clear_zones(&mut self) {
        self.zones.clear();
        self.increment_seqno();
    }

    /// 查询指定位置的语义类型
    pub fn semantic_type_at(&self, pos: usize) -> Option<super::cell_attrs::SemanticType> {
        self.zones
            .iter()
            .find(|zone| zone.contains(pos))
            .map(|zone| zone.semantic_type)
    }

    /// 查询包含指定位置的语义区域
    pub fn zone_at(&self, pos: usize) -> Option<&ZoneRange> {
        self.zones.iter().find(|zone| zone.contains(pos))
    }

    /// 查询指定范围内的所有语义区域
    pub fn zones_in_range(&self, start: usize, end: usize) -> Vec<&ZoneRange> {
        let range = ZoneRange::new(start, end, super::cell_attrs::SemanticType::Output);
        self.zones
            .iter()
            .filter(|zone| zone.overlaps(&range))
            .collect()
    }

    /// 合并相邻的同类型语义区域
    pub fn consolidate_zones(&mut self) {
        if self.zones.len() <= 1 {
            return;
        }

        let mut consolidated = Vec::new();
        let mut current = self.zones[0];

        for zone in &self.zones[1..] {
            if current.can_merge(zone) {
                current = current.merge(zone);
            } else {
                consolidated.push(current);
                current = *zone;
            }
        }
        consolidated.push(current);

        self.zones = consolidated;
        self.increment_seqno();
    }

    // ========== 工具方法 ==========

    /// 查找双击选择范围（基于分隔符）
    ///
    /// 对应 WezTerm 的 double_click_range
    pub fn double_click_range(&self, pos: usize, delimiters: &str) -> Option<(usize, usize)> {
        if pos >= self.len() {
            return None;
        }

        let cells = self.to_vec();
        let mut start = pos;
        let mut end = pos;

        // 向前查找
        while start > 0 {
            let cell_text = cells[start - 1].text();
            if delimiters.contains(cell_text) || cell_text.trim().is_empty() {
                break;
            }
            start -= 1;
        }

        // 向后查找
        while end < cells.len() {
            let cell_text = cells[end].text();
            if delimiters.contains(cell_text) || cell_text.trim().is_empty() {
                break;
            }
            end += 1;
        }

        if start < end {
            Some((start, end))
        } else {
            None
        }
    }

    /// 估算内存使用（字节）
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.cells.memory_usage()
            + self.zones.capacity() * std::mem::size_of::<ZoneRange>()
    }

    /// 获取行的文本内容
    pub fn text(&self) -> String {
        self.to_vec()
            .iter()
            .map(|cell| cell.text())
            .collect::<String>()
    }

    /// 获取指定范围的文本
    pub fn text_range(&self, start: usize, end: usize) -> String {
        let cells = self.to_vec();
        cells[start..end.min(cells.len())]
            .iter()
            .map(|cell| cell.text())
            .collect()
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::SemanticType;

    #[test]
    fn test_line_new() {
        let line = Line::new();
        assert!(line.is_empty());
        assert_eq!(line.len(), 0);
        assert_eq!(line.seqno(), 0);
    }

    #[test]
    fn test_line_from_cells() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let line = Line::from_cells(cells.clone());

        assert_eq!(line.len(), 3);
        assert!(!line.is_clustered());
    }

    #[test]
    fn test_line_from_clustered() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let clustered = ClusteredLine::from_cells(&cells);
        let line = Line::from_clustered(clustered);

        assert_eq!(line.len(), 2);
        assert!(line.is_clustered());
    }

    #[test]
    fn test_line_push() {
        let mut line = Line::new();
        let initial_seqno = line.seqno();

        line.push(Cell::new("a"));
        assert_eq!(line.len(), 1);
        assert_ne!(line.seqno(), initial_seqno); // seqno 应该增加
    }

    #[test]
    fn test_line_append_line() {
        let mut line1 = Line::from_cells(vec![Cell::new("a"), Cell::new("b")]);
        let line2 = Line::from_cells(vec![Cell::new("c"), Cell::new("d")]);

        line1.append_line(&line2);
        assert_eq!(line1.len(), 4);
        assert_eq!(line1.text(), "abcd");
    }

    #[test]
    fn test_line_append_line_with_zones() {
        let mut line1 = Line::from_cells(vec![Cell::new("a"), Cell::new("b")]);
        line1.add_zone(ZoneRange::prompt(0, 2));

        let mut line2 = Line::from_cells(vec![Cell::new("c"), Cell::new("d")]);
        line2.add_zone(ZoneRange::input(0, 2));

        line1.append_line(&line2);
        assert_eq!(line1.zones().len(), 2);

        // 第二个区域应该偏移
        assert_eq!(line1.zones()[1].start, 2);
        assert_eq!(line1.zones()[1].end, 4);
    }

    #[test]
    fn test_line_resize() {
        let mut line = Line::new();
        line.resize(5, Cell::blank());

        assert_eq!(line.len(), 5);
    }

    #[test]
    fn test_line_clear() {
        let mut line = Line::from_cells(vec![Cell::new("a")]);
        line.add_zone(ZoneRange::prompt(0, 1));
        line.set_wrapped(true);

        line.clear();
        assert!(line.is_empty());
        assert!(line.zones().is_empty());
        assert!(!line.is_wrapped());
    }

    #[test]
    fn test_line_wrapped_flag() {
        let mut line = Line::new();
        assert!(!line.is_wrapped());

        line.set_wrapped(true);
        assert!(line.is_wrapped());
    }

    #[test]
    fn test_line_hyperlink_flag() {
        let mut line = Line::new();
        assert!(!line.has_hyperlink());

        line.set_has_hyperlink(true);
        assert!(line.has_hyperlink());
    }

    #[test]
    fn test_line_seqno_increment() {
        let mut line = Line::new();
        let initial = line.seqno();

        line.increment_seqno();
        assert_eq!(line.seqno(), initial + 1);

        line.increment_seqno();
        assert_eq!(line.seqno(), initial + 2);
    }

    #[test]
    fn test_line_add_zone() {
        let mut line = Line::from_cells(vec![Cell::new("a"); 10]);

        line.add_zone(ZoneRange::prompt(0, 3));
        line.add_zone(ZoneRange::input(3, 7));
        line.add_zone(ZoneRange::output(7, 10));

        assert_eq!(line.zones().len(), 3);
    }

    #[test]
    fn test_line_semantic_type_at() {
        let mut line = Line::from_cells(vec![Cell::new("a"); 10]);
        line.add_zone(ZoneRange::prompt(0, 3));
        line.add_zone(ZoneRange::input(3, 7));

        assert_eq!(line.semantic_type_at(0), Some(SemanticType::Prompt));
        assert_eq!(line.semantic_type_at(5), Some(SemanticType::Input));
        assert_eq!(line.semantic_type_at(9), None);
    }

    #[test]
    fn test_line_zone_at() {
        let mut line = Line::from_cells(vec![Cell::new("a"); 10]);
        let zone = ZoneRange::prompt(2, 5);
        line.add_zone(zone);

        assert!(line.zone_at(1).is_none());
        assert!(line.zone_at(3).is_some());
        assert_eq!(line.zone_at(3).unwrap(), &zone);
    }

    #[test]
    fn test_line_zones_in_range() {
        let mut line = Line::from_cells(vec![Cell::new("a"); 20]);
        line.add_zone(ZoneRange::prompt(0, 5));
        line.add_zone(ZoneRange::input(5, 10));
        line.add_zone(ZoneRange::output(15, 20));

        let zones = line.zones_in_range(3, 12);
        assert_eq!(zones.len(), 2); // prompt 和 input
    }

    #[test]
    fn test_line_consolidate_zones() {
        let mut line = Line::from_cells(vec![Cell::new("a"); 10]);
        line.add_zone(ZoneRange::prompt(0, 3));
        line.add_zone(ZoneRange::prompt(3, 6)); // 相邻且类型相同
        line.add_zone(ZoneRange::input(6, 10));

        line.consolidate_zones();
        assert_eq!(line.zones().len(), 2); // prompt 应该合并

        assert_eq!(line.zones()[0].start, 0);
        assert_eq!(line.zones()[0].end, 6);
        assert_eq!(line.zones()[0].semantic_type, SemanticType::Prompt);
    }

    #[test]
    fn test_line_double_click_range() {
        let cells = vec![
            Cell::new("h"),
            Cell::new("e"),
            Cell::new("l"),
            Cell::new("l"),
            Cell::new("o"),
            Cell::new(" "),
            Cell::new("w"),
            Cell::new("o"),
            Cell::new("r"),
            Cell::new("l"),
            Cell::new("d"),
        ];
        let line = Line::from_cells(cells);

        // 点击 "hello" 中的 'l'
        let range = line.double_click_range(3, " \t");
        assert_eq!(range, Some((0, 5)));

        // 点击 "world" 中的 'o'
        let range = line.double_click_range(7, " \t");
        assert_eq!(range, Some((6, 11)));
    }

    #[test]
    fn test_line_text() {
        let cells = vec![Cell::new("h"), Cell::new("e"), Cell::new("l"), Cell::new("l"), Cell::new("o")];
        let line = Line::from_cells(cells);

        assert_eq!(line.text(), "hello");
    }

    #[test]
    fn test_line_text_range() {
        let cells = vec![Cell::new("h"), Cell::new("e"), Cell::new("l"), Cell::new("l"), Cell::new("o")];
        let line = Line::from_cells(cells);

        assert_eq!(line.text_range(1, 4), "ell");
    }

    #[test]
    fn test_line_memory_usage() {
        let line = Line::from_cells(vec![Cell::new("a"); 100]);
        let usage = line.memory_usage();

        assert!(usage > std::mem::size_of::<Line>());
    }

    #[test]
    fn test_line_into_clustered() {
        let cells = vec![Cell::new("a"); 10];
        let line = Line::from_cells(cells);

        assert!(!line.is_clustered());
        let line = line.into_clustered();
        assert!(line.is_clustered());
    }

    #[test]
    fn test_line_ensure_vec() {
        let cells = vec![Cell::new("a")];
        let clustered = ClusteredLine::from_cells(&cells);
        let mut line = Line::from_clustered(clustered);

        assert!(line.is_clustered());
        line.ensure_vec();
        assert!(!line.is_clustered());
    }

    #[test]
    fn test_line_get_set() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let mut line = Line::from_cells(cells);

        assert_eq!(line.get(1).unwrap().text(), "b");

        line.set_cell(1, Cell::new("x"));
        assert_eq!(line.get(1).unwrap().text(), "x");
    }

    #[test]
    fn test_line_extend() {
        let mut line = Line::new();
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];

        line.extend(cells);
        assert_eq!(line.len(), 3);
        assert_eq!(line.text(), "abc");
    }
}
