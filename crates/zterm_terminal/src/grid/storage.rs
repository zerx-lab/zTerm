//! Cell 存储策略
//!
//! 提供两种存储方式：Vec 和 ClusteredLine
//! 对应 WezTerm 的 CellStorage

use serde::{Deserialize, Serialize};

use super::cell::Cell;
use super::clusterline::ClusteredLine;

/// Vec 存储（简单存储）
///
/// 直接使用 Vec<Cell> 存储，适合频繁修改的行
pub type VecStorage = Vec<Cell>;

/// Cell 存储策略枚举
///
/// 对应 WezTerm 的 CellStorage
/// 提供两种存储方式：
/// - V: Vec<Cell> - 简单直接，适合频繁修改
/// - C: ClusteredLine - 内存优化，适合只读或少量修改
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellStorage {
    /// Vec 存储
    V(VecStorage),
    /// Clustered 存储
    C(ClusteredLine),
}

impl CellStorage {
    /// 创建 Vec 存储
    pub fn new_vec(cells: Vec<Cell>) -> Self {
        Self::V(cells)
    }

    /// 创建空的 Vec 存储
    pub fn new_vec_empty() -> Self {
        Self::V(Vec::new())
    }

    /// 创建指定容量的 Vec 存储
    pub fn new_vec_with_capacity(capacity: usize) -> Self {
        Self::V(Vec::with_capacity(capacity))
    }

    /// 创建 Clustered 存储
    pub fn new_clustered(line: ClusteredLine) -> Self {
        Self::C(line)
    }

    /// 从 Vec<Cell> 创建 Clustered 存储
    pub fn new_clustered_from_cells(cells: &[Cell]) -> Self {
        Self::C(ClusteredLine::from_cells(cells))
    }

    /// 获取行长度
    pub fn len(&self) -> usize {
        match self {
            Self::V(vec) => vec.len(),
            Self::C(clustered) => clustered.len(),
        }
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 转换为 Vec<Cell>
    pub fn to_vec(&self) -> Vec<Cell> {
        match self {
            Self::V(vec) => vec.clone(),
            Self::C(clustered) => clustered.to_cells(),
        }
    }

    /// 获取指定位置的单元格引用（仅适用于 Vec 存储）
    pub fn get(&self, index: usize) -> Option<&Cell> {
        match self {
            Self::V(vec) => vec.get(index),
            Self::C(_) => None, // Clustered 存储不支持直接索引
        }
    }

    /// 获取指定位置的可变单元格引用（仅适用于 Vec 存储）
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Cell> {
        match self {
            Self::V(vec) => vec.get_mut(index),
            Self::C(_) => None,
        }
    }

    /// 转换为 Vec 存储（如果是 Clustered 则转换）
    pub fn into_vec(self) -> Self {
        match self {
            Self::V(_) => self,
            Self::C(clustered) => Self::V(clustered.to_cells()),
        }
    }

    /// 转换为 Clustered 存储（如果是 Vec 则转换）
    pub fn into_clustered(self) -> Self {
        match self {
            Self::V(vec) => Self::C(ClusteredLine::from_cells(&vec)),
            Self::C(_) => self,
        }
    }

    /// 确保是 Vec 存储（可变访问前调用）
    pub fn ensure_vec(&mut self) {
        if let Self::C(clustered) = self {
            let cells = clustered.to_cells();
            *self = Self::V(cells);
        }
    }

    /// 是否是 Vec 存储
    pub fn is_vec(&self) -> bool {
        matches!(self, Self::V(_))
    }

    /// 是否是 Clustered 存储
    pub fn is_clustered(&self) -> bool {
        matches!(self, Self::C(_))
    }

    /// 估算内存使用（字节）
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::V(vec) => {
                std::mem::size_of::<Vec<Cell>>() + vec.capacity() * std::mem::size_of::<Cell>()
            }
            Self::C(clustered) => clustered.memory_usage(),
        }
    }

    /// 追加单元格（会自动转换为 Vec 存储）
    pub fn push(&mut self, cell: Cell) {
        self.ensure_vec();
        if let Self::V(vec) = self {
            vec.push(cell);
        }
    }

    /// 清空存储
    pub fn clear(&mut self) {
        match self {
            Self::V(vec) => vec.clear(),
            Self::C(clustered) => *clustered = ClusteredLine::new(),
        }
    }

    /// 调整大小（会自动转换为 Vec 存储）
    pub fn resize(&mut self, new_len: usize, value: Cell) {
        self.ensure_vec();
        if let Self::V(vec) = self {
            vec.resize(new_len, value);
        }
    }
}

impl Default for CellStorage {
    fn default() -> Self {
        Self::new_vec_empty()
    }
}

impl From<Vec<Cell>> for CellStorage {
    fn from(vec: Vec<Cell>) -> Self {
        Self::V(vec)
    }
}

impl From<ClusteredLine> for CellStorage {
    fn from(clustered: ClusteredLine) -> Self {
        Self::C(clustered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CellAttributes;

    #[test]
    fn test_storage_new_vec() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let storage = CellStorage::new_vec(cells.clone());

        assert!(storage.is_vec());
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_storage_new_vec_empty() {
        let storage = CellStorage::new_vec_empty();
        assert!(storage.is_vec());
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_new_clustered() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let storage = CellStorage::new_clustered_from_cells(&cells);

        assert!(storage.is_clustered());
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_storage_to_vec() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let storage = CellStorage::new_clustered_from_cells(&cells);
        let result = storage.to_vec();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_storage_get_vec() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let storage = CellStorage::new_vec(cells);

        assert!(storage.get(0).is_some());
        assert_eq!(storage.get(0).unwrap().text(), "a");
        assert!(storage.get(2).is_none());
    }

    #[test]
    fn test_storage_get_clustered() {
        let cells = vec![Cell::new("a")];
        let storage = CellStorage::new_clustered_from_cells(&cells);

        // Clustered 存储不支持直接索引
        assert!(storage.get(0).is_none());
    }

    #[test]
    fn test_storage_get_mut() {
        let cells = vec![Cell::new("a")];
        let mut storage = CellStorage::new_vec(cells);

        if let Some(cell) = storage.get_mut(0) {
            let mut attrs = CellAttributes::default();
            attrs.set_bold(true);
            *cell = Cell::new_with_attrs("b", attrs, 1);
        }

        assert_eq!(storage.get(0).unwrap().text(), "b");
    }

    #[test]
    fn test_storage_into_vec() {
        let cells = vec![Cell::new("a")];
        let storage = CellStorage::new_clustered_from_cells(&cells);

        let storage = storage.into_vec();
        assert!(storage.is_vec());
        assert!(storage.get(0).is_some());
    }

    #[test]
    fn test_storage_into_clustered() {
        let cells = vec![Cell::new("a")];
        let storage = CellStorage::new_vec(cells);

        let storage = storage.into_clustered();
        assert!(storage.is_clustered());
    }

    #[test]
    fn test_storage_ensure_vec() {
        let cells = vec![Cell::new("a")];
        let mut storage = CellStorage::new_clustered_from_cells(&cells);

        assert!(storage.is_clustered());
        storage.ensure_vec();
        assert!(storage.is_vec());
        assert!(storage.get(0).is_some());
    }

    #[test]
    fn test_storage_push() {
        let mut storage = CellStorage::new_vec_empty();
        storage.push(Cell::new("a"));
        storage.push(Cell::new("b"));

        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_storage_push_clustered() {
        let cells = vec![Cell::new("a")];
        let mut storage = CellStorage::new_clustered_from_cells(&cells);

        // push 会自动转换为 Vec
        storage.push(Cell::new("b"));
        assert!(storage.is_vec());
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_storage_clear() {
        let cells = vec![Cell::new("a"), Cell::new("b")];
        let mut storage = CellStorage::new_vec(cells);

        storage.clear();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_resize() {
        let mut storage = CellStorage::new_vec_empty();
        storage.resize(5, Cell::blank());

        assert_eq!(storage.len(), 5);
    }

    #[test]
    fn test_storage_memory_usage() {
        let cells = vec![Cell::new("a"); 100];
        let vec_storage = CellStorage::new_vec(cells.clone());
        let clustered_storage = CellStorage::new_clustered_from_cells(&cells);

        let vec_usage = vec_storage.memory_usage();
        let clustered_usage = clustered_storage.memory_usage();

        // 都应该有合理的内存使用量
        assert!(vec_usage > 0);
        assert!(clustered_usage > 0);
    }

    #[test]
    fn test_storage_from_vec() {
        let cells = vec![Cell::new("a")];
        let storage: CellStorage = cells.into();
        assert!(storage.is_vec());
    }

    #[test]
    fn test_storage_from_clustered() {
        let clustered = ClusteredLine::new();
        let storage: CellStorage = clustered.into();
        assert!(storage.is_clustered());
    }

    #[test]
    fn test_storage_default() {
        let storage = CellStorage::default();
        assert!(storage.is_vec());
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_round_trip() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let storage = CellStorage::new_vec(cells.clone());

        // Vec -> Clustered -> Vec
        let storage = storage.into_clustered();
        let storage = storage.into_vec();

        let result = storage.to_vec();
        assert_eq!(result.len(), cells.len());
    }
}
