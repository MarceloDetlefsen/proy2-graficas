use raylib::prelude::Vector3;

/// Personaje representado como un sprite plano (pixel art) en vez de voxeles.
/// Mucho más barato de intersectar que un conjunto de cubos, y mantiene
/// la estética 2D original de los sprites de FF6/Chrono Trigger.
#[derive(Clone, Copy)]
pub struct Billboard {
    pub position: Vector3, // base del sprite (donde toca el piso, centro horizontal)
    pub width: f32,
    pub height: f32,
    pub texture: &'static str, // debe tener canal alpha para recortar la silueta
}

impl Billboard {
    pub fn new(position: Vector3, width: f32, height: f32, texture: &'static str) -> Self {
        Billboard { position, width, height, texture }
    }

    /// Intersección rayo-plano con orientación de cámara fija por frame.
    /// normal = -forward, u y v calculados con right y up.
    /// Devuelve (t, u, v) donde u,v son coordenadas de textura en [0,1].
    pub fn intersect(
        &self,
        origin: Vector3,
        dir: Vector3,
        forward: Vector3,
        right: Vector3,
        up: Vector3,
    ) -> Option<(f32, f32, f32)> {
        let normal = -forward;
        let denom = normal.dot(dir);
        if denom.abs() < 1e-6 {
            return None;
        }

        // El centro del billboard está a mitad de altura por encima de los pies
        let center = self.position + up * (self.height * 0.5);
        let t = (center - origin).dot(normal) / denom;
        if t < 0.0 {
            return None;
        }

        let hit = origin + dir * t;
        let local = hit - center;
        let u = local.dot(right) / self.width + 0.5;
        let v = 0.5 - local.dot(up) / self.height;

        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
            Some((t, u, v))
        } else {
            None
        }
    }
}
