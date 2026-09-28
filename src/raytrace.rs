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
    let mut hit_billboard: Option<(&crate::billboard::Billboard, Vector3, f32, f32)> = None;
    let mut hit_ground_sprite: Option<(&crate::billboard::GroundSprite, Vector3, f32, f32)> = None;

    // 1. Intersección con cubos acelerada mediante Grid 3D uniforme y DDA
    if let Some((t, cube_idx, u, v)) = scene.grid.intersect_closest(&scene.cubes, origin, dir) {
        if t < closest_t && t > 1e-4 {
            closest_t = t;
            hit_cube = Some(&scene.cubes[cube_idx]);
            hit_uv = (u, v);
            hit_billboard = None;
            hit_ground_sprite = None;
        }
    }

    // 2. Intersección con billboards (sprites planos orientados a cámara)
    for bb in &scene.billboards {
        if let Some((t, u, v)) = bb.intersect(origin, dir, scene.camera_forward, scene.camera_right, scene.camera_up, closest_t) {
            let (color, alpha) = scene.textures.sample_uv_rgba(bb.texture, u, v);
            // Si el texel tiene alpha < 0.5, el rayo pasa de largo
            if alpha >= 0.5 {
                closest_t = t;
                hit_billboard = Some((bb, color, u, v));
                hit_ground_sprite = None;
                hit_cube = None;
            }
        }
    }

    // 3. Intersección con ground_sprites (personajes acostados horizontalmente)
    for gs in &scene.ground_sprites {
        if let Some((t, u, v)) = gs.intersect(origin, dir) {
            if t < closest_t && t > 1e-4 {
                let (color, alpha) = scene.textures.sample_uv_rgba(gs.texture, u, v);
                if alpha >= 0.5 {
                    closest_t = t;
                    hit_ground_sprite = Some((gs, color, u, v));
                    hit_billboard = None;
                    hit_cube = None;
                }
            }
        }
    }

    // Sombreado de ground_sprite (personajes acostados en el suelo)
    if let Some((_gs, sprite_color, _u, _v)) = hit_ground_sprite {
        let hit_point = origin + dir * closest_t;
        // Luz ambiental azul noche con piso legible para que los personajes no salgan casi negros
        let ambient_color = Vector3::new(0.20, 0.25, 0.44);
        let ambient = mul_vec3(sprite_color, ambient_color) + sprite_color * 0.10;
        let mut diffuse = Vector3::zero();

        for light in &scene.lights {
            let light_vec = light.position - hit_point;
            let light_dist = light_vec.length();
            if light_dist < 1e-4 {
                continue;
            }
            let light_dir = light_vec / light_dist;

            let in_shadow = if light.casts_shadow {
                let shadow_orig = hit_point + Vector3::new(0.0, 1e-3, 0.0);
                scene.grid.is_occluded(&scene.cubes, shadow_orig, light_dir, light_dist)
            } else {
                false
            };

            if !in_shadow {
                let norm_dist = light_dist / light.radius;
                let attenuation = (light.intensity / (1.0 + norm_dist * norm_dist)).max(0.0);
                let normal = Vector3::new(0.0, 1.0, 0.0);
                let n_dot_l = normal.dot(light_dir).max(0.2);

                let light_contrib = mul_vec3(sprite_color, light.color) * (attenuation * n_dot_l);
                diffuse += light_contrib;
            }
        }

        return ambient + diffuse;
    }

    // Sombreado de billboard
    if let Some((bb, sprite_color, u, _v)) = hit_billboard {
        let hit_point = origin + dir * closest_t;

        let (effective_sprite_color, emission) = if bb.is_smoke {
            // Teñido cálido abajo y frío arriba según altura sobre el fuego (~3.2 a ~9.0)
            let height_t = ((hit_point.y - 3.2) / 5.5).clamp(0.0, 1.0);
            let warm = Vector3::new(1.15, 0.95, 0.75);
            let cold = Vector3::new(0.70, 0.82, 1.15);
            let tint = warm * (1.0 - height_t) + cold * height_t;
            (mul_vec3(sprite_color, tint), bb.emission)
        } else {
            (sprite_color, bb.emission)
        };

        // Luz ambiental azul noche (estilo Chrono Trigger, +20% para leer relieve)
        let ambient_color = Vector3::new(0.12, 0.17, 0.36);
        let ambient = mul_vec3(effective_sprite_color, ambient_color);
        let mut diffuse = Vector3::zero();

        for light in &scene.lights {
            let light_vec = light.position - hit_point;
            let light_dist = light_vec.length();
            if light_dist < 1e-4 {
                continue;
            }
            let light_dir = light_vec / light_dist;

            let in_shadow = if bb.is_smoke {
                false // humo sin sombras por rayo
            } else if light.casts_shadow {
                let shadow_orig = hit_point - scene.camera_forward * 1e-3;
                scene.grid.is_occluded(&scene.cubes, shadow_orig, light_dir, light_dist)
            } else {
                false
            };

            if !in_shadow {
                // Atenuación suave 1 / (1 + (d/r)^2)
                let norm_dist = light_dist / light.radius;
                let attenuation = (light.intensity / (1.0 + norm_dist * norm_dist)).max(0.0);
                let normal = -scene.camera_forward;
                let wrap = (normal.dot(light_dir) * 0.5 + 0.5).max(0.25);

                // Gradiente horizontal: el lado del sprite orientado hacia la fogata recibe más iluminación cálida
                let to_fire_h = Vector3::new(light_vec.x, 0.0, light_vec.z).normalized();
                let side_bias = (scene.camera_right.dot(to_fire_h) * (u - 0.5) * 1.5).clamp(-0.25, 0.25);
                let fire_factor = (wrap + side_bias).clamp(0.2, 1.25);

                let light_contrib = mul_vec3(effective_sprite_color, light.color) * (attenuation * fire_factor);
                diffuse += light_contrib;
            }
        }

        return ambient + diffuse + emission;
    }

    match hit_cube {
        None => scene.skybox.sample(dir),
        Some(cube) => {
            let hit_point = origin + dir * closest_t;
            let (raw_normal, raw_tangent, raw_bitangent) = cube.tangent_frame_at(hit_point);
            let is_inside = dir.dot(raw_normal) > 0.0;
            let normal = if is_inside { -raw_normal } else { raw_normal };
            let tangent = if is_inside { -raw_tangent } else { raw_tangent };
            let bitangent = raw_bitangent;

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

            // Perturbación de normal mediante normal map (espacio tangente -> mundo)
            let shading_normal = if scene.use_normal_maps {
                if let Some(norm_path) = cube.material.normal_map {
                    if scene.textures.has_texture(norm_path) {
                        let sample = scene.textures.sample_uv(norm_path, hit_u, hit_v);
                        let ts_normal = Vector3::new(
                            sample.x * 2.0 - 1.0,
                            sample.y * 2.0 - 1.0,
                            sample.z * 2.0 - 1.0,
                        );
                        (tangent * ts_normal.x + bitangent * ts_normal.y + normal * ts_normal.z).normalized()
                    } else {
                        normal
                    }
                } else {
                    normal
                }
            } else {
                normal
            };

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
            // Luz ambiental azul noche multiplicada por el albedo de cada superficie (+20%)
            let ambient_color = Vector3::new(0.12, 0.17, 0.36);
            let ambient = mul_vec3(base_color, ambient_color);
            let mut diffuse_specular = Vector3::zero();

            // Iluminación por luces (fogata con sombras + luz de relleno azul tenue sin sombras)
            for light in &scene.lights {
                let light_vec = light.position - hit_point;
                let light_dist = light_vec.length();
                if light_dist < 1e-4 {
                    continue;
                }
                let light_dir = light_vec / light_dist;

                if normal.dot(light_dir) <= 0.0 {
                    continue;
                }

                let n_dot_l = shading_normal.dot(light_dir);
                if n_dot_l <= 0.0 {
                    continue;
                }

                let in_shadow = if light.casts_shadow {
                    let shadow_orig = hit_point + normal * 1e-3;
                    scene.grid.is_occluded(&scene.cubes, shadow_orig, light_dir, light_dist)
                } else {
                    false
                };

                if !in_shadow {
                    // Atenuación suave 1 / (1 + (d/r)^2) para halo cálido y brillo en troncos
                    let norm_dist = light_dist / light.radius;
                    let attenuation = (light.intensity / (1.0 + norm_dist * norm_dist)).max(0.0);
                    let diffuse = mul_vec3(base_color, light.color) * (n_dot_l * attenuation);

                    let mut specular = Vector3::zero();
                    if cube.material.specular > 0.0 {
                        let half_dir = (light_dir + view_dir).normalized();
                        let n_dot_h = shading_normal.dot(half_dir).max(0.0);
                        let shininess = cube.material.specular.max(1.0);
                        let spec_factor = n_dot_h.powf(shininess);
                        let spec_strength = (cube.material.specular / 60.0).clamp(0.04, 0.6);
                        specular = light.color * (spec_factor * attenuation * spec_strength);
                    }

                    diffuse_specular += diffuse + specular;
                }
            }

            // Sombra blanda circular (blob) de ~0.5 bloques de radio bajo los pies de cada personaje
            let mut blob_shadow = 1.0f32;
            for bb in &scene.billboards {
                if !bb.casts_blob_shadow {
                    continue;
                }
                let dx = hit_point.x - bb.position.x;
                let dz = hit_point.z - bb.position.z;
                let dy = hit_point.y - bb.position.y;
                if dy >= -0.25 && dy <= 0.15 {
                    let dist = (dx * dx + dz * dz).sqrt();
                    let radius = 0.5f32;
                    if dist < radius {
                        let t = dist / radius;
                        let falloff = 1.0 - t * t * (3.0 - 2.0 * t);
                        let s = 1.0 - 0.65 * falloff;
                        blob_shadow = blob_shadow.min(s);
                    }
                }
            }
            for gs in &scene.ground_sprites {
                let dx = hit_point.x - gs.position.x;
                let dz = hit_point.z - gs.position.z;
                let dy = hit_point.y - gs.position.y;
                if dy >= -0.15 && dy <= 0.08 {
                    let dist = (dx * dx + dz * dz).sqrt();
                    let radius = 0.85f32;
                    if dist < radius {
                        let t = dist / radius;
                        let falloff = 1.0 - t * t * (3.0 - 2.0 * t);
                        let s = 1.0 - 0.40 * falloff;
                        blob_shadow = blob_shadow.min(s);
                    }
                }
            }

            let local_shading = (ambient + diffuse_specular) * blob_shadow;
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
                        let refl_dir = reflect(&dir, &shading_normal);
                        let refl_orig = hit_point + normal * 1e-3;
                        trace_ray(scene, refl_orig, refl_dir, depth + 1)
                    }
                };
                final_color = final_color * (1.0 - cube.material.transparency)
                    + trans_color * cube.material.transparency;
            }

            if cube.material.reflectivity > 0.0 && depth < MAX_DEPTH {
                let refl_dir = reflect(&dir, &shading_normal);
                let refl_orig = hit_point + normal * 1e-3;
                let refl_color = trace_ray(scene, refl_orig, refl_dir, depth + 1);
                final_color = final_color * (1.0 - cube.material.reflectivity)
                    + refl_color * cube.material.reflectivity;
            }

            final_color + cube.material.emission
        }
    }
}
