use raylib::prelude::Vector3;

pub struct Light {
    pub position: Vector3,
    pub color: Vector3,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vector3, color: Vector3, intensity: f32) -> Self {
        Light { position, color, intensity }
    }

    /// Luz cálida y parpadeante para colocar justo sobre los bloques de fogata.
    /// La fogata es al mismo tiempo geometría emisiva (se ve brillante directo
    /// a cámara) y fuente de luz real (ilumina el resto de la escena de noche).
    pub fn campfire(position: Vector3) -> Self {
        Light::new(position, Vector3::new(1.0, 0.6, 0.25), 6.0)
    }
}
