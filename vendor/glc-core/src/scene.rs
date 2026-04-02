use crate::bounding_box::BoundingBox;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::transform::Transform;
use crate::types::{EntityId, RenderMode};
use serde::{Deserialize, Serialize};

/// Index into the `World::nodes` arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeIndex(pub usize);

/// A node in the scene graph, matching GLC_StructOccurrence.
///
/// Uses index-based children (no raw pointers) for safe, serializable scene graphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: EntityId,
    pub name: String,
    /// Local transform relative to parent.
    pub transform: Transform,
    /// Mesh index into `World::meshes`, if this node has geometry.
    pub mesh_index: Option<usize>,
    /// Child node indices into `World::nodes`.
    pub children: Vec<NodeIndex>,
    /// Parent node index (None for root).
    pub parent: Option<NodeIndex>,
    /// Whether this node is visible.
    pub visible: bool,
    /// Shader/shading group assignment (index into a shader list).
    pub shader_index: Option<usize>,
    /// Per-node render mode override.
    pub render_mode: Option<RenderMode>,
    /// Per-occurrence transform override (None = use inherited `transform`).
    /// Matches GLC_lib 2.5.2's makeFlexible/makeRigid system.
    pub flexible_transform: Option<Transform>,
}

impl SceneNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            name: name.into(),
            transform: Transform::IDENTITY,
            mesh_index: None,
            children: Vec::new(),
            parent: None,
            visible: true,
            shader_index: None,
            render_mode: None,
            flexible_transform: None,
        }
    }

    /// Whether this node has a flexible (per-occurrence) transform override.
    pub fn is_flexible(&self) -> bool {
        self.flexible_transform.is_some()
    }

    /// Returns the effective transform: flexible override if set, else the base transform.
    pub fn effective_transform(&self) -> Transform {
        self.flexible_transform.unwrap_or(self.transform)
    }

    /// Set a flexible (per-occurrence) transform override.
    pub fn make_flexible(&mut self, matrix: Transform) {
        self.flexible_transform = Some(matrix);
    }

    /// Remove the flexible transform override, reverting to the base transform.
    pub fn make_rigid(&mut self) {
        self.flexible_transform = None;
    }

    /// Create a node with geometry.
    pub fn with_mesh(name: impl Into<String>, mesh_index: usize) -> Self {
        Self {
            mesh_index: Some(mesh_index),
            ..Self::new(name)
        }
    }
}

/// The complete 3D world, matching GLC_World.
///
/// Contains all meshes, materials, and a scene graph of nodes.
/// Arena-based: nodes reference each other by `NodeIndex`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    /// All meshes in this world (shared, referenced by index).
    pub meshes: Vec<Mesh>,
    /// All materials in this world (shared, referenced by index).
    pub materials: Vec<Material>,
    /// Scene graph nodes (arena allocation).
    pub nodes: Vec<SceneNode>,
    /// Index of the root node.
    pub root: Option<NodeIndex>,
    /// Source file path (if loaded from file).
    pub source_path: Option<String>,
    /// Schema version from file header (e.g. "3.0", "4.0").
    pub schema_version: Option<String>,
    /// Title from file header.
    pub header_title: Option<String>,
    /// Generator from file header (e.g. "SolidWorks 2024").
    pub header_generator: Option<String>,
    /// Default camera eye position from file (e.g. DefaultView).
    pub default_camera_eye: Option<[f32; 3]>,
    /// Default camera target position from file (e.g. DefaultView).
    pub default_camera_target: Option<[f32; 3]>,
}

impl World {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
            nodes: Vec::new(),
            root: None,
            source_path: None,
            schema_version: None,
            header_title: None,
            header_generator: None,
            default_camera_eye: None,
            default_camera_target: None,
        }
    }

    /// Add a node to the arena and return its index.
    pub fn add_node(&mut self, node: SceneNode) -> NodeIndex {
        let idx = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        idx
    }

    /// Set a node as a child of another node.
    pub fn set_parent(&mut self, child: NodeIndex, parent: NodeIndex) {
        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.push(child);
    }

    /// Add a mesh and return its index.
    pub fn add_mesh(&mut self, mesh: Mesh) -> usize {
        let idx = self.meshes.len();
        self.meshes.push(mesh);
        idx
    }

    /// Add a material and return its index.
    pub fn add_material(&mut self, material: Material) -> usize {
        let idx = self.materials.len();
        self.materials.push(material);
        idx
    }

    /// Total number of triangles across all meshes.
    pub fn total_face_count(&self) -> usize {
        self.meshes.iter().map(|m| m.face_count()).sum()
    }

    /// Total number of vertices across all meshes.
    pub fn total_vertex_count(&self) -> usize {
        self.meshes.iter().map(|m| m.vertex_count()).sum()
    }

    /// Number of instances (leaf nodes with geometry).
    pub fn instance_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.mesh_index.is_some()).count()
    }

    /// Compute the world-space bounding box of the entire scene.
    pub fn bounding_box(&self) -> BoundingBox {
        let mut bb = BoundingBox::EMPTY;
        for mesh in &self.meshes {
            let mesh_bb = mesh.bounding_box();
            if !mesh_bb.is_empty() {
                bb = bb.combine(&mesh_bb);
            }
        }
        bb
    }

    /// Compute the world-space bounding box for a node and all its descendants.
    pub fn compute_node_bounding_box(&self, node: NodeIndex) -> BoundingBox {
        let mut bb = BoundingBox::EMPTY;
        self.collect_node_bounds(node, &mut bb);
        bb
    }

    fn collect_node_bounds(&self, node: NodeIndex, bb: &mut BoundingBox) {
        if let Some(mesh_idx) = self.nodes[node.0].mesh_index {
            let mesh_bb = self.meshes[mesh_idx].bounding_box();
            if !mesh_bb.is_empty() {
                let abs_transform = self.compute_absolute_transform(node);
                for corner in mesh_bb.corners() {
                    bb.expand(abs_transform.transform_point(corner));
                }
            }
        }
        for &child in &self.nodes[node.0].children {
            self.collect_node_bounds(child, bb);
        }
    }

    /// Insert a child at a specific position in the parent's children list.
    pub fn insert_child(&mut self, parent: NodeIndex, position: usize, child: NodeIndex) {
        self.nodes[child.0].parent = Some(parent);
        let children = &mut self.nodes[parent.0].children;
        let pos = position.min(children.len());
        children.insert(pos, child);
    }

    /// Swap two children of a parent node by their positions.
    pub fn swap_children(&mut self, parent: NodeIndex, i: usize, j: usize) {
        self.nodes[parent.0].children.swap(i, j);
    }

    /// Find the position of a child within a parent's children list.
    pub fn index_of_child(&self, parent: NodeIndex, child: NodeIndex) -> Option<usize> {
        self.nodes[parent.0]
            .children
            .iter()
            .position(|c| *c == child)
    }

    /// Check if a node is a direct child of a parent.
    pub fn contains_child(&self, parent: NodeIndex, child: NodeIndex) -> bool {
        self.nodes[parent.0].children.contains(&child)
    }

    /// Collect all ancestors of a node, from parent up to root.
    pub fn ancestor_list(&self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut ancestors = Vec::new();
        let mut current = self.nodes[node.0].parent;
        while let Some(parent_idx) = current {
            ancestors.push(parent_idx);
            current = self.nodes[parent_idx.0].parent;
        }
        ancestors
    }

    /// Set visibility on a node and all its descendants recursively.
    pub fn set_visibility_recursive(&mut self, node: NodeIndex, visible: bool) {
        self.nodes[node.0].visible = visible;
        let children: Vec<NodeIndex> = self.nodes[node.0].children.clone();
        for child in children {
            self.set_visibility_recursive(child, visible);
        }
    }

    /// Take the root node index, leaving the World without a root.
    pub fn take_root(&mut self) -> Option<NodeIndex> {
        self.root.take()
    }

    /// Replace the root node.
    pub fn replace_root(&mut self, new_root: NodeIndex) {
        self.root = Some(new_root);
    }

    /// Compute the absolute (world-space) transform for a node by walking the parent chain.
    /// Uses effective_transform (respects flexible overrides).
    pub fn compute_absolute_transform(&self, node: NodeIndex) -> Transform {
        let mut result = self.nodes[node.0].effective_transform();
        let mut current = self.nodes[node.0].parent;
        while let Some(parent_idx) = current {
            let parent_transform = self.nodes[parent_idx.0].effective_transform();
            result = Transform::new(parent_transform.matrix * result.matrix);
            current = self.nodes[parent_idx.0].parent;
        }
        result
    }

    /// Collect names of all invisible (hidden) instances.
    pub fn invisible_instance_names(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|n| !n.visible && n.mesh_index.is_some())
            .map(|n| n.name.clone())
            .collect()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_scene() {
        let mut world = World::new();

        // Add a material
        let mat_idx = world.add_material(Material::default());
        assert_eq!(mat_idx, 0);

        // Add a mesh (empty for now)
        let mesh_idx = world.add_mesh(Mesh::default());
        assert_eq!(mesh_idx, 0);

        // Build scene graph
        let root = world.add_node(SceneNode::new("Root"));
        let child = world.add_node(SceneNode::with_mesh("Cube", mesh_idx));
        world.set_parent(child, root);
        world.root = Some(root);

        assert_eq!(world.nodes.len(), 2);
        assert_eq!(world.nodes[root.0].children.len(), 1);
        assert_eq!(world.nodes[child.0].parent, Some(root));
        assert_eq!(world.instance_count(), 1);
    }

    #[test]
    fn test_flexible_occurrence() {
        let mut node = SceneNode::new("Flex");
        assert!(!node.is_flexible());
        assert_eq!(node.effective_transform(), Transform::IDENTITY);

        let t = Transform::from_translation(glam::Vec3::new(1.0, 2.0, 3.0));
        node.make_flexible(t);
        assert!(node.is_flexible());
        assert_eq!(node.effective_transform(), t);

        node.make_rigid();
        assert!(!node.is_flexible());
        assert_eq!(node.effective_transform(), Transform::IDENTITY);
    }

    #[test]
    fn test_insert_child_at_position() {
        let mut world = World::new();
        let root = world.add_node(SceneNode::new("Root"));
        let a = world.add_node(SceneNode::new("A"));
        let b = world.add_node(SceneNode::new("B"));
        let c = world.add_node(SceneNode::new("C"));
        world.root = Some(root);

        // Insert at end (only child)
        world.insert_child(root, 100, a); // clamped to 0
        assert_eq!(world.nodes[root.0].children, vec![a]);

        // Insert at end
        world.insert_child(root, 1, b);
        assert_eq!(world.nodes[root.0].children, vec![a, b]);

        // Insert at beginning
        world.insert_child(root, 0, c);
        assert_eq!(world.nodes[root.0].children, vec![c, a, b]);
    }

    #[test]
    fn test_swap_children() {
        let mut world = World::new();
        let root = world.add_node(SceneNode::new("Root"));
        let a = world.add_node(SceneNode::new("A"));
        let b = world.add_node(SceneNode::new("B"));
        let c = world.add_node(SceneNode::new("C"));
        world.set_parent(a, root);
        world.set_parent(b, root);
        world.set_parent(c, root);
        world.root = Some(root);

        assert_eq!(world.nodes[root.0].children, vec![a, b, c]);
        world.swap_children(root, 0, 2);
        assert_eq!(world.nodes[root.0].children, vec![c, b, a]);
    }

    #[test]
    fn test_ancestor_list() {
        let mut world = World::new();
        let root = world.add_node(SceneNode::new("Root"));
        let mid = world.add_node(SceneNode::new("Mid"));
        let leaf = world.add_node(SceneNode::new("Leaf"));
        world.set_parent(mid, root);
        world.set_parent(leaf, mid);
        world.root = Some(root);

        let ancestors = world.ancestor_list(leaf);
        assert_eq!(ancestors, vec![mid, root]);

        let root_ancestors = world.ancestor_list(root);
        assert!(root_ancestors.is_empty());
    }

    #[test]
    fn test_set_visibility_recursive() {
        let mut world = World::new();
        let root = world.add_node(SceneNode::new("Root"));
        let a = world.add_node(SceneNode::new("A"));
        let b = world.add_node(SceneNode::new("B"));
        let c = world.add_node(SceneNode::new("C"));
        world.set_parent(a, root);
        world.set_parent(b, root);
        world.set_parent(c, a);
        world.root = Some(root);

        // All start visible
        assert!(world.nodes[root.0].visible);
        assert!(world.nodes[a.0].visible);
        assert!(world.nodes[c.0].visible);

        // Hide root — cascades to all descendants
        world.set_visibility_recursive(root, false);
        assert!(!world.nodes[root.0].visible);
        assert!(!world.nodes[a.0].visible);
        assert!(!world.nodes[b.0].visible);
        assert!(!world.nodes[c.0].visible);

        // Show subtree a — only a and c become visible
        world.set_visibility_recursive(a, true);
        assert!(!world.nodes[root.0].visible); // root stays hidden
        assert!(world.nodes[a.0].visible);
        assert!(!world.nodes[b.0].visible); // b stays hidden
        assert!(world.nodes[c.0].visible);
    }

    #[test]
    fn test_compute_absolute_transform() {
        let mut world = World::new();

        let t1 = Transform::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
        let t2 = Transform::from_translation(glam::Vec3::new(0.0, 2.0, 0.0));
        let t3 = Transform::from_translation(glam::Vec3::new(0.0, 0.0, 3.0));

        let mut root_node = SceneNode::new("Root");
        root_node.transform = t1;
        let mut mid_node = SceneNode::new("Mid");
        mid_node.transform = t2;
        let mut leaf_node = SceneNode::new("Leaf");
        leaf_node.transform = t3;

        let root = world.add_node(root_node);
        let mid = world.add_node(mid_node);
        let leaf = world.add_node(leaf_node);
        world.set_parent(mid, root);
        world.set_parent(leaf, mid);
        world.root = Some(root);

        // Absolute transform of leaf = root * mid * leaf
        let abs_t = world.compute_absolute_transform(leaf);
        let point = abs_t.transform_point(glam::Vec3::ZERO);
        assert!((point.x - 1.0).abs() < 1e-6);
        assert!((point.y - 2.0).abs() < 1e-6);
        assert!((point.z - 3.0).abs() < 1e-6);

        // Test flexible override: set mid's flexible transform to identity
        world.nodes[mid.0].make_flexible(Transform::IDENTITY);
        let abs_t2 = world.compute_absolute_transform(leaf);
        let point2 = abs_t2.transform_point(glam::Vec3::ZERO);
        // Now mid contributes identity, so result = root * identity * leaf = (1,0,3)
        assert!((point2.x - 1.0).abs() < 1e-6);
        assert!((point2.y - 0.0).abs() < 1e-6);
        assert!((point2.z - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_node_bounding_box() {
        use crate::mesh::Mesh;

        let mut world = World::new();

        // Create a mesh with known bounding box (unit cube at origin)
        let mut mesh = Mesh::default();
        mesh.positions = vec![
            0.0, 0.0, 0.0, // min corner
            1.0, 1.0, 1.0, // max corner
        ];
        let mesh_idx = world.add_mesh(mesh);

        // Root at origin, child translated by (10, 0, 0)
        let root = world.add_node(SceneNode::new("Root"));
        let mut child_node = SceneNode::with_mesh("Child", mesh_idx);
        child_node.transform = Transform::from_translation(glam::Vec3::new(10.0, 0.0, 0.0));
        let child = world.add_node(child_node);
        world.set_parent(child, root);
        world.root = Some(root);

        // Node bounding box for child should be in world space (translated)
        let bb = world.compute_node_bounding_box(child);
        assert!(!bb.is_empty());
        assert!((bb.min.x - 10.0).abs() < 1e-5);
        assert!((bb.max.x - 11.0).abs() < 1e-5);

        // Node bounding box for root should include all descendants
        let root_bb = world.compute_node_bounding_box(root);
        assert!(!root_bb.is_empty());
        assert!((root_bb.min.x - 10.0).abs() < 1e-5);
        assert!((root_bb.max.x - 11.0).abs() < 1e-5);
    }

    #[test]
    fn test_invisible_instances() {
        let mut world = World::new();
        let mesh_idx = world.add_mesh(Mesh::default());

        let root = world.add_node(SceneNode::new("Root"));
        let mut visible_node = SceneNode::with_mesh("Visible", mesh_idx);
        visible_node.visible = true;
        let mut hidden_node = SceneNode::with_mesh("Hidden", mesh_idx);
        hidden_node.visible = false;

        let vis = world.add_node(visible_node);
        let hid = world.add_node(hidden_node);
        world.set_parent(vis, root);
        world.set_parent(hid, root);
        world.root = Some(root);

        let invisible = world.invisible_instance_names();
        assert_eq!(invisible, vec!["Hidden".to_string()]);
    }
}
