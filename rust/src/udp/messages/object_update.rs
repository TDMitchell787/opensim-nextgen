use bytes::{BufMut, BytesMut};
use uuid::Uuid;

pub const PCODE_AVATAR: u8 = 47;
pub const MATERIAL_FLESH: u8 = 4;

pub fn build_ruth_avatar_texture_entry() -> Vec<u8> {
    let mut te = Vec::with_capacity(300);

    let default_tex = Uuid::parse_str("c228d1cf-4b5d-4ba8-84f4-899a0796aa97").unwrap();
    te.extend_from_slice(default_tex.as_bytes());

    let default_avatar_tex = Uuid::parse_str("5748decc-f629-461c-9a36-a35a221fe21f").unwrap();
    let bake_faces: [u32; 5] = [8, 9, 10, 11, 20];
    for face in &bake_faces {
        te.extend_from_slice(&encode_face_bitfield(*face));
        te.extend_from_slice(default_avatar_tex.as_bytes());
    }

    te.push(0);

    te.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    te.push(0);

    te.extend_from_slice(&1.0f32.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&1.0f32.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);

    te.push(0); te.push(0);
    te.push(0); te.push(0);
    te.push(0); te.push(0);
    te.extend_from_slice(&[0u8; 16]);

    te
}

fn encode_face_bitfield(face: u32) -> Vec<u8> {
    let bitfield: u64 = 1u64 << face;

    let mut byte_length = 0;
    let mut tmp = bitfield;
    while tmp != 0 {
        tmp >>= 7;
        byte_length += 1;
    }

    if byte_length == 0 {
        return vec![0];
    }

    let mut result = Vec::with_capacity(byte_length);
    for i in 0..byte_length {
        let shift = 7 * (byte_length - i - 1);
        let mut byte = ((bitfield >> shift) & 0x7F) as u8;
        if i < byte_length - 1 {
            byte |= 0x80;
        }
        result.push(byte);
    }
    result
}

pub fn build_default_prim_texture_entry() -> Vec<u8> {
    build_prim_texture_entry(None, None)
}

pub fn build_colored_prim_texture_entry(color: [f32; 4]) -> Vec<u8> {
    build_prim_texture_entry(None, Some(color))
}

pub fn build_textured_prim_texture_entry(texture_uuid: Uuid) -> Vec<u8> {
    build_prim_texture_entry(Some(texture_uuid), None)
}

fn build_prim_texture_entry(texture: Option<Uuid>, color: Option<[f32; 4]>) -> Vec<u8> {
    let mut te = Vec::with_capacity(64);
    let tex = texture.unwrap_or_else(|| Uuid::parse_str("89556747-24cb-43ed-920b-47caed15465f").unwrap());
    te.extend_from_slice(tex.as_bytes());
    te.push(0);
    let c = color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
    te.extend_from_slice(&[
        (255 - (c[0].clamp(0.0, 1.0) * 255.0) as u8),
        (255 - (c[1].clamp(0.0, 1.0) * 255.0) as u8),
        (255 - (c[2].clamp(0.0, 1.0) * 255.0) as u8),
        (255 - (c[3].clamp(0.0, 1.0) * 255.0) as u8),
    ]);
    te.push(0);
    te.extend_from_slice(&1.0f32.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&1.0f32.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);
    te.extend_from_slice(&0i16.to_le_bytes());
    te.push(0);
    te.push(0); te.push(0);
    te.push(0); te.push(0);
    te.push(0); te.push(0);
    te.extend_from_slice(&[0u8; 16]);
    te.push(0);
    te
}

pub const ALL_SIDES: i32 = -1;
pub const MAX_FACES: usize = 9;

#[derive(Debug, Clone)]
pub struct TextureEntryFace {
    pub texture_id: Uuid,
    pub color: [f32; 4],
    pub repeat_u: f32,
    pub repeat_v: f32,
    pub offset_u: f32,
    pub offset_v: f32,
    pub rotation: f32,
    pub material: u8,
    pub media: u8,
    pub glow: u8,
}

impl Default for TextureEntryFace {
    fn default() -> Self {
        Self {
            texture_id: Uuid::parse_str("89556747-24cb-43ed-920b-47caed15465f").unwrap(),
            color: [1.0, 1.0, 1.0, 1.0],
            repeat_u: 1.0,
            repeat_v: 1.0,
            offset_u: 0.0,
            offset_v: 0.0,
            rotation: 0.0,
            material: 0,
            media: 0,
            glow: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureEntryData {
    pub default_face: TextureEntryFace,
    pub faces: [Option<TextureEntryFace>; MAX_FACES],
}

impl TextureEntryData {
    pub fn new() -> Self {
        Self {
            default_face: TextureEntryFace::default(),
            faces: Default::default(),
        }
    }

    pub fn get_face(&self, index: usize) -> &TextureEntryFace {
        if index < MAX_FACES {
            if let Some(ref f) = self.faces[index] {
                return f;
            }
        }
        &self.default_face
    }

    pub fn get_face_mut(&mut self, index: usize) -> &mut TextureEntryFace {
        if index < MAX_FACES {
            if self.faces[index].is_none() {
                self.faces[index] = Some(self.default_face.clone());
            }
            return self.faces[index].as_mut().unwrap();
        }
        &mut self.default_face
    }

    pub fn set_color(&mut self, face: i32, r: f32, g: f32, b: f32) {
        if face == ALL_SIDES {
            self.default_face.color[0] = r;
            self.default_face.color[1] = g;
            self.default_face.color[2] = b;
            for f in self.faces.iter_mut().flatten() {
                f.color[0] = r;
                f.color[1] = g;
                f.color[2] = b;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.color[0] = r;
            f.color[1] = g;
            f.color[2] = b;
        }
    }

    pub fn set_alpha(&mut self, face: i32, alpha: f32) {
        if face == ALL_SIDES {
            self.default_face.color[3] = alpha;
            for f in self.faces.iter_mut().flatten() {
                f.color[3] = alpha;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.color[3] = alpha;
        }
    }

    pub fn set_texture(&mut self, face: i32, texture_id: Uuid) {
        if face == ALL_SIDES {
            self.default_face.texture_id = texture_id;
            for f in self.faces.iter_mut().flatten() {
                f.texture_id = texture_id;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.texture_id = texture_id;
        }
    }

    pub fn set_scale(&mut self, face: i32, u: f32, v: f32) {
        if face == ALL_SIDES {
            self.default_face.repeat_u = u;
            self.default_face.repeat_v = v;
            for f in self.faces.iter_mut().flatten() {
                f.repeat_u = u;
                f.repeat_v = v;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.repeat_u = u;
            f.repeat_v = v;
        }
    }

    pub fn set_offset(&mut self, face: i32, u: f32, v: f32) {
        if face == ALL_SIDES {
            self.default_face.offset_u = u;
            self.default_face.offset_v = v;
            for f in self.faces.iter_mut().flatten() {
                f.offset_u = u;
                f.offset_v = v;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.offset_u = u;
            f.offset_v = v;
        }
    }

    pub fn set_rotation(&mut self, face: i32, rotation: f32) {
        if face == ALL_SIDES {
            self.default_face.rotation = rotation;
            for f in self.faces.iter_mut().flatten() {
                f.rotation = rotation;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.rotation = rotation;
        }
    }

    pub fn set_fullbright(&mut self, face: i32, value: bool) {
        let media_val = if value { 0x20 } else { 0x00 };
        if face == ALL_SIDES {
            self.default_face.media = (self.default_face.media & 0xDF) | media_val;
            for f in self.faces.iter_mut().flatten() {
                f.media = (f.media & 0xDF) | media_val;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.media = (f.media & 0xDF) | media_val;
        }
    }

    pub fn set_glow(&mut self, face: i32, intensity: f32) {
        let glow_byte = (intensity.clamp(0.0, 1.0) * 255.0) as u8;
        if face == ALL_SIDES {
            self.default_face.glow = glow_byte;
            for f in self.faces.iter_mut().flatten() {
                f.glow = glow_byte;
            }
        } else if (face as usize) < MAX_FACES {
            let f = self.get_face_mut(face as usize);
            f.glow = glow_byte;
        }
    }

    pub fn parse(data: &[u8]) -> Self {
        let mut te = Self::new();
        if data.is_empty() {
            return te;
        }
        let mut pos = 0;

        fn read_uuid(data: &[u8], pos: &mut usize) -> Uuid {
            if *pos + 16 <= data.len() {
                let uuid = Uuid::from_slice(&data[*pos..*pos + 16]).unwrap_or(Uuid::nil());
                *pos += 16;
                uuid
            } else {
                *pos = data.len();
                Uuid::nil()
            }
        }

        fn read_color4(data: &[u8], pos: &mut usize) -> [f32; 4] {
            if *pos + 4 <= data.len() {
                let r = (255 - data[*pos]) as f32 / 255.0;
                let g = (255 - data[*pos + 1]) as f32 / 255.0;
                let b = (255 - data[*pos + 2]) as f32 / 255.0;
                let a = (255 - data[*pos + 3]) as f32 / 255.0;
                *pos += 4;
                [r, g, b, a]
            } else {
                *pos = data.len();
                [1.0, 1.0, 1.0, 1.0]
            }
        }

        fn read_f32(data: &[u8], pos: &mut usize) -> f32 {
            if *pos + 4 <= data.len() {
                let val = f32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]);
                *pos += 4;
                val
            } else {
                *pos = data.len();
                0.0
            }
        }

        fn read_i16_as_f32(data: &[u8], pos: &mut usize) -> f32 {
            if *pos + 2 <= data.len() {
                let val = i16::from_le_bytes([data[*pos], data[*pos+1]]);
                *pos += 2;
                val as f32 / 32768.0 * std::f32::consts::PI
            } else {
                *pos = data.len();
                0.0
            }
        }

        fn read_bitfield(data: &[u8], pos: &mut usize) -> u32 {
            let mut facebits: u32 = 0;
            let mut bit_pos = 0u32;
            loop {
                if *pos >= data.len() { break; }
                let b = data[*pos];
                *pos += 1;
                facebits |= ((b & 0x7F) as u32) << bit_pos;
                if b & 0x80 == 0 { break; }
                bit_pos += 7;
            }
            facebits
        }

        fn skip_section_u8(data: &[u8], pos: &mut usize, te: &mut TextureEntryData, setter: fn(&mut TextureEntryFace, u8)) {
            if *pos >= data.len() { return; }
            let default_val = data[*pos];
            *pos += 1;
            setter(&mut te.default_face, default_val);
            while *pos < data.len() && data[*pos] != 0 {
                let bits = read_bitfield(data, pos);
                if *pos >= data.len() { break; }
                let val = data[*pos];
                *pos += 1;
                for i in 0..MAX_FACES {
                    if bits & (1 << i) != 0 {
                        if te.faces[i].is_none() {
                            te.faces[i] = Some(te.default_face.clone());
                        }
                        setter(te.faces[i].as_mut().unwrap(), val);
                    }
                }
            }
            if *pos < data.len() { *pos += 1; }
        }

        te.default_face.texture_id = read_uuid(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let uuid = read_uuid(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().texture_id = uuid;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.color = read_color4(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let color = read_color4(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().color = color;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.repeat_u = read_f32(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let val = read_f32(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().repeat_u = val;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.repeat_v = read_f32(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let val = read_f32(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().repeat_v = val;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.offset_u = read_i16_as_f32(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let val = read_i16_as_f32(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().offset_u = val;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.offset_v = read_i16_as_f32(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let val = read_i16_as_f32(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().offset_v = val;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        te.default_face.rotation = read_i16_as_f32(data, &mut pos);
        while pos < data.len() && data[pos] != 0 {
            let bits = read_bitfield(data, &mut pos);
            let val = read_i16_as_f32(data, &mut pos);
            for i in 0..MAX_FACES {
                if bits & (1 << i) != 0 {
                    if te.faces[i].is_none() {
                        te.faces[i] = Some(te.default_face.clone());
                    }
                    te.faces[i].as_mut().unwrap().rotation = val;
                }
            }
        }
        if pos < data.len() { pos += 1; }

        skip_section_u8(data, &mut pos, &mut te, |f, v| f.material = v);
        skip_section_u8(data, &mut pos, &mut te, |f, v| f.media = v);
        skip_section_u8(data, &mut pos, &mut te, |f, v| f.glow = v);

        te
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut te = Vec::with_capacity(128);

        fn write_bitfield(te: &mut Vec<u8>, bits: u32) {
            let mut val = bits;
            loop {
                let b = (val & 0x7F) as u8;
                val >>= 7;
                if val > 0 {
                    te.push(b | 0x80);
                } else {
                    te.push(b);
                    break;
                }
            }
        }

        fn f32_to_i16_fixed(v: f32) -> i16 {
            (v / std::f32::consts::PI * 32768.0).clamp(-32768.0, 32767.0) as i16
        }

        fn color_to_inverted(c: [f32; 4]) -> [u8; 4] {
            [
                255u8.saturating_sub((c[0].clamp(0.0, 1.0) * 255.0) as u8),
                255u8.saturating_sub((c[1].clamp(0.0, 1.0) * 255.0) as u8),
                255u8.saturating_sub((c[2].clamp(0.0, 1.0) * 255.0) as u8),
                255u8.saturating_sub((c[3].clamp(0.0, 1.0) * 255.0) as u8),
            ]
        }

        te.extend_from_slice(self.default_face.texture_id.as_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.texture_id != self.default_face.texture_id {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(f.texture_id.as_bytes());
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&color_to_inverted(self.default_face.color));
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.color != self.default_face.color {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&color_to_inverted(f.color));
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&self.default_face.repeat_u.to_le_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.repeat_u != self.default_face.repeat_u {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&f.repeat_u.to_le_bytes());
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&self.default_face.repeat_v.to_le_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.repeat_v != self.default_face.repeat_v {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&f.repeat_v.to_le_bytes());
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&f32_to_i16_fixed(self.default_face.offset_u).to_le_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.offset_u != self.default_face.offset_u {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&f32_to_i16_fixed(f.offset_u).to_le_bytes());
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&f32_to_i16_fixed(self.default_face.offset_v).to_le_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.offset_v != self.default_face.offset_v {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&f32_to_i16_fixed(f.offset_v).to_le_bytes());
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&f32_to_i16_fixed(self.default_face.rotation).to_le_bytes());
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.rotation != self.default_face.rotation {
                    write_bitfield(&mut te, 1 << i);
                    te.extend_from_slice(&f32_to_i16_fixed(f.rotation).to_le_bytes());
                }
            }
        }
        te.push(0);

        te.push(self.default_face.material);
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.material != self.default_face.material {
                    write_bitfield(&mut te, 1 << i);
                    te.push(f.material);
                }
            }
        }
        te.push(0);

        te.push(self.default_face.media);
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.media != self.default_face.media {
                    write_bitfield(&mut te, 1 << i);
                    te.push(f.media);
                }
            }
        }
        te.push(0);

        te.push(self.default_face.glow);
        for i in 0..MAX_FACES {
            if let Some(ref f) = self.faces[i] {
                if f.glow != self.default_face.glow {
                    write_bitfield(&mut te, 1 << i);
                    te.push(f.glow);
                }
            }
        }
        te.push(0);

        te.extend_from_slice(&[0u8; 16]);
        te.push(0);

        te
    }
}

pub fn modify_te_color(existing_te: &[u8], face: i32, r: f32, g: f32, b: f32) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_color(face, r, g, b);
    ted.to_bytes()
}

pub fn modify_te_alpha(existing_te: &[u8], face: i32, alpha: f32) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_alpha(face, alpha);
    ted.to_bytes()
}

pub fn modify_te_texture(existing_te: &[u8], face: i32, texture_id: Uuid) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_texture(face, texture_id);
    ted.to_bytes()
}

pub fn modify_te_scale(existing_te: &[u8], face: i32, u: f32, v: f32) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_scale(face, u, v);
    ted.to_bytes()
}

pub fn modify_te_offset(existing_te: &[u8], face: i32, u: f32, v: f32) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_offset(face, u, v);
    ted.to_bytes()
}

pub fn modify_te_rotation(existing_te: &[u8], face: i32, rotation: f32) -> Vec<u8> {
    let mut ted = TextureEntryData::parse(existing_te);
    ted.set_rotation(face, rotation);
    ted.to_bytes()
}

pub fn build_texture_anim(mode: i32, face: i32, size_x: i32, size_y: i32, start: f32, length: f32, rate: f32) -> Vec<u8> {
    let mut anim = Vec::with_capacity(16);
    anim.extend_from_slice(&(mode as u32).to_le_bytes());
    anim.extend_from_slice(&(face as i8).to_le_bytes());
    anim.push(size_x as u8);
    anim.push(size_y as u8);
    anim.push(0);
    anim.extend_from_slice(&start.to_le_bytes());
    anim.extend_from_slice(&length.to_le_bytes());
    anim.extend_from_slice(&rate.to_le_bytes());
    anim
}

pub const FLAGS_OBJECT_MODIFY: u32 = 0x00000004;
pub const FLAGS_OBJECT_COPY: u32 = 0x00000008;
pub const FLAGS_OBJECT_ANY_OWNER: u32 = 0x00000010;
pub const FLAGS_OBJECT_YOU_OWNER: u32 = 0x00000020;
pub const FLAGS_OBJECT_MOVE: u32 = 0x00000100;
pub const FLAGS_OBJECT_TRANSFER: u32 = 0x00020000;
pub const FLAGS_OBJECT_OWNER_MODIFY: u32 = 0x10000000;
pub const FLAGS_CHARACTER: u32 = 1 << 13;
pub const FLAGS_AVATAR: u32 = FLAGS_CHARACTER | FLAGS_OBJECT_YOU_OWNER;

pub const PERM_MASK_TRANSFER: u32 = 1 << 13;
pub const PERM_MASK_MODIFY: u32 = 1 << 14;
pub const PERM_MASK_COPY: u32 = 1 << 15;
pub const PERM_MASK_MOVE: u32 = 1 << 19;

pub fn compute_owner_update_flags(base_flags: u32, owner_mask: u32) -> u32 {
    let mut flags = base_flags;
    flags |= FLAGS_OBJECT_YOU_OWNER | FLAGS_OBJECT_ANY_OWNER;
    if owner_mask & PERM_MASK_COPY != 0 { flags |= FLAGS_OBJECT_COPY; }
    if owner_mask & PERM_MASK_MOVE != 0 { flags |= FLAGS_OBJECT_MOVE; }
    if owner_mask & PERM_MASK_MODIFY != 0 {
        flags |= FLAGS_OBJECT_MODIFY;
        flags |= FLAGS_OBJECT_OWNER_MODIFY;
    }
    if owner_mask & PERM_MASK_TRANSFER != 0 { flags |= FLAGS_OBJECT_TRANSFER; }
    flags
}

pub fn compute_nonowner_update_flags(base_flags: u32, everyone_mask: u32) -> u32 {
    let mut flags = base_flags;
    flags |= FLAGS_OBJECT_ANY_OWNER;
    flags &= !FLAGS_OBJECT_YOU_OWNER;
    if everyone_mask & PERM_MASK_COPY != 0 { flags |= FLAGS_OBJECT_COPY; }
    if everyone_mask & PERM_MASK_MOVE != 0 { flags |= FLAGS_OBJECT_MOVE; }
    if everyone_mask & PERM_MASK_MODIFY != 0 { flags |= FLAGS_OBJECT_MODIFY; }
    if everyone_mask & PERM_MASK_TRANSFER != 0 { flags |= FLAGS_OBJECT_TRANSFER; }
    flags
}

pub fn apply_viewer_flags(prim_data: &mut AvatarObjectData, obj_owner_id: uuid::Uuid, viewer_agent_id: uuid::Uuid, everyone_mask: u32) {
    if viewer_agent_id != obj_owner_id {
        let base_flags = prim_data.update_flags & !(
            FLAGS_OBJECT_YOU_OWNER | FLAGS_OBJECT_ANY_OWNER |
            FLAGS_OBJECT_COPY | FLAGS_OBJECT_MOVE |
            FLAGS_OBJECT_MODIFY | FLAGS_OBJECT_OWNER_MODIFY |
            FLAGS_OBJECT_TRANSFER
        );
        prim_data.update_flags = compute_nonowner_update_flags(base_flags, everyone_mask);
    }
}

#[derive(Debug, Clone)]
pub struct ObjectUpdateMessage {
    pub region_handle: u64,
    pub time_dilation: u16,
    pub objects: Vec<AvatarObjectData>,
}

#[derive(Debug, Clone)]
pub struct AvatarObjectData {
    pub local_id: u32,
    pub state: u8,
    pub full_id: Uuid,
    pub crc: u32,
    pub pcode: u8,
    pub material: u8,
    pub click_action: u8,
    pub scale: [f32; 3],
    pub collision_plane: [f32; 4],
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub rotation: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub parent_id: u32,
    pub update_flags: u32,
    pub path_curve: u8,
    pub profile_curve: u8,
    pub path_begin: u16,
    pub path_end: u16,
    pub path_scale_x: u8,
    pub path_scale_y: u8,
    pub path_shear_x: u8,
    pub path_shear_y: u8,
    pub path_twist: i8,
    pub path_twist_begin: i8,
    pub path_radius_offset: i8,
    pub path_taper_x: i8,
    pub path_taper_y: i8,
    pub path_revolutions: u8,
    pub path_skew: i8,
    pub profile_begin: u16,
    pub profile_end: u16,
    pub profile_hollow: u16,
    pub texture_entry: Vec<u8>,
    pub texture_anim: Vec<u8>,
    pub name_value: Vec<u8>,
    pub data: Vec<u8>,
    pub text: String,
    pub text_color: [u8; 4],
    pub media_url: String,
    pub ps_block: Vec<u8>,
    pub extra_params: Vec<u8>,
    pub sound_id: Uuid,
    pub sound_owner_id: Uuid,
    pub sound_gain: f32,
    pub sound_flags: u8,
    pub sound_radius: f32,
    pub joint_type: u8,
    pub joint_pivot: [f32; 3],
    pub joint_axis_or_anchor: [f32; 3],
}

impl AvatarObjectData {
    pub fn create_avatar(
        agent_id: Uuid,
        local_id: u32,
        position: [f32; 3],
        name: String,
    ) -> Self {
        let mut name_value = BytesMut::new();
        name_value.put_slice(b"FirstName STRING RW SV ");
        name_value.put_slice(name.split_whitespace().next().unwrap_or("Avatar").as_bytes());
        name_value.put_u8(b'\n');
        name_value.put_slice(b"LastName STRING RW SV ");
        name_value.put_slice(name.split_whitespace().nth(1).unwrap_or("Resident").as_bytes());
        name_value.put_u8(b'\n');
        name_value.put_slice(b"Title STRING RW SV \n");

        Self {
            local_id,
            state: 0,
            full_id: agent_id,
            crc: 0,
            pcode: PCODE_AVATAR,
            material: MATERIAL_FLESH,
            click_action: 0,
            scale: [0.45, 0.6, 1.9],
            collision_plane: [0.0, 0.0, 0.0, 1.0],
            position,
            velocity: [0.0, 0.0, 0.0],
            acceleration: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
            parent_id: 0,
            update_flags: 0, // OpenSim uses 0 for avatar update_flags (see LLClientView.cs:7282)
            // Phase 70.7: OpenSim non-zero-encoded version sets specific PBS values for avatars
            // (see LLClientView.cs:7209-7220 - CreateAvatarUpdateBlock non-zc version)
            path_curve: 16,      // OpenSim: dest[pos++] = 16
            profile_curve: 1,    // OpenSim: dest[pos++] = 1
            path_begin: 0,
            path_end: 0,
            path_scale_x: 100,   // OpenSim: dest[pos++] = 100
            path_scale_y: 100,   // OpenSim: dest[pos++] = 100
            path_shear_x: 0,
            path_shear_y: 0,
            path_twist: 0,
            path_twist_begin: 0,
            path_radius_offset: 0,
            path_taper_x: 0,
            path_taper_y: 0,
            path_revolutions: 0,
            path_skew: 0,
            profile_begin: 0,
            profile_end: 0,
            profile_hollow: 0,
            texture_entry: build_ruth_avatar_texture_entry(),
            texture_anim: Vec::new(),
            name_value: name_value.to_vec(),
            data: Vec::new(),
            text: String::new(),
            text_color: [0, 0, 0, 0],
            media_url: String::new(),
            ps_block: Vec::new(),
            extra_params: Vec::new(),
            sound_id: Uuid::nil(),
            sound_owner_id: Uuid::nil(),
            sound_gain: 0.0,
            sound_flags: 0,
            sound_radius: 0.0,
            joint_type: 0,
            joint_pivot: [0.0, 0.0, 0.0],
            joint_axis_or_anchor: [0.0, 0.0, 0.0],
        }
    }
}

pub const PCODE_PRIM: u8 = 9;
pub const FLAGS_CREATE_SELECTED: u32 = 1 << 1;
pub const FLAGS_SCRIPTED: u32 = 1 << 6;
pub const FLAGS_HANDLE_TOUCH: u32 = 1 << 7;

impl AvatarObjectData {
    pub fn create_prim(
        prim_id: Uuid,
        owner_id: Uuid,
        local_id: u32,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
        pcode: u8,
        material: u8,
        path_curve: u8,
        profile_curve: u8,
        path_begin: u16,
        path_end: u16,
        path_scale_x: u8,
        path_scale_y: u8,
        path_shear_x: u8,
        path_shear_y: u8,
        flags: u32,
    ) -> Self {
        Self {
            local_id,
            state: 0,
            full_id: prim_id,
            crc: 0,
            pcode,
            material,
            click_action: 0,
            scale,
            collision_plane: [0.0, 0.0, 0.0, 0.0],
            position,
            velocity: [0.0, 0.0, 0.0],
            acceleration: [0.0, 0.0, 0.0],
            rotation: [rotation[0], rotation[1], rotation[2]],
            angular_velocity: [0.0, 0.0, 0.0],
            parent_id: 0,
            update_flags: compute_owner_update_flags(flags, 0x7FFFFFFF),
            path_curve,
            profile_curve,
            path_begin,
            path_end,
            path_scale_x,
            path_scale_y,
            path_shear_x,
            path_shear_y,
            path_twist: 0,
            path_twist_begin: 0,
            path_radius_offset: 0,
            path_taper_x: 0,
            path_taper_y: 0,
            path_revolutions: 0,
            path_skew: 0,
            profile_begin: 0,
            profile_end: 0,
            profile_hollow: 0,
            texture_entry: build_default_prim_texture_entry(),
            texture_anim: Vec::new(),
            name_value: Vec::new(),
            data: Vec::new(),
            text: String::new(),
            text_color: [0, 0, 0, 0],
            media_url: String::new(),
            ps_block: Vec::new(),
            extra_params: Vec::new(),
            sound_id: Uuid::nil(),
            sound_owner_id: owner_id,
            sound_gain: 0.0,
            sound_flags: 0,
            sound_radius: 0.0,
            joint_type: 0,
            joint_pivot: [0.0, 0.0, 0.0],
            joint_axis_or_anchor: [0.0, 0.0, 0.0],
        }
    }

    pub fn create_prim_from_scene_object(obj: &crate::udp::server::SceneObject, _region_handle: u64) -> Self {
        let [rx, ry, rz, rw] = obj.rotation;
        let norm = (rx * rx + ry * ry + rz * rz + rw * rw).sqrt();
        let (nx, ny, nz) = if norm > 0.0001 {
            let inv = 1.0 / norm;
            if rw >= 0.0 {
                (rx * inv, ry * inv, rz * inv)
            } else {
                (-rx * inv, -ry * inv, -rz * inv)
            }
        } else {
            (0.0, 0.0, 0.0)
        };

        let extra_params = if obj.extra_params.len() >= 2 {
            obj.extra_params.clone()
        } else {
            Vec::new()
        };

        let is_mesh = extra_params.len() >= 24
            && extra_params[23] == 5;

        let mut profile_curve = obj.profile_curve;
        let mut profile_begin = obj.profile_begin;
        let mut profile_hollow = obj.profile_hollow;
        let mut path_scale_y = obj.path_scale_y;
        if is_mesh {
            profile_curve &= 0x0f;
            if profile_begin == 1 { profile_begin = 9375; }
            if profile_hollow == 1 { profile_hollow = 27500; }
            if profile_curve == 0 && path_scale_y < 150 {
                path_scale_y = 150;
            }
        }

        let state = if obj.attachment_point > 0 {
            let ap = obj.attachment_point as u32 & 0xFF;
            ((ap >> 4) | (ap << 4)) as u8
        } else {
            0
        };

        Self {
            local_id: obj.local_id,
            state,
            full_id: obj.uuid,
            crc: 0,
            pcode: obj.pcode,
            material: obj.material,
            click_action: 0,
            scale: obj.scale,
            collision_plane: [0.0, 0.0, 0.0, 0.0],
            position: obj.position,
            velocity: [0.0, 0.0, 0.0],
            acceleration: [0.0, 0.0, 0.0],
            rotation: [nx, ny, nz],
            angular_velocity: [0.0, 0.0, 0.0],
            parent_id: obj.parent_id,
            update_flags: {
                let mut uf = compute_owner_update_flags(obj.flags, obj.owner_mask);
                if !obj.script_items.is_empty() {
                    uf |= FLAGS_SCRIPTED | FLAGS_HANDLE_TOUCH;
                }
                uf
            },
            path_curve: obj.path_curve,
            profile_curve,
            path_begin: obj.path_begin,
            path_end: obj.path_end,
            path_scale_x: obj.path_scale_x,
            path_scale_y,
            path_shear_x: obj.path_shear_x,
            path_shear_y: obj.path_shear_y,
            path_twist: obj.path_twist,
            path_twist_begin: obj.path_twist_begin,
            path_radius_offset: obj.path_radius_offset,
            path_taper_x: obj.path_taper_x,
            path_taper_y: obj.path_taper_y,
            path_revolutions: obj.path_revolutions,
            path_skew: obj.path_skew,
            profile_begin,
            profile_end: obj.profile_end,
            profile_hollow,
            texture_entry: obj.texture_entry.clone(),
            texture_anim: obj.texture_anim.clone(),
            name_value: if obj.attachment_point > 0 && obj.item_id != uuid::Uuid::nil() && obj.link_number <= 1 {
                format!("AttachItemID STRING RW SV {}\n", obj.item_id).into_bytes()
            } else {
                Vec::new()
            },
            data: Vec::new(),
            text: obj.text.clone(),
            text_color: [0, 0, 0, 0],
            media_url: String::new(),
            ps_block: obj.particle_system.clone(),
            extra_params,
            sound_id: Uuid::nil(),
            sound_owner_id: Uuid::nil(),
            sound_gain: 0.0,
            sound_flags: 0,
            sound_radius: 0.0,
            joint_type: 0,
            joint_pivot: [0.0, 0.0, 0.0],
            joint_axis_or_anchor: [0.0, 0.0, 0.0],
        }
    }
}

impl ObjectUpdateMessage {
    pub fn create_avatar_update(
        agent_id: Uuid,
        local_id: u32,
        position: [f32; 3],
        name: String,
        region_handle: u64,
    ) -> Self {
        Self {
            region_handle,
            time_dilation: 65535,
            objects: vec![AvatarObjectData::create_avatar(agent_id, local_id, position, name)],
        }
    }

    pub fn create_avatar_update_with_texture(
        agent_id: Uuid,
        local_id: u32,
        position: [f32; 3],
        name: String,
        region_handle: u64,
        texture_entry: Vec<u8>,
    ) -> Self {
        let mut obj = AvatarObjectData::create_avatar(agent_id, local_id, position, name);
        obj.texture_entry = texture_entry;
        Self {
            region_handle,
            time_dilation: 65535,
            objects: vec![obj],
        }
    }

    pub fn create_prim_update(
        prim_data: AvatarObjectData,
        region_handle: u64,
    ) -> Self {
        Self {
            region_handle,
            time_dilation: 65535,
            objects: vec![prim_data],
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(512);

        buffer.put_u64_le(self.region_handle);
        buffer.put_u16_le(self.time_dilation);
        buffer.put_u8(self.objects.len() as u8);

        for obj in &self.objects {
            buffer.put_u32_le(obj.local_id);
            buffer.put_u8(obj.state);
            buffer.put_slice(obj.full_id.as_bytes());
            buffer.put_u32_le(obj.crc);
            buffer.put_u8(obj.pcode);
            buffer.put_u8(obj.material);
            buffer.put_u8(obj.click_action);

            for &s in &obj.scale {
                buffer.put_f32_le(s);
            }

            if obj.pcode == PCODE_AVATAR {
                buffer.put_u8(76);
                for &cp in &obj.collision_plane {
                    buffer.put_f32_le(cp);
                }
            } else {
                buffer.put_u8(60);
            }
            for &p in &obj.position {
                buffer.put_f32_le(p);
            }
            for &v in &obj.velocity {
                buffer.put_f32_le(v);
            }
            for &a in &obj.acceleration {
                buffer.put_f32_le(a);
            }
            for &r in &obj.rotation[..3] {
                buffer.put_f32_le(r);
            }
            for &av in &obj.angular_velocity {
                buffer.put_f32_le(av);
            }

            buffer.put_u32_le(obj.parent_id);
            buffer.put_u32_le(obj.update_flags);

            buffer.put_u8(obj.path_curve);
            buffer.put_u8(obj.profile_curve);
            buffer.put_u16_le(obj.path_begin);
            buffer.put_u16_le(obj.path_end);
            buffer.put_u8(obj.path_scale_x);
            buffer.put_u8(obj.path_scale_y);
            buffer.put_u8(obj.path_shear_x);
            buffer.put_u8(obj.path_shear_y);
            buffer.put_i8(obj.path_twist);
            buffer.put_i8(obj.path_twist_begin);
            buffer.put_i8(obj.path_radius_offset);
            buffer.put_i8(obj.path_taper_x);
            buffer.put_i8(obj.path_taper_y);
            buffer.put_u8(obj.path_revolutions);
            buffer.put_i8(obj.path_skew);
            buffer.put_u16_le(obj.profile_begin);
            buffer.put_u16_le(obj.profile_end);
            buffer.put_u16_le(obj.profile_hollow);

            buffer.put_u16_le(obj.texture_entry.len() as u16);
            buffer.put_slice(&obj.texture_entry);

            buffer.put_u8(obj.texture_anim.len() as u8);
            if !obj.texture_anim.is_empty() {
                buffer.put_slice(&obj.texture_anim);
            }

            buffer.put_u16_le(obj.name_value.len() as u16);
            buffer.put_slice(&obj.name_value);

            buffer.put_u16_le(obj.data.len() as u16);
            if !obj.data.is_empty() {
                buffer.put_slice(&obj.data);
            }

            let text_bytes = obj.text.as_bytes();
            buffer.put_u8(text_bytes.len().min(255) as u8);
            if !text_bytes.is_empty() {
                buffer.put_slice(&text_bytes[..text_bytes.len().min(255)]);
            }

            buffer.put_slice(&obj.text_color);

            let media_url_bytes = obj.media_url.as_bytes();
            buffer.put_u8(media_url_bytes.len().min(255) as u8);
            if !media_url_bytes.is_empty() {
                buffer.put_slice(&media_url_bytes[..media_url_bytes.len().min(255)]);
            }

            buffer.put_u8(obj.ps_block.len() as u8);
            if !obj.ps_block.is_empty() {
                buffer.put_slice(&obj.ps_block);
            }

            buffer.put_u8(obj.extra_params.len() as u8);
            if !obj.extra_params.is_empty() {
                buffer.put_slice(&obj.extra_params);
            }

            buffer.put_slice(obj.sound_id.as_bytes());
            buffer.put_slice(obj.sound_owner_id.as_bytes());
            buffer.put_f32_le(obj.sound_gain);
            buffer.put_u8(obj.sound_flags);
            buffer.put_f32_le(obj.sound_radius);

            buffer.put_u8(obj.joint_type);
            for &jp in &obj.joint_pivot {
                buffer.put_f32_le(jp);
            }
            for &ja in &obj.joint_axis_or_anchor {
                buffer.put_f32_le(ja);
            }
        }

        buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_object_creation() {
        let agent_id = Uuid::new_v4();
        let local_id = 1;
        let position = [128.0, 128.0, 25.0];
        let name = "Test Avatar".to_string();

        let obj = AvatarObjectData::create_avatar(agent_id, local_id, position, name);

        assert_eq!(obj.full_id, agent_id);
        assert_eq!(obj.local_id, local_id);
        assert_eq!(obj.pcode, PCODE_AVATAR);
        assert_eq!(obj.position, position);
    }

    #[test]
    fn test_object_update_message() {
        let agent_id = Uuid::new_v4();
        let message = ObjectUpdateMessage::create_avatar_update(
            agent_id,
            1,
            [128.0, 128.0, 25.0],
            "Test Avatar".to_string(),
            0x0000010000000100,
        );

        assert_eq!(message.objects.len(), 1);
        assert_eq!(message.time_dilation, 65535);
    }

    #[test]
    fn test_serialization_format() {
        let agent_id = Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap();
        let message = ObjectUpdateMessage::create_avatar_update(
            agent_id,
            1,
            [128.0, 128.0, 25.0],
            "Test Avatar".to_string(),
            (256000_u64 << 32) | 256000_u64,
        );

        let serialized = message.serialize();

        assert!(serialized.len() > 150);
        assert_eq!(serialized[10], 1);
        assert_eq!(serialized[11], 1);
        assert_eq!(serialized[12], 0);
        assert_eq!(serialized[13], 0);
        assert_eq!(serialized[14], 0);
        assert_eq!(serialized[15], 0);
        assert_eq!(serialized[36], PCODE_AVATAR);
        assert_eq!(serialized[37], MATERIAL_FLESH);
        assert_eq!(serialized[51], 76);
    }
}
