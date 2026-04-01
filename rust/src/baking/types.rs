use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BakeType {
    Head = 0,
    UpperBody = 1,
    LowerBody = 2,
    Eyes = 3,
    Skirt = 4,
    Hair = 5,
    BakedLeftArm = 6,
    BakedLeftLeg = 7,
    BakedAux1 = 8,
    BakedAux2 = 9,
    BakedAux3 = 10,
    Unknown = 255,
}

impl BakeType {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            BakeType::Eyes => (128, 128),
            _ => (1024, 1024),
        }
    }

    pub fn is_skin(&self) -> bool {
        matches!(self, BakeType::Head | BakeType::UpperBody | BakeType::LowerBody)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AvatarTextureIndex {
    HeadBodypaint = 0,
    UpperShirt = 1,
    LowerPants = 2,
    EyesIris = 3,
    Hair = 4,
    UpperBodypaint = 5,
    LowerBodypaint = 6,
    LowerShoes = 7,
    HeadBaked = 8,
    UpperBaked = 9,
    LowerBaked = 10,
    EyesBaked = 11,
    UpperGloves = 12,
    UpperUndershirt = 13,
    LowerUnderpants = 14,
    LowerSocks = 15,
    UpperJacket = 16,
    LowerJacket = 17,
    UpperJacketOpen = 18,
    LowerJacketOpen = 19,
    Skirt = 20,
    SkirtBaked = 21,
    HairBaked = 22,
    LowerAlpha = 23,
    UpperAlpha = 24,
    HeadAlpha = 25,
    EyesAlpha = 26,
    HairAlpha = 27,
    HeadTattoo = 28,
    UpperTattoo = 29,
    LowerTattoo = 30,
    LeftArmBaked = 31,
    LegLegBaked = 32,
    Aux1Baked = 33,
    Aux2Baked = 34,
    Aux3Baked = 35,
    LeftArmTattoo = 36,
    LeftLegTattoo = 37,
    Aux1Tattoo = 38,
    Aux2Tattoo = 39,
    Aux3Tattoo = 40,
    Unknown = 255,
}

impl AvatarTextureIndex {
    pub fn to_bake_type(&self) -> BakeType {
        match self {
            AvatarTextureIndex::HeadBodypaint => BakeType::Head,
            AvatarTextureIndex::UpperBodypaint
            | AvatarTextureIndex::UpperGloves
            | AvatarTextureIndex::UpperUndershirt
            | AvatarTextureIndex::UpperShirt
            | AvatarTextureIndex::UpperJacket => BakeType::UpperBody,
            AvatarTextureIndex::LowerBodypaint
            | AvatarTextureIndex::LowerUnderpants
            | AvatarTextureIndex::LowerSocks
            | AvatarTextureIndex::LowerShoes
            | AvatarTextureIndex::LowerPants
            | AvatarTextureIndex::LowerJacket => BakeType::LowerBody,
            AvatarTextureIndex::EyesIris => BakeType::Eyes,
            AvatarTextureIndex::Skirt => BakeType::Skirt,
            AvatarTextureIndex::Hair => BakeType::Hair,
            _ => BakeType::Unknown,
        }
    }
}

impl BakeType {
    pub fn to_agent_texture_index(&self) -> AvatarTextureIndex {
        match self {
            BakeType::Head => AvatarTextureIndex::HeadBaked,
            BakeType::UpperBody => AvatarTextureIndex::UpperBaked,
            BakeType::LowerBody => AvatarTextureIndex::LowerBaked,
            BakeType::Eyes => AvatarTextureIndex::EyesBaked,
            BakeType::Skirt => AvatarTextureIndex::SkirtBaked,
            BakeType::Hair => AvatarTextureIndex::HairBaked,
            BakeType::BakedLeftArm => AvatarTextureIndex::LeftArmBaked,
            BakeType::BakedLeftLeg => AvatarTextureIndex::LegLegBaked,
            BakeType::BakedAux1 => AvatarTextureIndex::Aux1Baked,
            BakeType::BakedAux2 => AvatarTextureIndex::Aux2Baked,
            BakeType::BakedAux3 => AvatarTextureIndex::Aux3Baked,
            BakeType::Unknown => AvatarTextureIndex::Unknown,
        }
    }

    pub fn get_texture_indices(&self) -> Vec<AvatarTextureIndex> {
        match self {
            BakeType::Head => vec![
                AvatarTextureIndex::HeadBodypaint,
                AvatarTextureIndex::HeadTattoo,
                AvatarTextureIndex::Hair,
                AvatarTextureIndex::HeadAlpha,
            ],
            BakeType::UpperBody => vec![
                AvatarTextureIndex::UpperBodypaint,
                AvatarTextureIndex::UpperTattoo,
                AvatarTextureIndex::UpperGloves,
                AvatarTextureIndex::UpperUndershirt,
                AvatarTextureIndex::UpperShirt,
                AvatarTextureIndex::UpperJacket,
                AvatarTextureIndex::UpperAlpha,
            ],
            BakeType::LowerBody => vec![
                AvatarTextureIndex::LowerBodypaint,
                AvatarTextureIndex::LowerTattoo,
                AvatarTextureIndex::LowerUnderpants,
                AvatarTextureIndex::LowerSocks,
                AvatarTextureIndex::LowerShoes,
                AvatarTextureIndex::LowerPants,
                AvatarTextureIndex::LowerJacket,
                AvatarTextureIndex::LowerAlpha,
            ],
            BakeType::Eyes => vec![
                AvatarTextureIndex::EyesIris,
                AvatarTextureIndex::EyesAlpha,
            ],
            BakeType::Skirt => vec![AvatarTextureIndex::Skirt],
            BakeType::Hair => vec![
                AvatarTextureIndex::Hair,
                AvatarTextureIndex::HairAlpha,
            ],
            _ => vec![],
        }
    }

    pub fn morph_layer(&self) -> AvatarTextureIndex {
        match self {
            BakeType::Head => AvatarTextureIndex::Hair,
            BakeType::UpperBody => AvatarTextureIndex::UpperShirt,
            BakeType::LowerBody => AvatarTextureIndex::LowerPants,
            BakeType::Skirt => AvatarTextureIndex::Skirt,
            BakeType::Hair => AvatarTextureIndex::Hair,
            BakeType::BakedLeftArm => AvatarTextureIndex::LeftArmTattoo,
            BakeType::BakedLeftLeg => AvatarTextureIndex::LeftLegTattoo,
            BakeType::BakedAux1 => AvatarTextureIndex::Aux1Tattoo,
            BakeType::BakedAux2 => AvatarTextureIndex::Aux2Tattoo,
            BakeType::BakedAux3 => AvatarTextureIndex::Aux3Tattoo,
            _ => AvatarTextureIndex::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub const WHITE: Color4 = Color4 { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color4 = Color4 { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color4 = Color4 { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
            (self.a.clamp(0.0, 1.0) * 255.0) as u8,
        ]
    }
}

impl Default for Color4 {
    fn default() -> Self {
        Self::WHITE
    }
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub texture_id: Uuid,
    pub texture_index: AvatarTextureIndex,
    pub color: Color4,
    pub texture_data: Option<Vec<u8>>,
}

impl Default for TextureData {
    fn default() -> Self {
        Self {
            texture_id: Uuid::nil(),
            texture_index: AvatarTextureIndex::Unknown,
            color: Color4::WHITE,
            texture_data: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WearableCacheItem {
    pub cache_id: Uuid,
    pub texture_index: u32,
    pub texture_id: Uuid,
    pub texture_asset: Option<Vec<u8>>,
}

pub const BAKED_TEXTURE_COUNT: usize = 6;
pub const TEXTURE_COUNT: usize = 41;
