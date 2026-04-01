#[derive(Debug, Clone)]
pub struct RenderMaterialOverrideEntry {
    pub te_index: u8,
    pub data: String,
}

#[derive(Debug, Clone, Default)]
pub struct RenderMaterials {
    pub overrides: Vec<RenderMaterialOverrideEntry>,
}
