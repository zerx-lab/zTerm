//! 语义区域范围
//!
//! 用于标记行中的语义区域（Prompt/Input/Output）
//! 对应 WezTerm 的 SemanticZone

use serde::{Deserialize, Serialize};

use super::cell_attrs::SemanticType;

/// 语义区域范围
///
/// 对应 WezTerm 的 SemanticZone
/// 标记行中某个范围的语义类型（用于 Shell Integration）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneRange {
    /// 起始位置（包含）
    pub start: usize,
    /// 结束位置（不包含）
    pub end: usize,
    /// 语义类型
    pub semantic_type: SemanticType,
}

impl ZoneRange {
    /// 创建新的语义区域
    ///
    /// # Panics
    ///
    /// 如果 start > end 会 panic
    pub fn new(start: usize, end: usize, semantic_type: SemanticType) -> Self {
        assert!(
            start <= end,
            "ZoneRange start ({}) must be <= end ({})",
            start,
            end
        );
        Self {
            start,
            end,
            semantic_type,
        }
    }

    /// 创建 Prompt 区域
    pub fn prompt(start: usize, end: usize) -> Self {
        Self::new(start, end, SemanticType::Prompt)
    }

    /// 创建 Input 区域
    pub fn input(start: usize, end: usize) -> Self {
        Self::new(start, end, SemanticType::Input)
    }

    /// 创建 Output 区域
    pub fn output(start: usize, end: usize) -> Self {
        Self::new(start, end, SemanticType::Output)
    }

    /// 获取区域长度
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// 是否为空区域
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// 是否包含指定位置
    #[inline]
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    /// 是否与另一个区域重叠
    #[inline]
    pub fn overlaps(&self, other: &ZoneRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// 是否与另一个区域相邻
    #[inline]
    pub fn adjacent_to(&self, other: &ZoneRange) -> bool {
        self.end == other.start || other.end == self.start
    }

    /// 是否可以与另一个区域合并（相邻且类型相同）
    #[inline]
    pub fn can_merge(&self, other: &ZoneRange) -> bool {
        self.semantic_type == other.semantic_type && self.adjacent_to(other)
    }

    /// 合并两个区域（必须相邻且类型相同）
    ///
    /// # Panics
    ///
    /// 如果两个区域不能合并会 panic
    pub fn merge(&self, other: &ZoneRange) -> Self {
        assert!(
            self.can_merge(other),
            "Cannot merge zones: not adjacent or different types"
        );

        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self::new(start, end, self.semantic_type)
    }

    /// 计算与另一个区域的交集
    pub fn intersection(&self, other: &ZoneRange) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }

        let start = self.start.max(other.start);
        let end = self.end.min(other.end);

        // 交集使用第一个区域的语义类型
        Some(Self::new(start, end, self.semantic_type))
    }

    /// 是否完全包含另一个区域
    #[inline]
    pub fn contains_range(&self, other: &ZoneRange) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// 偏移区域位置
    pub fn offset(&self, offset: isize) -> Self {
        let start = (self.start as isize + offset) as usize;
        let end = (self.end as isize + offset) as usize;
        Self::new(start, end, self.semantic_type)
    }

    /// 裁剪区域到指定范围
    pub fn clamp(&self, min: usize, max: usize) -> Option<Self> {
        let start = self.start.max(min).min(max);
        let end = self.end.max(min).min(max);

        if start >= end {
            None
        } else {
            Some(Self::new(start, end, self.semantic_type))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_range_new() {
        let zone = ZoneRange::new(0, 10, SemanticType::Prompt);
        assert_eq!(zone.start, 0);
        assert_eq!(zone.end, 10);
        assert_eq!(zone.semantic_type, SemanticType::Prompt);
    }

    #[test]
    #[should_panic(expected = "must be <= end")]
    fn test_zone_range_invalid() {
        ZoneRange::new(10, 5, SemanticType::Prompt);
    }

    #[test]
    fn test_zone_range_helpers() {
        let prompt = ZoneRange::prompt(0, 5);
        assert_eq!(prompt.semantic_type, SemanticType::Prompt);

        let input = ZoneRange::input(5, 10);
        assert_eq!(input.semantic_type, SemanticType::Input);

        let output = ZoneRange::output(10, 20);
        assert_eq!(output.semantic_type, SemanticType::Output);
    }

    #[test]
    fn test_zone_range_len() {
        let zone = ZoneRange::new(5, 15, SemanticType::Input);
        assert_eq!(zone.len(), 10);
    }

    #[test]
    fn test_zone_range_is_empty() {
        let empty = ZoneRange::new(5, 5, SemanticType::Output);
        assert!(empty.is_empty());

        let non_empty = ZoneRange::new(5, 10, SemanticType::Output);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_zone_range_contains() {
        let zone = ZoneRange::new(5, 15, SemanticType::Input);

        assert!(!zone.contains(4));
        assert!(zone.contains(5));
        assert!(zone.contains(10));
        assert!(zone.contains(14));
        assert!(!zone.contains(15));
    }

    #[test]
    fn test_zone_range_overlaps() {
        let zone1 = ZoneRange::new(5, 15, SemanticType::Prompt);
        let zone2 = ZoneRange::new(10, 20, SemanticType::Input);
        let zone3 = ZoneRange::new(20, 30, SemanticType::Output);

        assert!(zone1.overlaps(&zone2));
        assert!(zone2.overlaps(&zone1));
        assert!(!zone1.overlaps(&zone3));
        assert!(!zone2.overlaps(&zone3)); // 边界相邻不算重叠
    }

    #[test]
    fn test_zone_range_adjacent() {
        let zone1 = ZoneRange::new(0, 5, SemanticType::Prompt);
        let zone2 = ZoneRange::new(5, 10, SemanticType::Input);
        let zone3 = ZoneRange::new(15, 20, SemanticType::Output);

        assert!(zone1.adjacent_to(&zone2));
        assert!(zone2.adjacent_to(&zone1));
        assert!(!zone1.adjacent_to(&zone3));
    }

    #[test]
    fn test_zone_range_can_merge() {
        let zone1 = ZoneRange::prompt(0, 5);
        let zone2 = ZoneRange::prompt(5, 10);
        let zone3 = ZoneRange::input(10, 15);

        assert!(zone1.can_merge(&zone2)); // 相邻且类型相同
        assert!(!zone2.can_merge(&zone3)); // 相邻但类型不同
    }

    #[test]
    fn test_zone_range_merge() {
        let zone1 = ZoneRange::prompt(0, 5);
        let zone2 = ZoneRange::prompt(5, 10);

        let merged = zone1.merge(&zone2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 10);
        assert_eq!(merged.semantic_type, SemanticType::Prompt);
    }

    #[test]
    #[should_panic(expected = "Cannot merge zones")]
    fn test_zone_range_merge_invalid() {
        let zone1 = ZoneRange::prompt(0, 5);
        let zone2 = ZoneRange::input(5, 10);

        zone1.merge(&zone2); // 类型不同，应该 panic
    }

    #[test]
    fn test_zone_range_intersection() {
        let zone1 = ZoneRange::new(5, 15, SemanticType::Prompt);
        let zone2 = ZoneRange::new(10, 20, SemanticType::Input);

        let inter = zone1.intersection(&zone2).unwrap();
        assert_eq!(inter.start, 10);
        assert_eq!(inter.end, 15);
        assert_eq!(inter.semantic_type, SemanticType::Prompt);
    }

    #[test]
    fn test_zone_range_intersection_none() {
        let zone1 = ZoneRange::new(0, 10, SemanticType::Prompt);
        let zone2 = ZoneRange::new(15, 20, SemanticType::Input);

        assert!(zone1.intersection(&zone2).is_none());
    }

    #[test]
    fn test_zone_range_contains_range() {
        let zone1 = ZoneRange::new(0, 20, SemanticType::Output);
        let zone2 = ZoneRange::new(5, 15, SemanticType::Input);
        let zone3 = ZoneRange::new(10, 25, SemanticType::Prompt);

        assert!(zone1.contains_range(&zone2));
        assert!(!zone1.contains_range(&zone3));
    }

    #[test]
    fn test_zone_range_offset() {
        let zone = ZoneRange::new(10, 20, SemanticType::Input);

        let offset_plus = zone.offset(5);
        assert_eq!(offset_plus.start, 15);
        assert_eq!(offset_plus.end, 25);

        let offset_minus = zone.offset(-5);
        assert_eq!(offset_minus.start, 5);
        assert_eq!(offset_minus.end, 15);
    }

    #[test]
    fn test_zone_range_clamp() {
        let zone = ZoneRange::new(5, 25, SemanticType::Output);

        let clamped = zone.clamp(0, 30).unwrap();
        assert_eq!(clamped.start, 5);
        assert_eq!(clamped.end, 25);

        let clamped2 = zone.clamp(10, 20).unwrap();
        assert_eq!(clamped2.start, 10);
        assert_eq!(clamped2.end, 20);

        let clamped3 = zone.clamp(30, 40);
        assert!(clamped3.is_none()); // 完全在范围外
    }

    #[test]
    fn test_zone_range_equality() {
        let zone1 = ZoneRange::prompt(0, 10);
        let zone2 = ZoneRange::prompt(0, 10);
        let zone3 = ZoneRange::input(0, 10);

        assert_eq!(zone1, zone2);
        assert_ne!(zone1, zone3);
    }

    #[test]
    fn test_zone_range_clone() {
        let zone = ZoneRange::input(5, 15);
        let cloned = zone.clone();

        assert_eq!(zone, cloned);
    }
}
