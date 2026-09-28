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
        Some((*incident * eta + n * (eta * cosi - k.sqrt())).normalized())
    }
}

/// Reflexión especular estándar: R = I - 2*(I·N)*N
pub fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    (*incident - *normal * 2.0 * incident.dot(*normal)).normalized()
}

fn mul_vec3(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

const MAX_DEPTH: u32 = 4;

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
            let hit_point = origin + dir * closest_t;
            let raw_normal = cube.normal_at(hit_point);
            let is_inside = dir.dot(raw_normal) > 0.0;
            let normal = if is_inside { -raw_normal } else { raw_normal };

            // Si el rayo viene desde adentro de un objeto transparente (saliendo del material)
            if is_inside && cube.material.transparency > 0.0 {
                if depth < MAX_DEPTH {
                    let refr_ray = refract(&dir, &raw_normal, cube.material.refractive_index);
                    match refr_ray {
                        Some(exit_dir) => {
                            // Salida al aire: normal apunta hacia adentro, así que -normal apunta hacia afuera
                            let exit_orig = hit_point - normal * 1e-3;
                            return trace_ray(scene, exit_orig, exit_dir, depth + 1);
                        }
                        None => {
                            // Reflexión interna total dentro del cubo
                            let refl_dir = reflect(&dir, &normal);
                            let refl_orig = hit_point + normal * 1e-3;
                            return trace_ray(scene, refl_orig, refl_dir, depth + 1);
                        }
                    }
                } else {
                    return scene.skybox.sample(dir);
                }
            }

            let (hit_u, hit_v) = hit_uv;

            // Muestreo de color base con TextureManager::sample_uv usando las UV
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
            // Luz ambiental mínima
            let ambient = base_color * 0.12;
            let mut diffuse_specular = Vector3::zero();

            // Shadow rays hacia cada luz de scene.lights
            for light in &scene.lights {
                let light_vec = light.position - hit_point;
                let light_dist = light_vec.length();
                if light_dist < 1e-4 {
                    continue;
                }
                let light_dir = light_vec / light_dist;

                let n_dot_l = normal.dot(light_dir);
                if n_dot_l <= 0.0 {
                    continue;
                }

                let shadow_orig = hit_point + normal * 1e-3;
                let mut in_shadow = false;
                for occluder in &scene.cubes {
                    // Cubos casi totalmente transparentes dejan pasar la luz
                    if occluder.material.transparency > 0.7 {
                        continue;
                    }
                    if let Some((t, _, _)) = occluder.intersect(shadow_orig, light_dir) {
                        if t > 1e-3 && t < (light_dist - 1e-3) {
                            in_shadow = true;
                            break;
                        }
                    }
                }

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

            let local_shading = ambient + diffuse_specular;
            let mut final_color = local_shading;

            // 1 & 2. Refracción y reflexión recursivas
            if cube.material.transparency > 0.0 && depth < MAX_DEPTH {
                let refr_ray = refract(&dir, &raw_normal, cube.material.refractive_index);
                let trans_color = match refr_ray {
                    Some(refr_dir) => {
                        let refr_orig = hit_point - normal * 1e-3;
                        let c = trace_ray(scene, refr_orig, refr_dir, depth + 1);
                        let tint = base_color * 0.6 + Vector3::one() * 0.4;
                        mul_vec3(c, tint)
                    }
                    None => {
                        // Reflexión interna total
                        let refl_dir = reflect(&dir, &normal);
                        let refl_orig = hit_point + normal * 1e-3;
                        trace_ray(scene, refl_orig, refl_dir, depth + 1)
                    }
                };
                final_color = final_color * (1.0 - cube.material.transparency)
                    + trans_color * cube.material.transparency;
            }

            if cube.material.reflectivity > 0.0 && depth < MAX_DEPTH {
                let refl_dir = reflect(&dir, &normal);
                let refl_orig = hit_point + normal * 1e-3;
                let refl_color = trace_ray(scene, refl_orig, refl_dir, depth + 1);
                final_color = final_color * (1.0 - cube.material.reflectivity)
                    + refl_color * cube.material.reflectivity;
            }

            final_color + cube.material.emission
        }
    }
}
