use raylib::prelude::Vector3;

/// Personaje representado como un sprite plano (pixel art) en vez de voxeles.
/// Mucho más barato de intersectar que un conjunto de cubos, y mantiene
/// la estética 2D original de los sprites de FF6/Chrono Trigger.
pub struct Billboard {
    pub position: Vector3, // base del sprite (donde toca el piso)
    pub width: f32,
    pub height: f32,
    pub texture: &'static str, // debe tener canal alpha para recortar la silueta
}

impl Billboard {
    pub fn new(position: Vector3, width: f32, height: f32, texture: &'static str) -> Self {
        Billboard { position, width, height, texture }
    }

    /// Intersección rayo-plano, orientando el billboard para que su normal
    /// siempre apunte hacia el origen del rayo (efecto "always faces camera").
    /// Devuelve (t, u, v) donde u,v son coordenadas de textura en [0,1].
    pub fn intersect(&self, origin: Vector3, dir: Vector3, right: Vector3, up: Vector3) -> Option<(f32, f32, f32)> {
        let to_billboard = self.position - origin;
        let normal = to_billboard.normalized() * -1.0;
        // Proyectamos sobre el plano que contiene el billboard, usando right/up de cámara.
        let denom = normal.dot(dir);
        if denom.abs() < 1e-6 {
            return None;
        }

        let center = self.position + up * (self.height * 0.5);
        let t = (center - origin).dot(normal) / denom;
        if t < 0.0 {
            return None;
        }

        let hit = origin + dir * t;
        let local = hit - center;
        let u = local.dot(right) / self.width + 0.5;
        let v = 1.0 - (local.dot(up) / self.height + 0.5);

        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
            Some((t, u, v))
        } else {
            None
        }
    }
}
