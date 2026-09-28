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

/// Personaje o sprite acostado horizontalmente sobre el suelo (y = suelo + 0.03).
/// La intersección es rayo-plano horizontal con coordenadas locales rotadas por `angle_y`.
/// Cabeza hacia el fuego (+head_dir), pies hacia afuera (-head_dir).
#[derive(Clone, Copy)]
pub struct GroundSprite {
    pub position: Vector3, // Centro del cuadrilátero sobre el suelo
    pub width: f32,        // Ancho (eje transversal del sprite)
    pub length: f32,       // Largo en el suelo = altura normal del sprite
    pub angle_y: f32,      // Ángulo de orientación de la cabeza en el plano XZ (radianes)
    pub texture: &'static str,
}

impl GroundSprite {
    pub fn new(position: Vector3, width: f32, length: f32, angle_y: f32, texture: &'static str) -> Self {
        GroundSprite {
            position,
            width,
            length,
            angle_y,
            texture,
        }
    }

    /// Intersección rayo-plano horizontal a Y = position.y con coordenadas locales rotadas.
    /// Devuelve (t, u, v) con u,v en [0, 1].
    pub fn intersect(&self, origin: Vector3, dir: Vector3) -> Option<(f32, f32, f32)> {
        // Normal del plano horizontal apuntando hacia arriba
        if dir.y.abs() < 1e-6 {
            return None;
        }

        let t = (self.position.y - origin.y) / dir.y;
        if t < 1e-4 {
            return None;
        }

        let hit = origin + dir * t;
        let dx = hit.x - self.position.x;
        let dz = hit.z - self.position.z;

        // Dirección de la cabeza en el plano horizontal (hacia el fuego)
        let sin_a = self.angle_y.sin();
        let cos_a = self.angle_y.cos();
        let head_dir_x = sin_a;
        let head_dir_z = cos_a;

        // Eje derecho perpendicular: (head_dir_z, 0, -head_dir_x)
        let right_dir_x = head_dir_z;
        let right_dir_z = -head_dir_x;

        let dist_right = dx * right_dir_x + dz * right_dir_z;
        let dist_head = dx * head_dir_x + dz * head_dir_z;

        let u = (dist_right / self.width) + 0.5;
        // v = 0 en la cabeza (+dist_head), v = 1 en los pies (-dist_head)
        let v = 0.5 - (dist_head / self.length);

        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
            Some((t, u, v))
        } else {
            None
        }
    }
}
