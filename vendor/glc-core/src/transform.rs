use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Affine transform wrapper around `glam::Mat4`.
///
/// Provides convenience constructors and decomposition methods
/// for the scene graph transform hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub matrix: Mat4,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        matrix: Mat4::IDENTITY,
    };

    pub fn new(matrix: Mat4) -> Self {
        Self { matrix }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            matrix: Mat4::from_translation(translation),
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            matrix: Mat4::from_quat(rotation),
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            matrix: Mat4::from_scale(scale),
        }
    }

    pub fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            matrix: Mat4::from_scale_rotation_translation(scale, rotation, translation),
        }
    }

    /// Multiply two transforms: `self` applied after `other`.
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            matrix: other.matrix * self.matrix,
        }
    }

    /// Transform a point.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix.transform_point3(point)
    }

    /// Transform a direction vector (ignores translation).
    pub fn transform_direction(&self, dir: Vec3) -> Vec3 {
        self.matrix.transform_vector3(dir)
    }

    /// Inverse of this transform.
    pub fn inverse(&self) -> Self {
        Self {
            matrix: self.matrix.inverse(),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Transform {
        Transform {
            matrix: self.matrix * rhs.matrix,
        }
    }
}
