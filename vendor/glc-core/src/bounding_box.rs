use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box, matching GLC_BoundingBox.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    /// An empty (inverted) bounding box that will expand on first `combine`.
    pub const EMPTY: Self = Self {
        min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create a bounding box from a set of points.
    pub fn from_points(points: &[Vec3]) -> Self {
        let mut bb = Self::EMPTY;
        for &p in points {
            bb.min = bb.min.min(p);
            bb.max = bb.max.max(p);
        }
        bb
    }

    /// Combine two bounding boxes (union).
    pub fn combine(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Expand this bounding box to include a point.
    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Center point of the bounding box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Size (extents) of the bounding box along each axis.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Radius of the bounding sphere (half-diagonal).
    pub fn radius(&self) -> f32 {
        self.size().length() * 0.5
    }

    /// Return all 8 corner points of this bounding box.
    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    /// Returns true if this bounding box has never been expanded
    /// (i.e., it's still in the EMPTY state).
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Return a new bounding box expanded by `margin` in all directions.
    pub fn expanded_by(&self, margin: f32) -> BoundingBox {
        BoundingBox {
            min: self.min - Vec3::splat(margin),
            max: self.max + Vec3::splat(margin),
        }
    }

    /// Returns true if this bounding box overlaps `other` (inclusive of touching edges).
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_bounding_box() {
        let bb = BoundingBox::EMPTY;
        assert!(bb.is_empty());
    }

    #[test]
    fn test_from_points() {
        let points = vec![
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let bb = BoundingBox::from_points(&points);
        assert_eq!(bb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bb.max, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_center() {
        let bb = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(bb.center(), Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_size() {
        let bb = BoundingBox::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(bb.size(), Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_radius() {
        let bb = BoundingBox::new(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0));
        assert!((bb.radius() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_combine() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let b = BoundingBox::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(0.5, 0.5, 0.5));
        let c = a.combine(&b);
        assert_eq!(c.min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(c.max, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_corners() {
        let bb = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 2.0, 3.0));
        let corners = bb.corners();
        assert_eq!(corners.len(), 8);
        assert_eq!(corners[0], Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(corners[7], Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_expand() {
        let mut bb = BoundingBox::EMPTY;
        bb.expand(Vec3::new(1.0, 2.0, 3.0));
        bb.expand(Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bb.max, Vec3::new(1.0, 2.0, 3.0));
        assert!(!bb.is_empty());
    }

    #[test]
    fn test_expanded_by() {
        let bb = BoundingBox::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));
        let expanded = bb.expanded_by(0.5);
        assert_eq!(expanded.min, Vec3::new(0.5, 1.5, 2.5));
        assert_eq!(expanded.max, Vec3::new(4.5, 5.5, 6.5));
    }

    #[test]
    fn test_intersects_overlap() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        let b = BoundingBox::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 3.0, 3.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn test_intersects_touching() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let b = BoundingBox::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 1.0));
        assert!(a.intersects(&b)); // touching edges count as intersecting
    }

    #[test]
    fn test_intersects_separated() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let b = BoundingBox::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn test_intersects_one_axis_separated() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        // Overlaps on X and Y but separated on Z
        let b = BoundingBox::new(Vec3::new(0.5, 0.5, 5.0), Vec3::new(1.5, 1.5, 6.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_expanded_then_intersects() {
        let a = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let b = BoundingBox::new(Vec3::new(1.5, 0.0, 0.0), Vec3::new(2.5, 1.0, 1.0));
        // Not intersecting at natural size
        assert!(!a.intersects(&b));
        // But intersecting after expanding by 0.5
        assert!(a.expanded_by(0.5).intersects(&b));
    }
}
