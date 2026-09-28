use raylib::prelude::Vector3;
use crate::scene::Scene;

/// Snell's Law: calcula la dirección del rayo refractado al pasar de un medio a otro.
/// Devuelve None si ocurre reflexión interna total.
pub fn refract(incident: &Vector3, normal: &Vector3, refractive_index: f32) -> Option<Vector3> {
    let mut cosi = incident.dot(*normal).max(-1.0).min(1.0);

    let mut etai = 1.0; // Aire/vacío
    let mut etat = refractive_index;
    let mut n = *normal;

    if cosi > 0.0 {
        // El rayo va de adentro del material hacia afuera: invertir índices y normal.
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    } else {
        cosi = -cosi;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        None // Reflexión interna total
    } else {
        Some(*incident * eta + n * (eta * cosi - k.sqrt()))
    }
}

/// Reflexión especular estándar: R = I - 2*(I·N)*N
pub fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    *incident - *normal * 2.0 * incident.dot(*normal)
}

const MAX_DEPTH: u32 = 3;

/// Punto de entrada del raytracer para un solo rayo. Por ahora resuelve
/// intersección con cubos + skybox; sombras/reflexión/refracción recursiva
/// y billboards se agregan en el siguiente paso.
pub fn trace_ray(scene: &Scene, origin: Vector3, dir: Vector3, depth: u32) -> Vector3 {
    if depth > MAX_DEPTH {
        return Vector3::zero();
    }

    let mut closest_t = f32::MAX;
    let mut hit_cube = None;

    for cube in &scene.cubes {
        if let Some((t, _, _)) = cube.intersect(origin, dir) {
            if t < closest_t {
                closest_t = t;
                hit_cube = Some(cube);
            }
        }
    }

    match hit_cube {
        None => scene.skybox.sample(dir),
        Some(cube) => {
            // TODO: aplicar textura via UV, mapa normal, sombras hacia scene.lights,
            // mezclar con reflect()/refract() según material.reflectivity / transparency,
            // y sumar material.emission si es emisivo.
            let _hit_point = origin + dir * closest_t;
            cube.material.albedo
        }
    }
}
