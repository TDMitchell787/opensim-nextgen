use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Default field of view angle in degrees (matches original 35.0).
pub const DEFAULT_FOV_DEGREES: f64 = 35.0;

/// Camera definition matching GLC_Camera.
///
/// Uses eye/target/up representation. The default up vector is Z-axis
/// (matching GLC_lib's `glc::Z_AXIS` convention for CAD models).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Default up vector for the coordinate system (Z-up for CAD).
    pub default_up: Vec3,
    /// Field of view in degrees.
    pub fov_degrees: f64,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Self {
            eye,
            target,
            up,
            default_up: Vec3::Z,
            fov_degrees: DEFAULT_FOV_DEGREES,
        }
    }

    /// Compute the view (look-at) matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    /// Compute the projection matrix for a given aspect ratio and near/far planes.
    pub fn projection_matrix(&self, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective_rh(
            (self.fov_degrees as f32).to_radians(),
            aspect_ratio,
            near,
            far,
        )
    }

    /// Distance from eye to target.
    pub fn distance(&self) -> f32 {
        (self.eye - self.target).length()
    }

    /// Forward direction (normalized, from eye toward target).
    pub fn forward(&self) -> Vec3 {
        (self.target - self.eye).normalize()
    }

    /// Right direction (normalized).
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }

    // --- Predefined views (matching original OpenglView slots) ---

    /// Isometric view 1: looking from (+X, +Y, +Z) quadrant.
    pub fn iso_view_1(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        let dir = Vec3::new(1.0, 1.0, 1.0).normalize();
        Self::new(center + dir * dist, center, Vec3::Z)
    }

    /// Isometric view 2: looking from (-X, +Y, +Z) quadrant.
    pub fn iso_view_2(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        let dir = Vec3::new(-1.0, 1.0, 1.0).normalize();
        Self::new(center + dir * dist, center, Vec3::Z)
    }

    /// Isometric view 3: looking from (-X, -Y, +Z) quadrant.
    pub fn iso_view_3(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        let dir = Vec3::new(-1.0, -1.0, 1.0).normalize();
        Self::new(center + dir * dist, center, Vec3::Z)
    }

    /// Isometric view 4: looking from (+X, -Y, +Z) quadrant.
    pub fn iso_view_4(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        let dir = Vec3::new(1.0, -1.0, 1.0).normalize();
        Self::new(center + dir * dist, center, Vec3::Z)
    }

    /// Front view: looking along -Y axis (Z up).
    pub fn front_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(0.0, dist, 0.0), center, Vec3::Z)
    }

    /// Right view: looking along -X axis (Z up).
    pub fn right_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(dist, 0.0, 0.0), center, Vec3::Z)
    }

    /// Top view: looking along -Z axis (Y up).
    pub fn top_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(0.0, 0.0, dist), center, Vec3::Y)
    }

    /// Rear view: looking along +Y axis (Z up).
    pub fn rear_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(0.0, -dist, 0.0), center, Vec3::Z)
    }

    /// Left view: looking along +X axis (Z up).
    pub fn left_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(-dist, 0.0, 0.0), center, Vec3::Z)
    }

    /// Bottom view: looking along +Z axis (negative Y up).
    pub fn bottom_view(center: Vec3, radius: f32) -> Self {
        let dist = radius * 3.0;
        Self::new(center + Vec3::new(0.0, 0.0, -dist), center, Vec3::NEG_Y)
    }
}

impl Default for Camera {
    fn default() -> Self {
        // Default: isometric view of a unit sphere at origin.
        Self::iso_view_1(Vec3::ZERO, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_matrix_not_degenerate() {
        let cam = Camera::default();
        let view = cam.view_matrix();
        // View matrix should be invertible (non-zero determinant).
        assert!(view.determinant().abs() > 1e-6);
    }

    #[test]
    fn test_front_view_looks_along_neg_y() {
        let cam = Camera::front_view(Vec3::ZERO, 1.0);
        let forward = cam.forward();
        // Forward should point roughly along -Y.
        assert!(forward.y < -0.99);
        assert!(forward.x.abs() < 0.01);
        assert!(forward.z.abs() < 0.01);
    }

    #[test]
    fn test_right_view_looks_along_neg_x() {
        let cam = Camera::right_view(Vec3::ZERO, 1.0);
        let forward = cam.forward();
        assert!(forward.x < -0.99);
    }

    #[test]
    fn test_top_view_looks_along_neg_z() {
        let cam = Camera::top_view(Vec3::ZERO, 1.0);
        let forward = cam.forward();
        assert!(forward.z < -0.99);
    }

    #[test]
    fn test_iso_view_1_direction() {
        let cam = Camera::iso_view_1(Vec3::ZERO, 1.0);
        let forward = cam.forward();
        // Iso 1 looks from (+,+,+) toward origin → forward is (-,-,-)
        assert!(forward.x < 0.0);
        assert!(forward.y < 0.0);
        assert!(forward.z < 0.0);
    }

    #[test]
    fn test_distance() {
        let cam = Camera::new(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        assert!((cam.distance() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_predefined_views_have_correct_distance() {
        let center = Vec3::new(1.0, 2.0, 3.0);
        let radius = 5.0;
        let views = [
            Camera::iso_view_1(center, radius),
            Camera::iso_view_2(center, radius),
            Camera::iso_view_3(center, radius),
            Camera::iso_view_4(center, radius),
            Camera::front_view(center, radius),
            Camera::right_view(center, radius),
            Camera::top_view(center, radius),
        ];
        for cam in &views {
            let dist = cam.distance();
            assert!(
                (dist - radius * 3.0).abs() < 0.01,
                "Expected distance {}, got {}",
                radius * 3.0,
                dist
            );
        }
    }

    #[test]
    fn test_projection_matrix_not_degenerate() {
        let cam = Camera::default();
        let proj = cam.projection_matrix(1.5, 0.1, 1000.0);
        assert!(proj.determinant().abs() > 1e-10);
    }
}
