use raylib::prelude::Vector3;
use crate::material::Material;

/// Un cubo axis-aligned (AABB) con un material asignado.
/// `position` es la esquina mínima (no el centro), para que encaje
/// directo con la lógica de grid de `generate_terrain`.
#[derive(Clone, Copy)]
pub struct Cube {
    pub min: Vector3,
    pub max: Vector3,
    pub material: Material,
    pub tile_uv: bool,
    pub casts_shadow: bool,
    pub log_axis: Option<char>, // None: estándar, Some('X'): veta a lo largo de X, Some('Z'): veta a lo largo de Z
}

impl Cube {
    pub fn new(position: Vector3, size: f32, material: Material) -> Self {
        Cube {
            min: position,
            max: position + Vector3::new(size, size, size),
            material,
            tile_uv: true,
            casts_shadow: true,
            log_axis: None,
        }
    }

    pub fn new_box(position: Vector3, size: Vector3, material: Material) -> Self {
        Cube {
            min: position,
            max: position + size,
            material,
            tile_uv: true,
            casts_shadow: true,
            log_axis: None,
        }
    }

    pub fn with_tile_uv(mut self, tile: bool) -> Self {
        self.tile_uv = tile;
        self
    }

    pub fn without_shadow(mut self) -> Self {
        self.casts_shadow = false;
        self
    }

    pub fn with_log_axis(mut self, axis: char) -> Self {
        self.log_axis = Some(axis);
        self
    }

    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }

    /// Intersección rayo-AABB (slab method). Devuelve (t, u, v) donde t es
    /// la distancia del impacto más cercano y u, v son las coordenadas de textura
    /// en la cara golpeada en el rango [0, 1].
    pub fn intersect(&self, origin: Vector3, dir: Vector3) -> Option<(f32, f32, f32)> {
        self.intersect_with_uv(origin, dir)
    }

    /// Intersección con cálculo de UV según la cara golpeada.
    pub fn intersect_with_uv(&self, origin: Vector3, dir: Vector3) -> Option<(f32, f32, f32)> {
        let inv_dir = Vector3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);

        let mut t_min = (self.min.x - origin.x) * inv_dir.x;
        let mut t_max = (self.max.x - origin.x) * inv_dir.x;
        if inv_dir.x < 0.0 {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (self.min.y - origin.y) * inv_dir.y;
        let mut ty_max = (self.max.y - origin.y) * inv_dir.y;
        if inv_dir.y < 0.0 {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        if t_min > ty_max || ty_min > t_max {
            return None;
        }
        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        let mut tz_min = (self.min.z - origin.z) * inv_dir.z;
        let mut tz_max = (self.max.z - origin.z) * inv_dir.z;
        if inv_dir.z < 0.0 {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        if t_min > tz_max || tz_min > t_max {
            return None;
        }
        if tz_min > t_min {
            t_min = tz_min;
        }

        let _ = tz_max;

        if t_min < 0.0 {
            None
        } else {
            let hit_point = origin + dir * t_min;
            let (u, v) = self.uv_at(hit_point);
            Some((t_min, u, v))
        }
    }

    /// Coordenadas UV [0, 1] en la cara golpeada del cubo. Cada cara del cubo
    /// mapea su textura completa (no repetida) sobre dicha cara.
    pub fn uv_at(&self, hit_point: Vector3) -> (f32, f32) {
        let sx = (self.max.x - self.min.x).max(1e-6);
        let sy = (self.max.y - self.min.y).max(1e-6);
        let sz = (self.max.z - self.min.z).max(1e-6);

        let d_min_x = (hit_point.x - self.min.x).abs();
        let d_max_x = (hit_point.x - self.max.x).abs();
        let d_min_y = (hit_point.y - self.min.y).abs();
        let d_max_y = (hit_point.y - self.max.y).abs();
        let d_min_z = (hit_point.z - self.min.z).abs();
        let d_max_z = (hit_point.z - self.max.z).abs();

        let mut min_d = d_min_x;
        let mut face = 0; // 0: -X, 1: +X, 2: -Y, 3: +Y, 4: -Z, 5: +Z

        if d_max_x < min_d { min_d = d_max_x; face = 1; }
        if d_min_y < min_d { min_d = d_min_y; face = 2; }
        if d_max_y < min_d { min_d = d_max_y; face = 3; }
        if d_min_z < min_d { min_d = d_min_z; face = 4; }
        if d_max_z < min_d { face = 5; }

        let (u, v) = match self.log_axis {
            Some('X') => {
                // Eje largo en X: la veta (eje V) corre a lo largo de X en las 4 caras laterales
                match face {
                    0 => (hit_point.z - self.min.z, self.max.y - hit_point.y), // Tapa -X
                    1 => (self.max.z - hit_point.z, self.max.y - hit_point.y), // Tapa +X
                    2 => (hit_point.z - self.min.z, hit_point.x - self.min.x), // Lateral -Y
                    3 => (hit_point.z - self.min.z, hit_point.x - self.min.x), // Lateral +Y (superior)
                    4 => (self.max.y - hit_point.y, hit_point.x - self.min.x), // Lateral -Z (trasera)
                    _ => (self.max.y - hit_point.y, hit_point.x - self.min.x), // Lateral +Z (delantera)
                }
            }
            Some('Z') => {
                // Eje largo en Z: la veta (eje V) corre a lo largo de Z en las 4 caras laterales
                match face {
                    0 => (self.max.y - hit_point.y, hit_point.z - self.min.z), // Lateral -X
                    1 => (self.max.y - hit_point.y, hit_point.z - self.min.z), // Lateral +X
                    2 => (hit_point.x - self.min.x, hit_point.z - self.min.z), // Lateral -Y
                    3 => (hit_point.x - self.min.x, hit_point.z - self.min.z), // Lateral +Y (superior)
                    4 => (self.max.x - hit_point.x, self.max.y - hit_point.y), // Tapa -Z
                    _ => (hit_point.x - self.min.x, self.max.y - hit_point.y), // Tapa +Z
                }
            }
            _ => {
                if self.tile_uv {
                    match face {
                        0 => (hit_point.z - self.min.z, self.max.y - hit_point.y), // -X (izquierda)
                        1 => (self.max.z - hit_point.z, self.max.y - hit_point.y), // +X (derecha)
                        2 => (hit_point.x - self.min.x, self.max.z - hit_point.z), // -Y (abajo)
                        3 => (hit_point.x - self.min.x, hit_point.z - self.min.z), // +Y (arriba)
                        4 => (self.max.x - hit_point.x, self.max.y - hit_point.y), // -Z (atrás)
                        _ => (hit_point.x - self.min.x, self.max.y - hit_point.y), // +Z (frente)
                    }
                } else {
                    match face {
                        0 => ((hit_point.z - self.min.z) / sz, (self.max.y - hit_point.y) / sy), // -X (izquierda)
                        1 => ((self.max.z - hit_point.z) / sz, (self.max.y - hit_point.y) / sy), // +X (derecha)
                        2 => ((hit_point.x - self.min.x) / sx, (self.max.z - hit_point.z) / sz), // -Y (abajo)
                        3 => ((hit_point.x - self.min.x) / sx, (hit_point.z - self.min.z) / sz), // +Y (arriba)
                        4 => ((self.max.x - hit_point.x) / sx, (self.max.y - hit_point.y) / sy), // -Z (atrás)
                        _ => ((hit_point.x - self.min.x) / sx, (self.max.y - hit_point.y) / sy), // +Z (frente)
                    }
                }
            }
        };

        (u.rem_euclid(1.0).clamp(0.0, 0.99999), v.rem_euclid(1.0).clamp(0.0, 0.99999))
    }

    /// Devuelve la base ortonormal de la cara golpeada: (normal, tangent, bitangent)
    /// donde tangent es la dirección en la que crece U, y bitangent donde crece V,
    /// coincidiendo exactamente con el mapeo uv_at().
    pub fn tangent_frame_at(&self, hit_point: Vector3) -> (Vector3, Vector3, Vector3) {
        let d_min_x = (hit_point.x - self.min.x).abs();
        let d_max_x = (hit_point.x - self.max.x).abs();
        let d_min_y = (hit_point.y - self.min.y).abs();
        let d_max_y = (hit_point.y - self.max.y).abs();
        let d_min_z = (hit_point.z - self.min.z).abs();
        let d_max_z = (hit_point.z - self.max.z).abs();

        let mut min_d = d_min_x;
        let mut face = 0; // 0: -X, 1: +X, 2: -Y, 3: +Y, 4: -Z, 5: +Z

        if d_max_x < min_d { min_d = d_max_x; face = 1; }
        if d_min_y < min_d { min_d = d_min_y; face = 2; }
        if d_max_y < min_d { min_d = d_max_y; face = 3; }
        if d_min_z < min_d { min_d = d_min_z; face = 4; }
        if d_max_z < min_d { face = 5; }

        match self.log_axis {
            Some('X') => match face {
                0 => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, -1.0, 0.0)),
                1 => (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, -1.0, 0.0)),
                2 => (Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
                3 => (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
                4 => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
                _ => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
            },
            Some('Z') => match face {
                0 => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                1 => (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                2 => (Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                3 => (Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                4 => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
                _ => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
            },
            _ => match face {
                0 => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, -1.0, 0.0)),
                1 => (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, -1.0, 0.0)),
                2 => (Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
                3 => (Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                4 => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
                _ => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
            },
        }
    }

    /// Normal de la cara golpeada en el punto `hit_point`.
    /// Necesaria para reflexión, refracción y mapas normales.
    #[allow(dead_code)]
    pub fn normal_at(&self, hit_point: Vector3) -> Vector3 {
        self.tangent_frame_at(hit_point).0
    }
}
