//! 聚类（Cluster）结构
//!
//! 用于 ClusteredLine 的属性聚类，将相同属性的连续字符压缩存储

use serde::{Deserialize, Serialize};

use super::attributes::CellAttributes;

/// 属性聚类
///
/// 表示一段具有相同属性的连续字符
/// 对应 WezTerm 的 Cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cluster {
    /// 该聚类占据的单元格宽度
    pub cell_width: u16,
    /// 该聚类的属性
    pub attrs: CellAttributes,
}

impl Cluster {
    /// 创建新的聚类
    #[inline]
    pub fn new(cell_width: u16, attrs: CellAttributes) -> Self {
        Self { cell_width, attrs }
    }

    /// 获取单元格宽度
    #[inline]
    pub fn width(&self) -> u16 {
        self.cell_width
    }

    /// 获取属性引用
    #[inline]
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    /// 获取可变属性引用
    #[inline]
    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }

    /// 设置属性
    #[inline]
    pub fn set_attrs(&mut self, attrs: CellAttributes) {
        self.attrs = attrs;
    }

    /// 增加宽度
    #[inline]
    pub fn add_width(&mut self, width: u16) {
        self.cell_width += width;
    }

    /// 是否可以与另一个聚类合并（属性相同）
    #[inline]
    pub fn can_merge(&self, other: &Cluster) -> bool {
        self.attrs == other.attrs
    }

    /// 合并另一个聚类（属性必须相同）
    ///
    /// # Panics
    ///
    /// 如果属性不同会 panic
    #[inline]
    pub fn merge(&mut self, other: &Cluster) {
        assert_eq!(
            self.attrs, other.attrs,
            "Cannot merge clusters with different attributes"
        );
        self.cell_width += other.cell_width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Color, Intensity, SemanticType, UnderlineStyle};

    #[test]
    fn test_cluster_new() {
        let attrs = CellAttributes::default();
        let cluster = Cluster::new(5, attrs.clone());
        assert_eq!(cluster.width(), 5);
        assert_eq!(cluster.attrs(), &attrs);
    }

    #[test]
    fn test_cluster_width() {
        let cluster = Cluster::new(3, CellAttributes::default());
        assert_eq!(cluster.cell_width, 3);
        assert_eq!(cluster.width(), 3);
    }

    #[test]
    fn test_cluster_attrs_access() {
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);
        let mut cluster = Cluster::new(1, attrs.clone());

        assert_eq!(cluster.attrs(), &attrs);
        assert!(cluster.attrs().is_bold());

        cluster.attrs_mut().set_italic(true);
        assert!(cluster.attrs().italic());
    }

    #[test]
    fn test_cluster_set_attrs() {
        let mut cluster = Cluster::new(2, CellAttributes::default());

        let mut new_attrs = CellAttributes::default();
        new_attrs.set_foreground(Color::Rgb { r: 255, g: 0, b: 0 });
        cluster.set_attrs(new_attrs.clone());

        assert_eq!(cluster.attrs(), &new_attrs);
    }

    #[test]
    fn test_cluster_add_width() {
        let mut cluster = Cluster::new(3, CellAttributes::default());
        cluster.add_width(2);
        assert_eq!(cluster.width(), 5);

        cluster.add_width(10);
        assert_eq!(cluster.width(), 15);
    }

    #[test]
    fn test_cluster_can_merge_same_attrs() {
        let attrs = CellAttributes::default();
        let cluster1 = Cluster::new(3, attrs.clone());
        let cluster2 = Cluster::new(2, attrs);

        assert!(cluster1.can_merge(&cluster2));
    }

    #[test]
    fn test_cluster_can_merge_different_attrs() {
        let mut attrs1 = CellAttributes::default();
        attrs1.set_bold(true);

        let mut attrs2 = CellAttributes::default();
        attrs2.set_italic(true);

        let cluster1 = Cluster::new(3, attrs1);
        let cluster2 = Cluster::new(2, attrs2);

        assert!(!cluster1.can_merge(&cluster2));
    }

    #[test]
    fn test_cluster_merge_same_attrs() {
        let attrs = CellAttributes::default();
        let mut cluster1 = Cluster::new(3, attrs.clone());
        let cluster2 = Cluster::new(2, attrs);

        cluster1.merge(&cluster2);
        assert_eq!(cluster1.width(), 5);
    }

    #[test]
    #[should_panic(expected = "Cannot merge clusters with different attributes")]
    fn test_cluster_merge_different_attrs_panics() {
        let mut attrs1 = CellAttributes::default();
        attrs1.set_bold(true);

        let mut attrs2 = CellAttributes::default();
        attrs2.set_italic(true);

        let mut cluster1 = Cluster::new(3, attrs1);
        let cluster2 = Cluster::new(2, attrs2);

        cluster1.merge(&cluster2); // Should panic
    }

    #[test]
    fn test_cluster_equality() {
        let attrs = CellAttributes::default();
        let cluster1 = Cluster::new(3, attrs.clone());
        let cluster2 = Cluster::new(3, attrs.clone());
        let cluster3 = Cluster::new(2, attrs);

        assert_eq!(cluster1, cluster2);
        assert_ne!(cluster1, cluster3); // Different width
    }

    #[test]
    fn test_cluster_with_complex_attrs() {
        let mut attrs = CellAttributes::default();
        attrs.set_intensity(Intensity::Bold);
        attrs.set_underline(UnderlineStyle::Curly);
        attrs.set_foreground(Color::Indexed(12));
        attrs.set_background(Color::Rgb {
            r: 30,
            g: 30,
            b: 30,
        });
        attrs.set_semantic_type(SemanticType::Prompt);

        let cluster = Cluster::new(7, attrs.clone());
        assert_eq!(cluster.attrs(), &attrs);
        assert_eq!(cluster.width(), 7);
    }

    #[test]
    fn test_cluster_multiple_merges() {
        let attrs = CellAttributes::default();
        let mut cluster1 = Cluster::new(1, attrs.clone());
        let cluster2 = Cluster::new(2, attrs.clone());
        let cluster3 = Cluster::new(3, attrs);

        cluster1.merge(&cluster2);
        assert_eq!(cluster1.width(), 3);

        cluster1.merge(&cluster3);
        assert_eq!(cluster1.width(), 6);
    }

    #[test]
    fn test_cluster_clone() {
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);
        let cluster = Cluster::new(5, attrs);

        let cloned = cluster.clone();
        assert_eq!(cluster, cloned);
        assert_eq!(cloned.width(), 5);
        assert!(cloned.attrs().is_bold());
    }
}
