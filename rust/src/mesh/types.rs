use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BlockRef {
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct MeshAssetHeader {
    pub data_start: usize,
    pub physics_convex: Option<BlockRef>,
    pub physics_mesh: Option<BlockRef>,
    pub high_lod: Option<BlockRef>,
    pub medium_lod: Option<BlockRef>,
    pub low_lod: Option<BlockRef>,
    pub lowest_lod: Option<BlockRef>,
    pub skin: Option<BlockRef>,
}

#[derive(Debug, Clone)]
pub struct ConvexPhysicsData {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub bounding_hull: Vec<[f32; 3]>,
    pub hulls: Vec<Vec<[f32; 3]>>,
}

impl ConvexPhysicsData {
    pub fn total_vertices(&self) -> usize {
        self.hulls.iter().map(|h| h.len()).sum()
    }

    pub fn hull_count(&self) -> usize {
        self.hulls.len()
    }
}
