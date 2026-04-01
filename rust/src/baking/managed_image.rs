use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageChannels(u8);

impl ImageChannels {
    pub const COLOR: ImageChannels = ImageChannels(1);
    pub const ALPHA: ImageChannels = ImageChannels(2);
    pub const BUMP: ImageChannels = ImageChannels(4);

    pub fn has_color(&self) -> bool {
        self.0 & Self::COLOR.0 != 0
    }

    pub fn has_alpha(&self) -> bool {
        self.0 & Self::ALPHA.0 != 0
    }

    pub fn has_bump(&self) -> bool {
        self.0 & Self::BUMP.0 != 0
    }
}

impl std::ops::BitOr for ImageChannels {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        ImageChannels(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ImageChannels {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        ImageChannels(self.0 & rhs.0)
    }
}

#[derive(Clone)]
pub struct ManagedImage {
    pub width: u32,
    pub height: u32,
    pub channels: ImageChannels,
    pub red: Vec<u8>,
    pub green: Vec<u8>,
    pub blue: Vec<u8>,
    pub alpha: Vec<u8>,
    pub bump: Vec<u8>,
}

impl ManagedImage {
    pub fn new(width: u32, height: u32, channels: ImageChannels) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            channels,
            red: if channels.has_color() { vec![0; size] } else { Vec::new() },
            green: if channels.has_color() { vec![0; size] } else { Vec::new() },
            blue: if channels.has_color() { vec![0; size] } else { Vec::new() },
            alpha: if channels.has_alpha() { vec![255; size] } else { Vec::new() },
            bump: if channels.has_bump() { vec![0; size] } else { Vec::new() },
        }
    }

    pub fn from_rgba(image: &RgbaImage) -> Self {
        let width = image.width();
        let height = image.height();
        let size = (width * height) as usize;

        let mut red = vec![0u8; size];
        let mut green = vec![0u8; size];
        let mut blue = vec![0u8; size];
        let mut alpha = vec![255u8; size];

        for (i, pixel) in image.pixels().enumerate() {
            red[i] = pixel[0];
            green[i] = pixel[1];
            blue[i] = pixel[2];
            alpha[i] = pixel[3];
        }

        Self {
            width,
            height,
            channels: ImageChannels::COLOR | ImageChannels::ALPHA,
            red,
            green,
            blue,
            alpha,
            bump: Vec::new(),
        }
    }

    pub fn from_dynamic_image(image: &DynamicImage) -> Self {
        Self::from_rgba(&image.to_rgba8())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory(data)?;
        Ok(Self::from_dynamic_image(&img))
    }

    pub fn to_rgba_image(&self) -> RgbaImage {
        let mut img = RgbaImage::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let i = (y * self.width + x) as usize;
                let r = if !self.red.is_empty() { self.red[i] } else { 0 };
                let g = if !self.green.is_empty() { self.green[i] } else { 0 };
                let b = if !self.blue.is_empty() { self.blue[i] } else { 0 };
                let a = if !self.alpha.is_empty() { self.alpha[i] } else { 255 };
                img.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }

        img
    }

    pub fn to_png_bytes(&self) -> Vec<u8> {
        let img = self.to_rgba_image();
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap_or_default();
        bytes
    }

    pub fn to_jpeg_bytes(&self, quality: u8) -> Vec<u8> {
        let img = self.to_rgba_image();
        let rgb_img = DynamicImage::ImageRgba8(img).to_rgb8();
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder.encode_image(&rgb_img).unwrap_or_default();
        bytes
    }

    pub fn resize_nearest_neighbor(&mut self, new_width: u32, new_height: u32) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let new_size = (new_width * new_height) as usize;
        let x_ratio = self.width as f64 / new_width as f64;
        let y_ratio = self.height as f64 / new_height as f64;

        let mut new_red = if self.channels.has_color() { vec![0u8; new_size] } else { Vec::new() };
        let mut new_green = if self.channels.has_color() { vec![0u8; new_size] } else { Vec::new() };
        let mut new_blue = if self.channels.has_color() { vec![0u8; new_size] } else { Vec::new() };
        let mut new_alpha = if self.channels.has_alpha() { vec![255u8; new_size] } else { Vec::new() };
        let mut new_bump = if self.channels.has_bump() { vec![0u8; new_size] } else { Vec::new() };

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f64 * x_ratio) as u32;
                let src_y = (y as f64 * y_ratio) as u32;
                let src_i = (src_y * self.width + src_x) as usize;
                let dst_i = (y * new_width + x) as usize;

                if self.channels.has_color() && src_i < self.red.len() {
                    new_red[dst_i] = self.red[src_i];
                    new_green[dst_i] = self.green[src_i];
                    new_blue[dst_i] = self.blue[src_i];
                }
                if self.channels.has_alpha() && src_i < self.alpha.len() {
                    new_alpha[dst_i] = self.alpha[src_i];
                }
                if self.channels.has_bump() && src_i < self.bump.len() {
                    new_bump[dst_i] = self.bump[src_i];
                }
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.red = new_red;
        self.green = new_green;
        self.blue = new_blue;
        self.alpha = new_alpha;
        self.bump = new_bump;
    }

    pub fn convert_channels(&mut self, new_channels: ImageChannels) {
        let size = (self.width * self.height) as usize;

        if new_channels.has_color() && !self.channels.has_color() {
            self.red = vec![0; size];
            self.green = vec![0; size];
            self.blue = vec![0; size];
        }

        if new_channels.has_alpha() && !self.channels.has_alpha() {
            self.alpha = vec![255; size];
        }

        if new_channels.has_bump() && !self.channels.has_bump() {
            self.bump = vec![0; size];
        }

        self.channels = new_channels;
    }

    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        if self.channels.has_color() {
            for i in 0..self.red.len() {
                self.red[i] = r;
                self.green[i] = g;
                self.blue[i] = b;
            }
        }
        if self.channels.has_alpha() {
            for i in 0..self.alpha.len() {
                self.alpha[i] = a;
            }
        }
    }

    pub fn fill_dithered(&mut self, r: f32, g: f32, b: f32) {
        let r_byte = (r.clamp(0.0, 1.0) * 255.0) as u8;
        let g_byte = (g.clamp(0.0, 1.0) * 255.0) as u8;
        let b_byte = (b.clamp(0.0, 1.0) * 255.0) as u8;

        let r_alt = if r_byte < 255 { r_byte + 1 } else { r_byte - 1 };
        let g_alt = if g_byte < 255 { g_byte + 1 } else { g_byte - 1 };
        let b_alt = if b_byte < 255 { b_byte + 1 } else { b_byte - 1 };

        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if ((x ^ y) & 0x10) == 0 {
                    if self.channels.has_color() {
                        self.red[i] = r_alt;
                        self.green[i] = g_byte;
                        self.blue[i] = b_byte;
                    }
                } else {
                    if self.channels.has_color() {
                        self.red[i] = r_byte;
                        self.green[i] = g_alt;
                        self.blue[i] = b_alt;
                    }
                }
                if self.channels.has_alpha() {
                    self.alpha[i] = 255;
                }
                if self.channels.has_bump() {
                    self.bump[i] = 0;
                }
                i += 1;
            }
        }
    }
}

impl std::fmt::Debug for ManagedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("channels", &self.channels)
            .field("has_color_data", &!self.red.is_empty())
            .field("has_alpha_data", &!self.alpha.is_empty())
            .field("has_bump_data", &!self.bump.is_empty())
            .finish()
    }
}
