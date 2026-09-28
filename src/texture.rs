use raylib::prelude::*;
use std::collections::HashMap;

struct CpuTexture {
    width: i32,
    height: i32,
    pixels: Vec<Vector3>, // Valores RGB normalizados [0,1], usados por el raytracer en CPU
    alphas: Vec<f32>,     // Canal alfa normalizado [0,1] para recortes de sprites
}

impl CpuTexture {
    fn from_image(image: &Image) -> Self {
        let colors = image.get_image_data(); // Vec<Color>
        let mut pixels = Vec::with_capacity(colors.len());
        let mut alphas = Vec::with_capacity(colors.len());
        for c in colors.iter() {
            pixels.push(Vector3::new(
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
            ));
            alphas.push(c.a as f32 / 255.0);
        }

        CpuTexture {
            width: image.width,
            height: image.height,
            pixels,
            alphas,
        }
    }
}

pub struct TextureManager {
    cpu_textures: HashMap<String, CpuTexture>,
    textures: HashMap<String, Texture2D>, // Copias GPU solo para debug/preview, no para el raytrace
}

impl TextureManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_texture(&self, path: &str) -> bool {
        self.cpu_textures.contains_key(path)
    }

    pub fn load_cpu_texture(&mut self, path: &str) -> bool {
        if self.cpu_textures.contains_key(path) {
            return true;
        }
        if let Ok(image) = Image::load_image(path) {
            let cpu_texture = CpuTexture::from_image(&image);
            self.cpu_textures.insert(path.to_string(), cpu_texture);
            true
        } else {
            false
        }
    }

    pub fn load_texture(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, path: &str) {
        if self.textures.contains_key(path) {
            return;
        }

        let image = Image::load_image(path)
            .unwrap_or_else(|_| panic!("Failed to load image {}", path));

        let texture = rl
            .load_texture_from_image(thread, &image)
            .unwrap_or_else(|_| panic!("Failed to load texture {}", path));

        let cpu_texture = CpuTexture::from_image(&image);

        self.cpu_textures.insert(path.to_string(), cpu_texture);
        self.textures.insert(path.to_string(), texture);
    }

    /// Color de textura en coordenadas de pixel (u,v ya convertidos afuera).
    pub fn get_pixel_color(&self, path: &str, tx: u32, ty: u32) -> Vector3 {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            let x = tx.min(cpu_texture.width as u32 - 1) as i32;
            let y = ty.min(cpu_texture.height as u32 - 1) as i32;

            if x < 0 || y < 0 || x >= cpu_texture.width || y >= cpu_texture.height {
                return Vector3::one();
            }

            let index = (y * cpu_texture.width + x) as usize;
            cpu_texture.pixels.get(index).copied().unwrap_or(Vector3::one())
        } else {
            Vector3::one() // blanco por defecto si la textura no está cargada
        }
    }

    /// Igual que get_pixel_color pero recibe coordenadas UV normalizadas [0,1],
    /// con muestreo de vecino más cercano para conservar el aspecto pixel-art.
    pub fn sample_uv(&self, path: &str, u: f32, v: f32) -> Vector3 {
        self.sample_uv_rgba(path, u, v).0
    }

    /// Muestra color RGB y canal Alfa [0, 1] en coordenadas UV usando vecino más cercano (nearest-neighbor)
    /// sin interpolación para conservar la estética pixel-art nítida.
    pub fn sample_uv_rgba(&self, path: &str, u: f32, v: f32) -> (Vector3, f32) {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            let u_clamped = if u >= 0.0 && u < 1.0 { u } else { u.rem_euclid(1.0).clamp(0.0, 0.99999) };
            let v_clamped = if v >= 0.0 && v < 1.0 { v } else { v.rem_euclid(1.0).clamp(0.0, 0.99999) };
            let x = (u_clamped * cpu_texture.width as f32) as i32;
            let y = (v_clamped * cpu_texture.height as f32) as i32;

            let x = x.clamp(0, cpu_texture.width - 1);
            let y = y.clamp(0, cpu_texture.height - 1);
            let idx = (y * cpu_texture.width + x) as usize;
            let rgb = cpu_texture.pixels.get(idx).copied().unwrap_or(Vector3::one());
            let alpha = cpu_texture.alphas.get(idx).copied().unwrap_or(1.0);
            (rgb, alpha)
        } else {
            (Vector3::one(), 1.0)
        }
    }

    /// Alias retrocompatible de sample_uv_rgba
    pub fn sample_rgba(&self, path: &str, u: f32, v: f32) -> (Vector3, f32) {
        self.sample_uv_rgba(path, u, v)
    }

    pub fn get_texture(&self, path: &str) -> Option<&Texture2D> {
        self.textures.get(path)
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        TextureManager {
            cpu_textures: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}
