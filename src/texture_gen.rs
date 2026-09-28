use raylib::prelude::*;
use std::fs;
use std::path::Path;

/// Hash 2D determinístico para ruido de textura sin librerías externas
fn hash2d(x: i32, y: i32, seed: u32) -> f32 {
    let n = (x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ (seed as i32).wrapping_mul(1274126177)) as u32;
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n & 0x007fffff) as f32 / 8388607.0
}

/// Genera todos los assets necesarios para el diorama si no existen en disco
pub fn generate_all_assets() {
    fs::create_dir_all("assets/party").expect("Failed to create assets directory");

    generate_if_missing("assets/grass.png", gen_grass_texture);
    generate_if_missing("assets/dirt.png", gen_dirt_texture);
    generate_if_missing("assets/stone.png", gen_stone_texture);
    generate_if_missing("assets/stone_normal.png", gen_stone_normal);
    generate_if_missing("assets/bark.png", gen_bark_texture);
    generate_if_missing("assets/bark_normal.png", gen_bark_normal);
    generate_if_missing("assets/leaves.png", gen_leaves_texture);
    generate_if_missing("assets/fire.png", gen_fire_texture);
    generate_if_missing("assets/gem.png", gen_gem_texture);
    generate_if_missing("assets/gem_cyan.png", gen_gem_cyan_texture);
    generate_if_missing("assets/gem_magenta.png", gen_gem_magenta_texture);
    generate_if_missing("assets/water.png", gen_water_texture);
    generate_if_missing("assets/straw.png", gen_straw_texture);
    generate_if_missing("assets/tuft.png", gen_tuft_texture);
    generate_if_missing("assets/grass_tuft.png", gen_grass_tuft_texture);
    generate_if_missing("assets/planks.png", gen_planks_texture);
    generate_if_missing("assets/smoke_puff_0.png", gen_smoke_puff_0);
    generate_if_missing("assets/smoke_puff_1.png", gen_smoke_puff_1);
    generate_if_missing("assets/smoke_puff_2.png", gen_smoke_puff_2);
    generate_if_missing("assets/fern.png", gen_fern_texture);
    generate_if_missing("assets/mushroom.png", gen_mushroom_texture);
    generate_if_missing("assets/cloth.png", gen_cloth_texture);
    generate_if_missing("assets/tent.png", gen_tent_texture);
    generate_if_missing("assets/gate_vortex.png", gen_gate_vortex_texture);

    process_party_sprites();
}

fn process_party_sprites() {
    let party_names = ["Chrono", "Marle", "Lucca", "Frog", "Robo", "Ayla", "Magus"];
    for name in party_names {
        let png_path = format!("assets/party/{}.png", name);
        let jpg_path = format!("assets/party/{}.jpg", name);
        if !Path::new(&png_path).exists() && Path::new(&jpg_path).exists() {
            println!("Preprocesando sprite de party: {}", name);
            process_party_sprite(name, &jpg_path, &png_path);
        }
    }
}

fn process_party_sprite(name: &str, jpg_path: &str, png_path: &str) {
    let Ok(img) = Image::load_image(jpg_path) else {
        eprintln!("Error al cargar {}", jpg_path);
        return;
    };
    let w = img.width;
    let h = img.height;
    let src_colors = img.get_image_data();

    // a) Fondo -> alpha: distancia a blanco < 40 por canal (r >= 215, g >= 215, b >= 215)
    let is_white_like = |c: &Color| -> bool {
        c.r >= 215 && c.g >= 215 && c.b >= 215
    };

    let mut visited = vec![false; (w * h) as usize];
    let mut queue = std::collections::VecDeque::new();

    // Bordes superior e inferior
    for x in 0..w {
        let top = x as usize;
        if is_white_like(&src_colors[top]) {
            visited[top] = true;
            queue.push_back((x, 0));
        }
        let bot = ((h - 1) * w + x) as usize;
        if is_white_like(&src_colors[bot]) && !visited[bot] {
            visited[bot] = true;
            queue.push_back((x, h - 1));
        }
    }
    // Bordes izquierdo y derecho
    for y in 0..h {
        let left = (y * w) as usize;
        if is_white_like(&src_colors[left]) && !visited[left] {
            visited[left] = true;
            queue.push_back((0, y));
        }
        let right = (y * w + (w - 1)) as usize;
        if is_white_like(&src_colors[right]) && !visited[right] {
            visited[right] = true;
            queue.push_back((w - 1, y));
        }
    }

    // Flood fill BFS
    while let Some((cx, cy)) = queue.pop_front() {
        let neighbors = [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)];
        for (nx, ny) in neighbors {
            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                let idx = (ny * w + nx) as usize;
                if !visited[idx] && is_white_like(&src_colors[idx]) {
                    visited[idx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    // Máscara opaca inicial: todo lo no visitado
    let mut opaque = vec![false; (w * h) as usize];
    for i in 0..(w * h) as usize {
        opaque[i] = !visited[i];
    }

    // b) Hojas de dos sprites: aislar la versión chica izquierda
    let is_two_sprites = matches!(name, "Ayla" | "Lucca" | "Magus" | "Marle" | "Robo");
    if is_two_sprites {
        let mut col_counts = vec![0; w as usize];
        for x in 0..w {
            for y in 0..h {
                if opaque[(y * w + x) as usize] {
                    col_counts[x as usize] += 1;
                }
            }
        }
        let mut first_col = 0;
        while first_col < w as usize && col_counts[first_col] == 0 {
            first_col += 1;
        }
        let mut split_col = first_col;
        while split_col < w as usize && col_counts[split_col] > 0 {
            split_col += 1;
        }
        for x in split_col as i32..w {
            for y in 0..h {
                opaque[(y * w + x) as usize] = false;
            }
        }
    }

    // c) Recortar al bounding box de píxeles opacos
    let mut min_x = w;
    let mut max_x = -1;
    let mut min_y = h;
    let mut max_y = -1;

    for y in 0..h {
        for x in 0..w {
            if opaque[(y * w + x) as usize] {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if max_x < min_x || max_y < min_y {
        eprintln!("Advertencia: sin píxeles opacos en {}", name);
        return;
    }

    let crop_w = max_x - min_x + 1;
    let crop_h = max_y - min_y + 1;

    // d) Reducir a ~128 px de alto con promedio por área y binarizar alpha (>= 0.5)
    let out_h = 128;
    let out_w = (((out_h as f32 * crop_w as f32) / crop_h as f32).round() as i32).max(1);

    let mut out_img = Image::gen_image_color(out_w, out_h, Color::BLANK);

    for oy in 0..out_h {
        let sy0 = (oy as f32 * crop_h as f32 / out_h as f32).floor() as i32;
        let sy1 = (((oy + 1) as f32 * crop_h as f32 / out_h as f32).ceil() as i32).max(sy0 + 1).min(crop_h);

        for ox in 0..out_w {
            let sx0 = (ox as f32 * crop_w as f32 / out_w as f32).floor() as i32;
            let sx1 = (((ox + 1) as f32 * crop_w as f32 / out_w as f32).ceil() as i32).max(sx0 + 1).min(crop_w);

            let mut total_pixels = 0;
            let mut opaque_count = 0;
            let mut sum_r = 0.0f32;
            let mut sum_g = 0.0f32;
            let mut sum_b = 0.0f32;

            for iy in sy0..sy1 {
                let src_y = min_y + iy;
                for ix in sx0..sx1 {
                    let src_x = min_x + ix;
                    let idx = (src_y * w + src_x) as usize;
                    total_pixels += 1;
                    if opaque[idx] {
                        opaque_count += 1;
                        let c = &src_colors[idx];
                        sum_r += c.r as f32;
                        sum_g += c.g as f32;
                        sum_b += c.b as f32;
                    }
                }
            }

            let alpha_frac = if total_pixels > 0 {
                opaque_count as f32 / total_pixels as f32
            } else {
                0.0
            };

            if alpha_frac >= 0.5 && opaque_count > 0 {
                let avg_r = (sum_r / opaque_count as f32).round().clamp(0.0, 255.0) as u8;
                let avg_g = (sum_g / opaque_count as f32).round().clamp(0.0, 255.0) as u8;
                let avg_b = (sum_b / opaque_count as f32).round().clamp(0.0, 255.0) as u8;
                out_img.draw_pixel(ox, oy, Color::new(avg_r, avg_g, avg_b, 255));
            } else {
                out_img.draw_pixel(ox, oy, Color::BLANK);
            }
        }
    }

    out_img.export_image(png_path);
}

fn generate_if_missing<F: FnOnce() -> Image>(path: &str, generator: F) {
    if !Path::new(path).exists() {
        let img = generator();
        img.export_image(path);
    }
}

// --- Texturas de Materiales (32x32 px) ---

fn gen_grass_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(35, 105, 80, 255));
    for y in 0..32 {
        for x in 0..32 {
            let h = hash2d(x, y, 101);
            let c = if (x % 4 == 1 && y % 5 < 3) || h > 0.82 {
                Color::new(50, 140, 105, 255) // Brizna teal clara
            } else if h < 0.22 {
                Color::new(22, 75, 58, 255)  // Sombra teal oscura
            } else if h < 0.05 {
                Color::new(30, 50, 45, 255)  // Mota profunda
            } else {
                Color::new(35, 105, 80, 255) // Base azul-verdosa
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_straw_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(175, 135, 55, 255));
    for y in 0..32 {
        for x in 0..32 {
            let h = hash2d(x, y, 303);
            let fiber = (x * 3 + y * 2 + (h * 4.0) as i32) % 6;
            let c = if fiber == 0 || h > 0.85 {
                Color::new(215, 175, 80, 255) // Paja dorada brillante
            } else if fiber == 1 || h > 0.70 {
                Color::new(195, 150, 65, 255) // Tono medio
            } else if fiber == 5 || h < 0.20 {
                Color::new(135, 95, 35, 255)  // Sombra entre fibras
            } else {
                Color::new(165, 125, 50, 255) // Base paja
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_tuft_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(135, 155, 75, 255));
    for y in 0..32 {
        for x in 0..32 {
            let h = hash2d(x, y, 404);
            // Hebras verticales de pasto verde-amarillo desaturado (sin tonos naranjas)
            let vertical_blade = (x % 3 == 0) || (x % 5 == 1);
            let c = if vertical_blade && (y < 24 || h > 0.55) {
                if y < 10 || h > 0.8 {
                    Color::new(175, 195, 95, 255) // Punta verde-amarilla clara
                } else {
                    Color::new(150, 170, 80, 255) // Tallo verde oliva / sage
                }
            } else if h < 0.15 || (y > 24 && h < 0.4) {
                Color::new(85, 105, 45, 255)   // Sombra verde musgo oscuro en la raíz
            } else if h > 0.75 {
                Color::new(160, 180, 88, 255)  // Resplandor verde-amarillento
            } else {
                Color::new(130, 150, 70, 255)  // Base verde-amarillo desaturado
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

/// Genera la textura del mechón de pasto tipo billboard (32x32 px con fondo transparente).
/// 5-7 hojas con curvatura natural, puntas afiladas, base verde oliva oscuro y punta
/// verde-amarilla desaturada.
fn gen_grass_tuft_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::BLANK);

    // Hojas de pasto: (x_base, x_tip, y_tip, width_base)
    let blades = [
        (9.0f32, 4.0f32, 13.0f32, 2.6f32),
        (12.0f32, 8.0f32, 5.0f32, 3.0f32),
        (15.0f32, 15.0f32, 3.0f32, 3.2f32),
        (18.0f32, 22.0f32, 6.0f32, 3.0f32),
        (22.0f32, 27.0f32, 11.0f32, 2.6f32),
        (14.0f32, 12.0f32, 16.0f32, 2.2f32),
    ];

    for &(bx, tx, ty, w_base) in &blades {
        let total_h = 31.0 - ty;
        for y_i in (ty as i32)..=31 {
            let t = (31.0 - y_i as f32) / total_h; // 0 en la base (y=31), 1 en la punta (y=ty)
            let cx = bx + (tx - bx) * (t * 0.75 + t * t * 0.25);
            let half_w = (w_base * (1.0 - t * 0.85) * 0.5).max(0.40);

            let min_x = (cx - half_w).round() as i32;
            let max_x = (cx + half_w).round() as i32;

            // Base verde oliva oscuro: (42, 65, 22), Punta verde-amarillo desaturado: (170, 185, 75)
            let r = (42.0 + t * 128.0) as u8;
            let g = (65.0 + t * 120.0) as u8;
            let b = (22.0 + t * 53.0) as u8;

            for x_i in min_x..=max_x {
                if x_i >= 0 && x_i < 32 && y_i >= 0 && y_i < 32 {
                    let is_edge = (x_i == min_x || x_i == max_x) && t < 0.8;
                    let color = if is_edge {
                        Color::new((r as f32 * 0.78) as u8, (g as f32 * 0.78) as u8, (b as f32 * 0.78) as u8, 255)
                    } else {
                        Color::new(r, g, b, 255)
                    };
                    img.draw_pixel(x_i, y_i, color);
                }
            }
        }
    }
    img
}

fn gen_planks_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(140, 95, 52, 255));
    for y in 0..32 {
        let plank_idx = y / 8; // 4 tablones horizontales
        let is_groove = y % 8 == 0;

        for x in 0..32 {
            let h = hash2d(x, y, 505);
            // Juntas verticales intercaladas entre tablones
            let joint_x = match plank_idx {
                0 => 16,
                1 => 24,
                2 => 8,
                _ => 20,
            };
            let is_vertical_joint = x == joint_x;
            let is_nail = (x == joint_x - 2 || x == joint_x + 2) && (y % 8 == 4);

            let c = if is_groove || is_vertical_joint {
                Color::new(65, 38, 18, 255)   // Hendidura oscura entre tablones
            } else if is_nail {
                Color::new(45, 40, 38, 255)   // Clavo de hierro forjado
            } else if h > 0.85 {
                Color::new(165, 118, 70, 255) // Veta de madera clara
            } else if h < 0.20 {
                Color::new(115, 75, 40, 255)  // Sombra de veta
            } else {
                Color::new(138, 92, 50, 255)  // Madera de tablón cálida
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_dirt_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(100, 65, 38, 255));
    for y in 0..32 {
        for x in 0..32 {
            let h = hash2d(x, y, 202);
            let c = if h > 0.85 {
                Color::new(135, 95, 60, 255) // Grava / piedra clara
            } else if h > 0.70 {
                Color::new(118, 80, 48, 255)
            } else if h < 0.20 {
                Color::new(72, 45, 24, 255)  // Tierra profunda oscura
            } else {
                Color::new(100, 65, 38, 255)
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_stone_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(115, 115, 120, 255));
    for y in 0..32 {
        for x in 0..32 {
            let row = y / 8;
            let offset = if row % 2 == 1 { 8 } else { 0 };
            let col = (x + offset) % 16;
            let edge = col == 0 || col == 15 || y % 8 == 0 || y % 8 == 7;
            let h = hash2d(x, y, 303);

            let c = if edge {
                Color::new(60, 60, 65, 255) // Grieta / mortero oscuro
            } else if h > 0.75 {
                Color::new(145, 145, 150, 255) // Relieve claro
            } else if h < 0.25 {
                Color::new(95, 95, 100, 255)   // Sombra de piedra
            } else {
                Color::new(120, 120, 125, 255)
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn normal_from_heightmap(heights: &[[f32; 32]; 32], strength: f32) -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(128, 128, 255, 255));
    for y in 0..32 {
        for x in 0..32 {
            let x_prev = (x + 31) % 32;
            let x_next = (x + 1) % 32;
            let y_prev = (y + 31) % 32;
            let y_next = (y + 1) % 32;

            let dh_dx = (heights[y as usize][x_next as usize] - heights[y as usize][x_prev as usize]) * 0.5 * strength;
            let dh_dy = (heights[y_next as usize][x as usize] - heights[y_prev as usize][x as usize]) * 0.5 * strength;

            // En espacio tangente: +X es Tangent (U), +Y es Bitangent (V), +Z es Normal
            let n = Vector3::new(-dh_dx, -dh_dy, 1.0).normalized();
            let r = ((n.x * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let g = ((n.y * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let b = ((n.z * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;

            img.draw_pixel(x, y, Color::new(r, g, b, 255));
        }
    }
    img
}

fn gen_stone_normal() -> Image {
    let mut heights = [[0.0f32; 32]; 32];
    for y in 0..32 {
        for x in 0..32 {
            let row = y / 8;
            let offset = if row % 2 == 1 { 8 } else { 0 };
            let col = (x + offset) % 16;
            let y_rel = y % 8;

            // Borde biselado para juntas / grietas de mortero
            let dx = (col as f32 - 7.5).abs();
            let dy = (y_rel as f32 - 3.5).abs();
            let border_x = (7.5 - dx).max(0.0) / 7.5;
            let border_y = (3.5 - dy).max(0.0) / 3.5;
            let brick = border_x.min(border_y).sqrt();

            // Bultos e irregularidades tipo piedra natural
            let bumps = hash2d(x, y, 303) * 0.35;
            heights[y as usize][x as usize] = brick * 0.7 + bumps;
        }
    }
    normal_from_heightmap(&heights, 2.4)
}

fn gen_bark_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(82, 50, 26, 255));
    for y in 0..32 {
        let wave = ((y as f32 * 0.35).sin() * 2.0) as i32;
        for x in 0..32 {
            let rx = (x + wave + 32) % 8;
            let h = hash2d(x, y, 404);
            let c = if rx == 0 || rx == 7 {
                Color::new(50, 28, 14, 255)  // Hendidura oscura
            } else if rx == 3 || rx == 4 {
                Color::new(108, 68, 36, 255) // Cresta de corteza clara
            } else if h > 0.7 {
                Color::new(92, 58, 30, 255)
            } else {
                Color::new(76, 46, 24, 255)
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_bark_normal() -> Image {
    let mut heights = [[0.0f32; 32]; 32];
    for y in 0..32 {
        let wave = ((y as f32 * 0.35).sin() * 2.0) as i32;
        for x in 0..32 {
            let rx = (x + wave + 32) % 8; // 0..8
            let dist_center = (rx as f32 - 3.5).abs();
            let ridge = 1.0 - (dist_center / 3.5); // 1.0 en el centro de la cresta, 0.0 en el surco
            let noise = hash2d(x, y, 404) * 0.25;
            heights[y as usize][x as usize] = ridge * 1.1 + noise;
        }
    }
    normal_from_heightmap(&heights, 2.6)
}

fn gen_smoke_puff_0() -> Image { gen_smoke_puff_texture(0) }
fn gen_smoke_puff_1() -> Image { gen_smoke_puff_texture(1) }
fn gen_smoke_puff_2() -> Image { gen_smoke_puff_texture(2) }

/// Genera una textura de bocanada de humo tipo pixel-art (~24x24 px).
/// Silueta irregular de nube, gris claro (~0.55-0.62) con dithering en damero en bordes e interior.
fn gen_smoke_puff_texture(variant: usize) -> Image {
    let mut img = Image::gen_image_color(24, 24, Color::BLANK);

    // Centros y radios de los lóbulos de la nube según variante: (cx, cy, radius)
    let lobes: [(f32, f32, f32); 4] = match variant {
        0 => [(11.5, 12.5, 7.8), (7.5, 12.0, 5.2), (16.0, 11.5, 5.6), (12.0, 8.0, 5.0)],
        1 => [(12.0, 12.0, 7.5), (8.0, 13.0, 5.5), (15.5, 10.5, 5.2), (11.0, 7.5, 5.2)],
        _ => [(11.0, 11.5, 7.0), (7.0, 11.5, 4.8), (15.0, 12.0, 5.0), (12.5, 7.0, 4.8)],
    };

    for y in 0..24 {
        for x in 0..24 {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Distancia normalizada mínima al centro de los lóbulos
            let mut min_norm_dist = 2.0f32;
            for &(cx, cy, r) in &lobes {
                let dx = px - cx;
                let dy = py - cy;
                let d = (dx * dx + dy * dy).sqrt() / r;
                if d < min_norm_dist {
                    min_norm_dist = d;
                }
            }

            if min_norm_dist <= 1.0 {
                // Color gris claro: superior ligeramente más claro (~0.62), inferior ~0.55
                let norm_y = (py / 24.0).clamp(0.0, 1.0);
                let shade = 160.0 - norm_y * 20.0;
                let c = Color::new(shade as u8, (shade + 2.0) as u8, (shade + 6.0) as u8, 255);

                // Dithering en damero para translucidez pixel-art
                let is_checker = (x + y) % 2 == 1;
                let is_sparse = (x * 2 + y) % 3 != 0;

                let opaque = match variant {
                    0 => {
                        // Base densa: solo damero en borde exterior
                        if min_norm_dist > 0.88 {
                            !is_checker
                        } else if min_norm_dist > 0.78 {
                            !is_sparse
                        } else {
                            true
                        }
                    }
                    1 => {
                        // Medio: damero en borde y moteado sutil en interior
                        if min_norm_dist > 0.82 {
                            !is_checker
                        } else if min_norm_dist > 0.65 {
                            !is_sparse
                        } else {
                            true
                        }
                    }
                    _ => {
                        // Alto (ralo): damero extendido por todo el cuerpo para ver estrellas a través
                        if min_norm_dist > 0.80 {
                            !is_sparse
                        } else {
                            !is_checker
                        }
                    }
                };

                if opaque {
                    img.draw_pixel(x, y, c);
                }
            }
        }
    }

    img
}

fn gen_leaves_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(30, 80, 24, 255));
    for y in 0..32 {
        for x in 0..32 {
            let h = hash2d(x, y, 505);
            let c = if h > 0.80 {
                Color::new(85, 175, 60, 255) // Hoja iluminada
            } else if h > 0.50 {
                Color::new(55, 130, 40, 255) // Verde medio
            } else if h < 0.20 {
                Color::new(18, 48, 14, 255)  // Hueco de sombra profunda
            } else {
                Color::new(32, 85, 25, 255)
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_fire_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(255, 100, 20, 255));
    for y in 0..32 {
        let norm_y = y as f32 / 31.0; // 0 arriba, 1 abajo
        for x in 0..32 {
            let dist_center = ((x as f32 - 15.5) / 15.5).abs();
            let h = hash2d(x, y, 606) * 0.25;
            let intensity = norm_y - dist_center * 0.4 + h;

            let c = if intensity > 0.80 {
                Color::new(255, 250, 190, 255) // Núcleo incandescente blanco/amarillo
            } else if intensity > 0.55 {
                Color::new(255, 190, 30, 255)  // Amarillo fuego
            } else if intensity > 0.30 {
                Color::new(235, 95, 15, 255)   // Naranja brillante
            } else {
                Color::new(165, 30, 10, 255)   // Ascua roja oscura
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn gen_gem_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(80, 180, 230, 255));
    for y in 0..32 {
        for x in 0..32 {
            let f1 = (x + y) / 8;
            let f2 = (x - y + 32) / 8;
            let is_edge = (x + y) % 8 == 0 || (x - y + 32) % 8 == 0;

            let c = if is_edge {
                Color::new(195, 245, 255, 255) // Faceta / arista reflectante
            } else if (f1 + f2) % 2 == 0 {
                Color::new(130, 215, 250, 255) // Cara brillante
            } else {
                Color::new(55, 145, 210, 255)  // Cara profunda
            };
            img.draw_pixel(x, y, c);
        }
    }
    // Destello blanco en esquina superior
    img.draw_pixel(6, 6, Color::WHITE);
    img.draw_pixel(7, 6, Color::WHITE);
    img.draw_pixel(6, 7, Color::WHITE);
    img
}

fn gen_gem_cyan_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(20, 140, 230, 255));
    for y in 0..32 {
        for x in 0..32 {
            let f1 = (x + y) / 8;
            let f2 = (x - y + 32) / 8;
            let is_edge = (x + y) % 8 == 0 || (x - y + 32) % 8 == 0;

            let c = if is_edge {
                Color::new(190, 245, 255, 255) // Arista facetada reflectante cian
            } else if (f1 + f2) % 2 == 0 {
                Color::new(50, 195, 255, 255)  // Cara brillante
            } else {
                Color::new(15, 120, 210, 255)  // Cara profunda saturada
            };
            img.draw_pixel(x, y, c);
        }
    }
    img.draw_pixel(6, 6, Color::WHITE);
    img.draw_pixel(7, 6, Color::WHITE);
    img.draw_pixel(6, 7, Color::WHITE);
    img
}

fn gen_gem_magenta_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(210, 30, 180, 255));
    for y in 0..32 {
        for x in 0..32 {
            let f1 = (x + y) / 8;
            let f2 = (x - y + 32) / 8;
            let is_edge = (x + y) % 8 == 0 || (x - y + 32) % 8 == 0;

            let c = if is_edge {
                Color::new(255, 195, 255, 255) // Arista facetada reflectante magenta
            } else if (f1 + f2) % 2 == 0 {
                Color::new(245, 60, 215, 255)  // Cara brillante
            } else {
                Color::new(170, 15, 140, 255)  // Cara profunda saturada
            };
            img.draw_pixel(x, y, c);
        }
    }
    img.draw_pixel(6, 6, Color::WHITE);
    img.draw_pixel(7, 6, Color::WHITE);
    img.draw_pixel(6, 7, Color::WHITE);
    img
}

fn gen_water_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(35, 80, 145, 220));
    for y in 0..32 {
        let wave = ((x_wave(y) * 2.0) as i32 + 32) % 8;
        for x in 0..32 {
            let c = if wave == 0 {
                Color::new(160, 215, 255, 255) // Espuma / reflejo de onda
            } else if wave <= 2 {
                Color::new(75, 145, 220, 240)  // Onda suave
            } else {
                Color::new(28, 65, 125, 210)   // Fondo translúcido
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

fn x_wave(y: i32) -> f32 {
    (y as f32 * 0.4).sin()
}



fn fill_rect(img: &mut Image, x_start: i32, y_start: i32, w: i32, h: i32, color: Color) {
    for y in y_start..(y_start + h) {
        for x in x_start..(x_start + w) {
            if x >= 0 && x < img.width && y >= 0 && y < img.height {
                img.draw_pixel(x, y, color);
            }
        }
    }
}

/// Helecho de bosque (24x24 px, frondas arqueadas verde oliva y puntas desaturadas con alpha recortado)
fn gen_fern_texture() -> Image {
    let mut img = Image::gen_image_color(24, 24, Color::BLANK);
    let dark_green = Color::new(28, 62, 22, 255);
    let mid_green = Color::new(52, 108, 38, 255);
    let tip_green = Color::new(95, 142, 54, 255);

    // Fronda central arqueada
    for y in 4..24 {
        let x = 12 + (((y - 4) as f32 * 0.15).sin() * 2.0) as i32;
        let c = if y < 8 { tip_green } else if y < 16 { mid_green } else { dark_green };
        img.draw_pixel(x, y, c);
        // Hojas laterales
        let len = ((24 - y) as f32 * 0.35).min(5.0) as i32;
        for dx in 1..=len {
            if x - dx >= 0 { img.draw_pixel(x - dx, y - dx / 2, mid_green); }
            if x + dx < 24 { img.draw_pixel(x + dx, y - dx / 2, mid_green); }
        }
    }
    // Fronda izquierda inclinada
    for y in 8..24 {
        let x = 12 - (24 - y) * 8 / 16;
        let c = if y < 13 { tip_green } else { dark_green };
        if x >= 0 { img.draw_pixel(x, y, c); }
        for dy in 1..=3 {
            if x - 1 >= 0 && y - dy >= 0 { img.draw_pixel(x - 1, y - dy, mid_green); }
        }
    }
    // Fronda derecha inclinada
    for y in 8..24 {
        let x = 12 + (24 - y) * 8 / 16;
        let c = if y < 13 { tip_green } else { dark_green };
        if x < 24 { img.draw_pixel(x, y, c); }
        for dy in 1..=3 {
            if x + 1 < 24 && y - dy >= 0 { img.draw_pixel(x + 1, y - dy, mid_green); }
        }
    }
    img
}

/// Hongos pequeños luminiscentes (16x16 px con sombreretes azul-celeste brillante y leve brillo)
fn gen_mushroom_texture() -> Image {
    let mut img = Image::gen_image_color(16, 16, Color::BLANK);
    let stem = Color::new(210, 200, 180, 255);
    let cap_glow = Color::new(120, 225, 255, 255);
    let cap_mid = Color::new(45, 140, 220, 255);
    let cap_dark = Color::new(20, 75, 150, 255);

    // Hongo 1 (grande, centro-izquierda)
    fill_rect(&mut img, 5, 8, 2, 8, stem);
    fill_rect(&mut img, 3, 5, 6, 4, cap_mid);
    fill_rect(&mut img, 4, 4, 4, 2, cap_glow);
    img.draw_pixel(3, 8, cap_dark);
    img.draw_pixel(8, 8, cap_dark);

    // Hongo 2 (pequeño, derecha)
    fill_rect(&mut img, 11, 10, 2, 6, stem);
    fill_rect(&mut img, 10, 8, 4, 3, cap_mid);
    fill_rect(&mut img, 11, 7, 2, 2, cap_glow);
    img
}

/// Tela de lona rústica para carpa de campamento (32x32 px con tejido sutil)
fn gen_cloth_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::new(45, 60, 50, 255));
    for y in 0..32 {
        for x in 0..32 {
            let weave = (x + y) % 2 == 0;
            let h = hash2d(x, y, 707);
            let c = if (x % 16 == 0 || y % 16 == 0) && h > 0.4 {
                Color::new(75, 95, 75, 255) // Costura / borde de parche
            } else if weave {
                Color::new(52, 70, 58, 255) // Trama de tejido claro
            } else {
                Color::new(38, 52, 44, 255) // Urdimbre oscura
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}

/// Carpa de campamento triangular (40x36 px, lona ocre-roja apagada, entrada oscura, costuras y alpha)
fn gen_tent_texture() -> Image {
    let w = 40;
    let h = 36;
    let mut img = Image::gen_image_color(w, h, Color::BLANK);
    let apex_x = 20.0f32;
    let apex_y = 2.0f32;

    for y in 2..35 {
        let t = (y as f32 - apex_y) / (34.0 - apex_y);
        let half_w = t * 18.0;
        let left_x = (apex_x - half_w).round() as i32;
        let right_x = (apex_x + half_w).round() as i32;

        for x in left_x..=right_x {
            if x < 0 || x >= w {
                continue;
            }
            let is_edge = x == left_x || x == right_x || y == 34;
            let is_center_seam = x == 20;

            // Entrada triangular oscura al centro inferior
            let in_doorway = y >= 17 && (x - 20).abs() <= ((y - 17) * 7 / 17);

            let c = if in_doorway {
                // Interior oscuro de la carpa
                if y == 17 || (x - 20).abs() == ((y - 17) * 7 / 17) {
                    Color::new(95, 40, 25, 255) // Solapa / borde de entrada
                } else {
                    Color::new(24, 20, 22, 255) // Interior umbrío
                }
            } else if is_edge {
                Color::new(125, 50, 30, 255) // Borde perimetral reforzado
            } else if is_center_seam {
                Color::new(210, 115, 75, 255) // Costura central iluminada
            } else if y % 9 == 0 {
                Color::new(145, 62, 38, 255) // Costura horizontal
            } else if x < 20 {
                // Faceta izquierda ligeramente más iluminada (ocre rojizo)
                let noise = (hash2d(x, y, 311) * 20.0 - 10.0) as i32;
                Color::new(
                    (185 + noise).clamp(0, 255) as u8,
                    (88 + noise / 2).clamp(0, 255) as u8,
                    (52 + noise / 3).clamp(0, 255) as u8,
                    255,
                )
            } else {
                // Faceta derecha en sombra tenue
                let noise = (hash2d(x, y, 419) * 20.0 - 10.0) as i32;
                Color::new(
                    (155 + noise).clamp(0, 255) as u8,
                    (70 + noise / 2).clamp(0, 255) as u8,
                    (42 + noise / 3).clamp(0, 255) as u8,
                    255,
                )
            };
            img.draw_pixel(x, y, c);
        }
    }

    // Estacas de anclaje de madera en la base
    img.draw_pixel(2, 35, Color::new(85, 55, 30, 255));
    img.draw_pixel(37, 35, Color::new(85, 55, 30, 255));

    img
}

/// Remolino cósmico del portal del tiempo (Gate, 32x32 px con espiral azul-cian limpia, sin marrones)
fn gen_gate_vortex_texture() -> Image {
    let mut img = Image::gen_image_color(32, 32, Color::BLANK);
    let cx = 15.5f32;
    let cy = 15.5f32;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 14.5 {
                continue;
            }

            let angle = dy.atan2(dx);
            let spiral = (angle * 3.0 + dist * 0.8).sin();
            let dither = (x + y) % 2 == 0;

            // Borde exterior con dithering transparente (alpha 0)
            if dist > 12.2 && !dither {
                continue;
            }

            // Exclusivamente gama celeste y cian puro saturado (sin núcleo blanco ni marrones)
            let c = if dist < 3.2 {
                Color::new(45, 215, 255, 255)  // Núcleo cian saturado resplandeciente (sin blanco)
            } else if spiral > 0.25 {
                Color::new(30, 195, 255, 255)  // Brazo espiral celeste brillante
            } else if spiral > -0.28 {
                Color::new(18, 150, 245, 255)  // Flujo cian intermedio
            } else {
                Color::new(10, 95, 205, 255)   // Azul cósmico puro del vórtice
            };
            img.draw_pixel(x, y, c);
        }
    }
    img
}


