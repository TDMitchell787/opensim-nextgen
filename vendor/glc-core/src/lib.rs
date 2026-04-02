pub mod types;
pub mod material;
pub mod mesh;
pub mod bounding_box;
pub mod camera;
pub mod scene;
pub mod transform;

// Re-export commonly used types at crate root
pub use types::{EntityId, Color4f, RenderMode, PolygonMode};
pub use material::Material;
pub use mesh::Mesh;
pub use bounding_box::BoundingBox;
pub use camera::Camera;
pub use scene::{World, SceneNode};
pub use transform::Transform;
