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

fn mul_vec3(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

const MAX_DEPTH: u32 = 3;

/// Punto de entrada del raytracer para un solo rayo.
pub fn trace_ray(scene: &Scene, origin: Vector3, dir: Vector3, depth: u32) -> Vector3 {
    if depth > MAX_DEPTH {
        return Vector3::zero();
    }

    let mut closest_t = f32::MAX;
    let mut hit_cube = None;
    let mut hit_uv = (0.0, 0.0);

    for cube in &scene.cubes {
        if let Some((t, u, v)) = cube.intersect(origin, dir) {
            if t < closest_t {
                closest_t = t;
                hit_cube = Some(cube);
                hit_uv = (u, v);
            }
        }
    }

    match hit_cube {
        None => scene.skybox.sample(dir),
        Some(cube) => {
            // 1. Normal en el punto de impacto
            let hit_point = origin + dir * closest_t;
            let raw_normal = cube.normal_at(hit_point);
            let normal = if dir.dot(raw_normal) > 0.0 { -raw_normal } else { raw_normal };
            let (hit_u, hit_v) = hit_uv;

            // 2. Muestreo de color base con TextureManager::sample_uv usando las UV
            let base_color = if let Some(tex_path) = cube.material.texture {
                let sampled = scene.textures.sample_uv(tex_path, hit_u, hit_v);
                if scene.textures.has_texture(tex_path) {
                    sampled
                } else {
                    cube.material.albedo
                }
            } else {
                cube.material.albedo
            };

            let view_dir = -dir;
            // 2. Luz ambiental mínima (0.08 del albedo base) para que nunca quede 100% negra
            let ambient = base_color * 0.08;
            let mut diffuse_specular = Vector3::zero();

            // 3. Shadow rays hacia cada luz de scene.lights
            for light in &scene.lights {
                let light_vec = light.position - hit_point;
                let light_dist = light_vec.length();
                if light_dist < 1e-4 {
                    continue;
                }
                let light_dir = light_vec / light_dist;

                // Sombra propia: si la normal apunta en dirección opuesta a la luz, no aporta difuso/especular
                let n_dot_l = normal.dot(light_dir);
                if n_dot_l <= 0.0 {
                    continue;
                }

                // 1. Offset de 1e-3 a lo largo de la normal para evitar autointersección (shadow acne)
                let shadow_orig = hit_point + normal * 1e-3;
                let mut in_shadow = false;
                for occluder in &scene.cubes {
                    if let Some((t, _, _)) = occluder.intersect(shadow_orig, light_dir) {
                        if t > 1e-3 && t < (light_dist - 1e-3) {
                            in_shadow = true;
                            break;
                        }
                    }
                }

                // 3 & 4. Combinar difuso (Lambert, N·L) + especular (Blinn-Phong) con atenuación lineal
                if !in_shadow {
                    let attenuation = (light.intensity / (1.0 + 0.22 * light_dist)).max(0.0);

                    let diffuse = mul_vec3(base_color, light.color) * (n_dot_l * attenuation);

                    let mut specular = Vector3::zero();
                    if cube.material.specular > 0.0 {
                        let half_dir = (light_dir + view_dir).normalized();
                        let n_dot_h = normal.dot(half_dir).max(0.0);
                        let shininess = cube.material.specular.max(1.0);
                        let spec_factor = n_dot_h.powf(shininess);
                        let spec_strength = (cube.material.specular / 60.0).clamp(0.04, 0.6);
                        specular = light.color * (spec_factor * attenuation * spec_strength);
                    }

                    diffuse_specular += diffuse + specular;
                }
            }

            // 4. Sumar material.emission SIEMPRE al resultado final
            ambient + diffuse_specular + cube.material.emission
        }
    }
}
