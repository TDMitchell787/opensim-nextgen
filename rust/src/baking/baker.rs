use super::{
    managed_image::{ImageChannels, ManagedImage},
    static_assets::StaticAssets,
    types::{AvatarTextureIndex, BakeType, Color4, TextureData},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct Baker {
    bake_type: BakeType,
    bake_width: u32,
    bake_height: u32,
    textures: Vec<TextureData>,
    baked_texture: Option<ManagedImage>,
}

impl Baker {
    pub fn new(bake_type: BakeType) -> Self {
        let (bake_width, bake_height) = bake_type.dimensions();
        Self {
            bake_type,
            bake_width,
            bake_height,
            textures: Vec::new(),
            baked_texture: None,
        }
    }

    pub fn add_texture(&mut self, texture_data: TextureData) {
        self.textures.push(texture_data);
    }

    pub fn bake(&mut self) -> Result<(), String> {
        info!(
            "🎨 Starting bake for {:?} ({}x{})",
            self.bake_type, self.bake_width, self.bake_height
        );

        let mut baked = ManagedImage::new(
            self.bake_width,
            self.bake_height,
            ImageChannels::COLOR | ImageChannels::ALPHA | ImageChannels::BUMP,
        );

        let mut skin_texture: Option<&TextureData> = None;
        let mut tattoo_textures: Vec<&TextureData> = Vec::new();
        let mut alpha_wearable_textures: Vec<ManagedImage> = Vec::new();

        if self.bake_type == BakeType::Eyes {
            self.init_baked_layer_color(&mut baked, Color4::WHITE);
        } else if !self.textures.is_empty() {
            self.init_baked_layer_color(&mut baked, self.textures[0].color);
        }

        for tex in &self.textures {
            if tex.texture_data.is_none() {
                continue;
            }

            match tex.texture_index {
                AvatarTextureIndex::HeadBodypaint
                | AvatarTextureIndex::UpperBodypaint
                | AvatarTextureIndex::LowerBodypaint => {
                    skin_texture = Some(tex);
                }
                AvatarTextureIndex::HeadTattoo
                | AvatarTextureIndex::UpperTattoo
                | AvatarTextureIndex::LowerTattoo => {
                    tattoo_textures.push(tex);
                }
                _ => {}
            }

            if self.is_alpha_texture(&tex.texture_index) {
                if let Some(data) = &tex.texture_data {
                    if let Ok(img) = ManagedImage::from_bytes(data) {
                        if !img.alpha.is_empty() {
                            alpha_wearable_textures.push(img);
                        }
                    }
                }
            }
        }

        if self.bake_type == BakeType::Head {
            if let Some(resource) = StaticAssets::load_resource("head_color.tga") {
                if self.draw_layer(&mut baked, &resource, false) {
                    if let Some(alpha_resource) = StaticAssets::load_resource("head_alpha.tga") {
                        self.add_alpha(&mut baked, &alpha_resource);
                    }
                    if let Some(grain_resource) = StaticAssets::load_resource("head_skingrain.tga")
                    {
                        self.multiply_layer_from_alpha(&mut baked, &grain_resource);
                    }
                    debug!("[Bake]: created head master bake");
                }
            }
        }

        if skin_texture.is_none() {
            if self.bake_type == BakeType::UpperBody {
                if let Some(resource) = StaticAssets::load_resource("upperbody_color.tga") {
                    self.draw_layer(&mut baked, &resource, false);
                }
            }
            if self.bake_type == BakeType::LowerBody {
                if let Some(resource) = StaticAssets::load_resource("lowerbody_color.tga") {
                    self.draw_layer(&mut baked, &resource, false);
                }
            }
        }

        for (i, tex) in self.textures.iter().enumerate() {
            if tex.texture_data.is_none() {
                continue;
            }

            if self.is_alpha_texture(&tex.texture_index) {
                continue;
            }

            if self.bake_type == BakeType::Head
                && (tex.texture_index == AvatarTextureIndex::HeadBodypaint
                    || tex.texture_index == AvatarTextureIndex::HeadTattoo)
            {
                continue;
            }

            let texture_data = tex.texture_data.as_ref().unwrap();
            let mut texture = match ManagedImage::from_bytes(texture_data) {
                Ok(t) => t,
                Err(e) => {
                    warn!("Failed to decode texture {}: {}", tex.texture_id, e);
                    continue;
                }
            };

            if texture.width != self.bake_width || texture.height != self.bake_height {
                texture.resize_nearest_neighbor(self.bake_width, self.bake_height);
            }

            if skin_texture.is_none()
                && self.bake_type == BakeType::Head
                && tex.texture_index == AvatarTextureIndex::Hair
            {
                if !texture.alpha.is_empty() {
                    for j in 0..texture.alpha.len() {
                        texture.alpha[j] = 255;
                    }
                }
                if let Some(hair_resource) = StaticAssets::load_resource("head_hair.tga") {
                    self.multiply_layer_from_alpha(&mut texture, &hair_resource);
                }
            }

            if !matches!(
                tex.texture_index,
                AvatarTextureIndex::HeadBodypaint
                    | AvatarTextureIndex::UpperBodypaint
                    | AvatarTextureIndex::LowerBodypaint
            ) {
                self.apply_tint(&mut texture, tex.color);

                if self.bake_type == BakeType::Hair {
                    if !texture.alpha.is_empty() {
                        baked.bump = texture.alpha.clone();
                    } else {
                        for j in 0..baked.bump.len() {
                            baked.bump[j] = 255;
                        }
                    }
                }

                if tex.texture_index == self.bake_type.morph_layer() {
                    baked.bump = texture.alpha.clone();
                }
            }

            let use_alpha =
                i == 0 && (self.bake_type == BakeType::Skirt || self.bake_type == BakeType::Hair);
            self.draw_layer(&mut baked, &texture, use_alpha);
        }

        if self.bake_type == BakeType::Head {
            if let Some(skin_tex) = skin_texture {
                if let Some(data) = &skin_tex.texture_data {
                    if let Ok(mut texture) = ManagedImage::from_bytes(data) {
                        if texture.width != self.bake_width || texture.height != self.bake_height {
                            texture.resize_nearest_neighbor(self.bake_width, self.bake_height);
                        }
                        self.draw_layer(&mut baked, &texture, false);
                    }
                }
            }

            for tattoo_tex in tattoo_textures {
                if let Some(data) = &tattoo_tex.texture_data {
                    if let Ok(mut texture) = ManagedImage::from_bytes(data) {
                        if texture.width != self.bake_width || texture.height != self.bake_height {
                            texture.resize_nearest_neighbor(self.bake_width, self.bake_height);
                        }
                        self.draw_layer(&mut baked, &texture, false);
                    }
                }
            }
        }

        debug!(
            "[XBakes]: Number of alpha wearable textures: {}",
            alpha_wearable_textures.len()
        );
        for alpha_img in &alpha_wearable_textures {
            self.add_alpha(&mut baked, alpha_img);
        }

        self.baked_texture = Some(baked);
        info!("🎨 Bake complete for {:?}", self.bake_type);
        Ok(())
    }

    pub fn get_baked_texture(&self) -> Option<&ManagedImage> {
        self.baked_texture.as_ref()
    }

    pub fn get_baked_texture_bytes(&self) -> Option<Vec<u8>> {
        self.baked_texture.as_ref().map(|t| t.to_jpeg_bytes(90))
    }

    pub fn bake_type(&self) -> BakeType {
        self.bake_type
    }

    fn is_alpha_texture(&self, index: &AvatarTextureIndex) -> bool {
        matches!(
            index,
            AvatarTextureIndex::LowerAlpha
                | AvatarTextureIndex::UpperAlpha
                | AvatarTextureIndex::HeadAlpha
                | AvatarTextureIndex::EyesAlpha
                | AvatarTextureIndex::HairAlpha
        )
    }

    fn init_baked_layer_color(&self, dest: &mut ManagedImage, color: Color4) {
        dest.fill_dithered(color.r, color.g, color.b);
    }

    fn draw_layer(
        &self,
        dest: &mut ManagedImage,
        source: &ManagedImage,
        add_source_alpha: bool,
    ) -> bool {
        let source_has_color = source.channels.has_color() && !source.red.is_empty();
        let source_has_alpha = source.channels.has_alpha() && !source.alpha.is_empty();
        let add_source_alpha = add_source_alpha && source_has_alpha;

        let mut i = 0;
        for _y in 0..self.bake_height {
            for _x in 0..self.bake_width {
                let mut alpha: u8 = 0;
                let mut alpha_inv: u8 = 255;
                let mut loaded_alpha = false;

                if source_has_alpha && i < source.alpha.len() {
                    loaded_alpha = true;
                    alpha = source.alpha[i];
                    alpha_inv = 255 - alpha;
                }

                if source_has_color
                    && i < dest.red.len()
                    && i < dest.green.len()
                    && i < dest.blue.len()
                    && i < source.red.len()
                    && i < source.green.len()
                    && i < source.blue.len()
                {
                    if loaded_alpha {
                        dest.red[i] = ((dest.red[i] as u16 * alpha_inv as u16
                            + source.red[i] as u16 * alpha as u16)
                            >> 8) as u8;
                        dest.green[i] = ((dest.green[i] as u16 * alpha_inv as u16
                            + source.green[i] as u16 * alpha as u16)
                            >> 8) as u8;
                        dest.blue[i] = ((dest.blue[i] as u16 * alpha_inv as u16
                            + source.blue[i] as u16 * alpha as u16)
                            >> 8) as u8;
                    } else {
                        dest.red[i] = source.red[i];
                        dest.green[i] = source.green[i];
                        dest.blue[i] = source.blue[i];
                    }
                }

                if add_source_alpha && i < source.alpha.len() && i < dest.alpha.len() {
                    if source.alpha[i] < dest.alpha[i] {
                        dest.alpha[i] = source.alpha[i];
                    }
                }

                if source.channels.has_bump() && i < source.bump.len() && i < dest.bump.len() {
                    dest.bump[i] = source.bump[i];
                }

                i += 1;
            }
        }

        true
    }

    fn add_alpha(&self, dest: &mut ManagedImage, src: &ManagedImage) {
        if dest.alpha.is_empty() || src.alpha.is_empty() {
            return;
        }

        let len = dest.alpha.len().min(src.alpha.len());
        for i in 0..len {
            if src.alpha[i] < dest.alpha[i] {
                dest.alpha[i] = src.alpha[i];
            }
        }
    }

    fn multiply_layer_from_alpha(&self, dest: &mut ManagedImage, src: &ManagedImage) {
        if src.alpha.is_empty() || dest.red.is_empty() {
            return;
        }

        let len = dest.red.len().min(src.alpha.len());
        for i in 0..len {
            dest.red[i] = ((dest.red[i] as u16 * src.alpha[i] as u16) >> 8) as u8;
            dest.green[i] = ((dest.green[i] as u16 * src.alpha[i] as u16) >> 8) as u8;
            dest.blue[i] = ((dest.blue[i] as u16 * src.alpha[i] as u16) >> 8) as u8;
        }
    }

    fn apply_tint(&self, dest: &mut ManagedImage, color: Color4) {
        if dest.red.is_empty() {
            return;
        }

        let r_byte = (color.r.clamp(0.0, 1.0) * 255.0) as u16;
        let g_byte = (color.g.clamp(0.0, 1.0) * 255.0) as u16;
        let b_byte = (color.b.clamp(0.0, 1.0) * 255.0) as u16;

        for i in 0..dest.red.len() {
            dest.red[i] = ((dest.red[i] as u16 * r_byte) >> 8) as u8;
            dest.green[i] = ((dest.green[i] as u16 * g_byte) >> 8) as u8;
            dest.blue[i] = ((dest.blue[i] as u16 * b_byte) >> 8) as u8;
        }
    }
}

pub fn create_default_baked_texture(bake_type: BakeType) -> ManagedImage {
    let (width, height) = bake_type.dimensions();
    let mut img = ManagedImage::new(width, height, ImageChannels::COLOR | ImageChannels::ALPHA);

    let (r, g, b) = match bake_type {
        BakeType::Head | BakeType::UpperBody | BakeType::LowerBody => (0.8, 0.7, 0.6),
        BakeType::Eyes => (0.3, 0.5, 0.8),
        BakeType::Hair => (0.3, 0.2, 0.1),
        BakeType::Skirt => (0.5, 0.5, 0.5),
        _ => (0.8, 0.8, 0.8),
    };

    img.fill_dithered(r, g, b);
    img
}
