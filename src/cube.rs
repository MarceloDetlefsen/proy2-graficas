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
}

impl Cube {
    pub fn new(position: Vector3, size: f32, material: Material) -> Self {
        Cube {
            min: position,
            max: position + Vector3::new(size, size, size),
            material,
        }
    }

    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }

    /// Intersección rayo-AABB (slab method). Devuelve la distancia t
    /// del punto de impacto más cercano, si existe.
    pub fn intersect(&self, origin: Vector3, dir: Vector3) -> Option<f32> {
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
        // t_max no se usa después de este punto, pero se deja el cálculo
        // por si luego se necesita para transparencia (entrar y salir del cubo).
        let _ = tz_max;

        if t_min < 0.0 {
            None
        } else {
            Some(t_min)
        }
    }

    /// Normal de la cara golpeada en el punto `hit_point`.
    /// Necesaria para reflexión, refracción y mapas normales.
    pub fn normal_at(&self, hit_point: Vector3) -> Vector3 {
        const EPS: f32 = 1e-4;
        if (hit_point.x - self.min.x).abs() < EPS {
            Vector3::new(-1.0, 0.0, 0.0)
        } else if (hit_point.x - self.max.x).abs() < EPS {
            Vector3::new(1.0, 0.0, 0.0)
        } else if (hit_point.y - self.min.y).abs() < EPS {
            Vector3::new(0.0, -1.0, 0.0)
        } else if (hit_point.y - self.max.y).abs() < EPS {
            Vector3::new(0.0, 1.0, 0.0)
        } else if (hit_point.z - self.min.z).abs() < EPS {
            Vector3::new(0.0, 0.0, -1.0)
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        }
    }
}
