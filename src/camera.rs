use raylib::prelude::*;

pub struct Camera {
    pub eye: Vector3,
    pub center: Vector3,
    pub world_up: Vector3,
    pub up: Vector3,
    pub forward: Vector3,
    pub right: Vector3,
    changed: bool,
}

impl Camera {
    pub fn new(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            world_up: up,
            up: Vector3::zero(),
            forward: Vector3::zero(),
            right: Vector3::zero(),
            changed: true,
        };
        camera.update_basis_vectors();
        camera
    }

    pub fn set_view(&mut self, eye: Vector3, center: Vector3) {
        self.eye = eye;
        self.center = center;
        self.update_basis_vectors();
    }

    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalized();
        // Usar siempre world_up fijo (0, 1, 0) para que el horizonte se mantenga recto y sin inclinación
        self.right = self.forward.cross(self.world_up).normalized();
        self.up = self.right.cross(self.forward).normalized();
        self.changed = true;
    }

    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_pos = self.eye - self.center;
        let radius = relative_pos.length();
        let current_yaw = relative_pos.z.atan2(relative_pos.x);
        let current_pitch = (relative_pos.y / radius).asin();

        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5); // Evita gimbal lock
        let cos_pitch = new_pitch.cos();
        let new_relative_pos = Vector3::new(
            radius * cos_pitch * new_yaw.cos(),
            radius * new_pitch.sin(),
            radius * cos_pitch * new_yaw.sin(),
        );
        self.eye = self.center + new_relative_pos;
        self.update_basis_vectors();
    }

    pub fn zoom(&mut self, amount: f32) {
        let forward = (self.center - self.eye).normalized();
        let relative_pos = self.eye - self.center;
        let dist = relative_pos.length();
        if dist - amount > 1.0 {
            self.eye += forward * amount;
            self.update_basis_vectors();
        }
    }

    /// Desplaza el punto de mira y la cámara en el plano horizontal (WASD) y vertical (QE)
    pub fn move_target(&mut self, forward_delta: f32, right_delta: f32, up_delta: f32, cubes: &[crate::cube::Cube]) {
        // Dirección hacia adelante proyectada en el plano horizontal XZ
        let mut forward_xz = Vector3::new(self.forward.x, 0.0, self.forward.z);
        if forward_xz.length() > 1e-4 {
            forward_xz = forward_xz.normalized();
        } else {
            forward_xz = Vector3::new(0.0, 0.0, -1.0);
        }

        let mut right_xz = Vector3::new(self.right.x, 0.0, self.right.z);
        if right_xz.length() > 1e-4 {
            right_xz = right_xz.normalized();
        } else {
            right_xz = Vector3::new(1.0, 0.0, 0.0);
        }

        let delta = forward_xz * forward_delta + right_xz * right_delta + Vector3::new(0.0, up_delta, 0.0);

        let new_center = self.center + delta;
        // Limitar punto de mira a los bordes del terreno ([-9.0, 9.0] para terreno 20x20)
        let clamped_center_x = new_center.x.clamp(-9.0, 9.0);
        let clamped_center_z = new_center.z.clamp(-9.0, 9.0);
        let clamped_center_y = new_center.y.clamp(0.5, 12.0);
        let effective_delta = Vector3::new(
            clamped_center_x - self.center.x,
            clamped_center_y - self.center.y,
            clamped_center_z - self.center.z,
        );

        self.center += effective_delta;
        self.eye += effective_delta;

        // Evitar que la cámara quede por debajo del suelo o dentro de un cubo
        self.resolve_collision(cubes);
        self.update_basis_vectors();
    }

    pub fn resolve_collision(&mut self, cubes: &[crate::cube::Cube]) {
        // Altura mínima del terreno debajo de eye
        let min_y = crate::scene::terrain_height_at(cubes, self.eye.x, self.eye.z) + 0.35;
        if self.eye.y < min_y {
            self.eye.y = min_y;
        }

        // Evitar penetración dentro de cubos opacos
        for c in cubes {
            if !c.casts_shadow {
                continue;
            }
            let pad = 0.20;
            if self.eye.x >= c.min.x - pad && self.eye.x <= c.max.x + pad
                && self.eye.y >= c.min.y - pad && self.eye.y <= c.max.y + pad
                && self.eye.z >= c.min.z - pad && self.eye.z <= c.max.z + pad
            {
                if self.eye.y < c.max.y + pad {
                    self.eye.y = c.max.y + pad;
                }
            }
        }
    }

    pub fn is_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    pub fn basis_change(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            v.x * self.right.x + v.y * self.up.x - v.z * self.forward.x,
            v.x * self.right.y + v.y * self.up.y - v.z * self.forward.y,
            v.x * self.right.z + v.y * self.up.z - v.z * self.forward.z,
        )
    }
}
