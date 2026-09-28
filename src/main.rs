mod camera;
mod cube;
mod material;
mod light;
mod skybox;
mod billboard;
mod procedural;
mod scene;
mod raytrace;
mod texture;
mod texture_gen;
mod grid;

use raylib::prelude::*;
use rayon::prelude::*;
use camera::Camera;
use scene::Scene;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

// Comando ffmpeg para ensamblar el video de los frames generados por auto-orbit (--frames N):
// ffmpeg -r 24 -i frames/frame_%04d.png -c:v libx264 -pix_fmt yuv420p video_diorama.mp4

/// Calcula la vista de cámara para el auto-orbit según la especificación:
/// 360 grados en 24 s, elevación mínima 22 grados (antes 15), radio mínimo seguro.
fn get_orbit_camera(orbit_time: f32) -> (Vector3, Vector3) {
    let target = Vector3::new(0.5, 4.0, -0.6);
    let period = 24.0f32;
    let theta = (orbit_time % period) / period * std::f32::consts::PI * 2.0;

    // Distancia oscilando suavemente entre 9.0 y 13.5
    let d_min = 9.0f32;
    let d_max = 13.5f32;
    let dist = (d_min + d_max) * 0.5 + ((d_max - d_min) * 0.5) * (theta * 2.0).sin();

    // Elevación entre 22° y 36° (mínimo 22 grados requerido)
    let deg2rad = std::f32::consts::PI / 180.0;
    let p_min = 22.0 * deg2rad;
    let p_max = 36.0 * deg2rad;
    let pitch = (p_min + p_max) * 0.5 + ((p_max - p_min) * 0.5) * (theta + std::f32::consts::FRAC_PI_4).sin();

    // Comienza en encuadre frontal (yaw = -PI/2) y gira 360° en sentido horario
    let yaw = theta - std::f32::consts::FRAC_PI_2;
    let eye = target + Vector3::new(
        dist * pitch.cos() * yaw.cos(),
        dist * pitch.sin(),
        dist * pitch.cos() * yaw.sin(),
    );
    (eye, target)
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Diorama - Campamento Nocturno")
        .build();
    rl.set_target_fps(60);

    // 1. Asegurar que todos los assets requeridos existan en disco
    texture_gen::generate_all_assets();

    // Inicializar audio y reproducir música ambiental de fondo (Secret of the Forest)
    let audio = RaylibAudio::init_audio_device();
    let music = match audio {
        Ok(ref aud) => match aud.new_music("assets/audio/Secret of the Forest.mp3") {
            Ok(m) => {
                m.play_stream();
                println!("Música iniciada: Chrono Trigger - Secret of the Forest");
                Some(m)
            }
            Err(e) => {
                eprintln!("Aviso: no se pudo cargar la pista de audio: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Aviso: no se pudo inicializar dispositivo de audio: {}", e);
            None
        }
    };

    let args: Vec<String> = std::env::args().collect();
    let check_layout = args.iter().any(|a| a == "--check-layout");
    let toma5 = args.iter().any(|a| a == "--toma5" || a == "--gem-closeup" || a == "--gem");
    let toma6_on = args.iter().any(|a| a == "--toma6-on" || a == "--log-normal");
    let toma6_off = args.iter().any(|a| a == "--toma6-off" || a == "--log-nonormal");
    let toma7 = args.iter().any(|a| a == "--toma7" || a == "--smoke");
    let elevated = args.iter().any(|a| a == "--elevated" || a == "--toma2");
    let wasd_view = args.iter().any(|a| a == "--wasd-view" || a == "--toma3");
    let toma4_masamune = args.iter().any(|a| a == "--toma4" || a == "--masamune" || a == "--frog" || a == "--gate-closeup" || a == "--gate");
    let glint_view = args.iter().any(|a| a == "--glint" || a == "--easter-egg");
    let frames_arg = args.iter().position(|a| a == "--frames").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<usize>().ok());
    let moving_mode = args.iter().any(|a| a == "--moving");
    let headless = args.iter().any(|a| a == "--screenshot")
        || check_layout
        || toma5
        || toma6_on
        || toma6_off
        || toma7
        || elevated
        || wasd_view
        || toma4_masamune
        || glint_view
        || frames_arg.is_some();

    // 2. Crear la escena y cargar texturas en CPU
    let mut scene = Scene::campfire_diorama();
    scene.load_textures(&mut rl, &thread);

    if toma6_off {
        scene.use_normal_maps = false;
    }

    let mut camera = if toma6_on || toma6_off {
        // Toma 6: Tronco de asiento libre visto de lado a ~1 bloque con luz rasante del fuego (>= 60% cuadro)
        Camera::new(
            Vector3::new(-1.85, 4.32, -0.40),
            Vector3::new(-2.85, 4.22, -0.40),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if glint_view {
        // Closeup del destello del Gate (Easter egg escondido detrás del árbol)
        Camera::new(
            Vector3::new(-1.80, 5.00, -7.00),
            Vector3::new(-3.80, 5.20, -8.20),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if toma5 {
        // Toma 5: Closeup de las dos gemas con la llama entre ellas distorsionada por refracción
        Camera::new(
            Vector3::new(0.00, 4.75, 2.65),
            Vector3::new(0.00, 4.25, 0.40),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if toma7 {
        // Toma 7: Humo mirando hacia arriba contra las estrellas
        Camera::new(
            Vector3::new(0.00, 4.30, 2.50),
            Vector3::new(0.00, 7.80, 0.20),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if toma4_masamune {
        // Toma 4: Closeup de la Masamune clavada en el suelo junto a Frog
        Camera::new(
            Vector3::new(-3.40, 4.80, 1.20),
            Vector3::new(-4.40, 4.30, -0.30),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if wasd_view {
        // Toma 3: Vista a nivel del suelo a través del claro y la fogata
        Camera::new(
            Vector3::new(4.50, 4.60, 3.80),
            Vector3::new(-1.50, 4.20, -1.00),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if elevated {
        // Toma 2: Vista cenital / elevada general del claro y el diorama completo
        Camera::new(
            Vector3::new(3.00, 16.50, 13.50),
            Vector3::new(0.50, 3.80, -0.60),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else {
        // Toma 1: Encuadre inicial frontal con vista completa del campamento nocturno
        // Se ven los 7 personajes, la carpa, la fogata y el humo, con ~15% de margen y cielo entre copas
        Camera::new(
            Vector3::new(0.50, 8.40, 11.80),
            Vector3::new(0.50, 4.20, -0.60),
            Vector3::new(0.0, 1.0, 0.0),
        )
    };

    if check_layout {
        run_check_layout(&scene, &camera);
        return;
    }

    let mut framebuffer = vec![Color::BLACK; (WIDTH * HEIGHT) as usize];
    let mut screenshot_saved = false;

    // Procesar flag --frames N si fue especificado por línea de comandos
    if let Some(num_frames) = frames_arg {
        std::fs::create_dir_all("frames").expect("Failed to create frames directory");
        println!("Renderizando {} frames del auto-orbit a resolución completa (800x600)...", num_frames);
        let mut flagged_frames = Vec::new();
        for f in 0..num_frames {
            let t = f as f32 / num_frames as f32 * 24.0;
            let (orbit_eye, orbit_target) = get_orbit_camera(t);
            camera.set_view(orbit_eye, orbit_target);
            camera.resolve_collision(&scene.cubes);
            camera.update_basis_vectors();

            let cam_dist = (camera.eye - camera.center).length();
            scene.tree_cutaway_dist = Some(cam_dist - 1.5);

            scene.camera_forward = camera.forward;
            scene.camera_right = camera.right;
            scene.camera_up = camera.up;
            render(&scene, &camera, &mut framebuffer, 1);

            let total_pixels = (WIDTH * HEIGHT) as f32;
            let mut sum_brightness = 0.0f32;
            let mut dark_pixels = 0u32;
            for p in &framebuffer {
                let r = p.r as f32 / 255.0;
                let g = p.g as f32 / 255.0;
                let b = p.b as f32 / 255.0;
                let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                sum_brightness += lum;
                if r < 0.04 && g < 0.04 && b < 0.04 {
                    dark_pixels += 1;
                }
            }
            let mean_brightness = sum_brightness / total_pixels;
            let dark_fraction = dark_pixels as f32 / total_pixels;
            if mean_brightness < 0.08 || dark_fraction > 0.40 {
                flagged_frames.push(f);
                eprintln!(
                    "ALERTA Frame {:04}: brillo medio = {:.4} (< 0.08) o píxeles oscuros = {:.1}% (> 40%)",
                    f, mean_brightness, dark_fraction * 100.0
                );
            } else {
                println!(
                    "Frame {:04}: brillo medio = {:.4}, píxeles oscuros = {:.1}% [OK]",
                    f, mean_brightness, dark_fraction * 100.0
                );
            }

            let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    img.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
                }
            }
            let filename = format!("frames/frame_{:04}.png", f);
            img.export_image(&filename);
            println!("Frame guardado: {} ({}/{})", filename, f + 1, num_frames);
        }
        println!("Verificación de auto-orbit completada: {} frames marcados con problemas (meta: 0).", flagged_frames.len());
        return;
    }

    // Textura GPU para presentación rápida en 1 solo draw call (sin draw_pixel pixel por pixel)
    let initial_img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
    let mut render_texture = rl
        .load_texture_from_image(&thread, &initial_img)
        .expect("Error al crear Texture2D de presentación");

    let mut last_camera_move = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let mut is_moving = moving_mode;
    let mut needs_fullres = !moving_mode;
    let mut rendered_fullres = false;
    let mut auto_orbit = false;
    let mut orbit_time = 0.0f32;

    while !rl.window_should_close() {
        if let Some(ref m) = music {
            m.update_stream();
        }

        let dt = rl.get_frame_time();
        let mut moved = false;

        // --- Toggle de depuración de mapas normales: tecla N ---
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            scene.use_normal_maps = !scene.use_normal_maps;
            println!("Mapas normales: {}", if scene.use_normal_maps { "ACTIVADOS" } else { "DESACTIVADOS" });
            camera.orbit(0.0, 0.0);
            needs_fullres = true;
            rendered_fullres = false;
        }

        // --- Auto-órbita con tecla R: 360° en 24 s, resolución completa ---
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            auto_orbit = !auto_orbit;
            println!("Auto-órbita: {}", if auto_orbit { "ACTIVADA (resolución completa)" } else { "DESACTIVADA" });
        }

        // --- Presets de tomas: Teclas 1 a 7 ---
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            // Toma 1: Encuadre inicial mirando a la fogata
            camera.set_view(Vector3::new(0.50, 8.40, 11.80), Vector3::new(0.50, 4.20, -0.60));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            // Toma 2: Vista cenital / elevada general
            camera.set_view(Vector3::new(3.00, 16.50, 13.50), Vector3::new(0.50, 3.80, -0.60));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            // Toma 3: Vista a nivel de suelo a través del claro y la fogata
            camera.set_view(Vector3::new(4.50, 4.60, 3.80), Vector3::new(-1.50, 4.20, -1.00));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            // Toma 4: Closeup de la Masamune clavada en el suelo junto a Frog
            camera.set_view(Vector3::new(-3.40, 4.80, 1.20), Vector3::new(-4.40, 4.30, -0.30));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
            // Toma 5: Closeup de las dos gemas con la llama entre ellas
            camera.set_view(Vector3::new(0.00, 4.75, 2.65), Vector3::new(0.00, 4.25, 0.40));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
            // Toma 6: Tronco de asiento libre visto de lado a ~1 bloque con luz rasante del fuego
            camera.set_view(Vector3::new(-1.85, 4.32, -0.40), Vector3::new(-2.85, 4.22, -0.40));
            auto_orbit = false;
            moved = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SEVEN) {
            // Toma 7: Humo mirando hacia arriba contra las estrellas
            camera.set_view(Vector3::new(0.00, 4.30, 2.50), Vector3::new(0.00, 7.80, 0.20));
            auto_orbit = false;
            moved = true;
        }

        // --- Movimiento WASD / QE del punto de mira ---
        let mut move_speed = 3.5 * dt;
        if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
            move_speed *= 2.5;
        }

        let mut fwd = 0.0f32;
        let mut rgt = 0.0f32;
        let mut upw = 0.0f32;

        if rl.is_key_down(KeyboardKey::KEY_W) { fwd += move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_S) { fwd -= move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_D) { rgt += move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_A) { rgt -= move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_Q) { upw += move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_E) { upw -= move_speed; }

        if fwd != 0.0 || rgt != 0.0 || upw != 0.0 {
            camera.move_target(fwd, rgt, upw, &scene.cubes);
            moved = true;
            auto_orbit = false;
        }

        // --- Órbita con flechas del teclado y zoom con scroll ---
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(-1.5 * dt, 0.0);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(1.5 * dt, 0.0);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, 1.0 * dt);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, -1.0 * dt);
            moved = true;
            auto_orbit = false;
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            camera.zoom(wheel * 0.5);
            moved = true;
            auto_orbit = false;
        }

        if auto_orbit {
            orbit_time += dt;
            let (orbit_eye, orbit_target) = get_orbit_camera(orbit_time);
            camera.set_view(orbit_eye, orbit_target);
            camera.resolve_collision(&scene.cubes);
            camera.update_basis_vectors();
            let cam_dist = (camera.eye - camera.center).length();
            scene.tree_cutaway_dist = Some(cam_dist - 1.5);
            moved = true;
        } else if moved {
            scene.tree_cutaway_dist = None;
        }

        if moved {
            last_camera_move = std::time::Instant::now();
            is_moving = true;
            needs_fullres = true;
            rendered_fullres = false;
        }

        // --- Render progresivo ---
        // Mientras la cámara se mueve manualmente: render a 1/2 de resolución.
        // Al pasar >= 150 ms sin movimiento, o en auto-órbita: render a resolución completa (800x600).
        let should_render_moving = ((is_moving && !auto_orbit) && last_camera_move.elapsed() < std::time::Duration::from_millis(150)) || (moving_mode && !screenshot_saved);
        let should_render_settled = (needs_fullres && last_camera_move.elapsed() >= std::time::Duration::from_millis(150)) || auto_orbit;

        if should_render_moving && (camera.is_changed() || !screenshot_saved) {
            scene.camera_forward = camera.forward;
            scene.camera_right = camera.right;
            scene.camera_up = camera.up;
            let start = std::time::Instant::now();
            render(&scene, &camera, &mut framebuffer, 2);
            let elapsed = start.elapsed();
            println!("Render (movimiento 1/2 res): {:.2} ms (cubos: {})", elapsed.as_secs_f64() * 1000.0, scene.cubes.len());
            let _ = render_texture.update_texture(framebuffer_as_bytes(&framebuffer));

            if moving_mode && !screenshot_saved {
                let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        img.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
                    }
                }
                img.export_image("screenshot_moving.png");
                screenshot_saved = true;
                if headless {
                    break;
                }
            }
        } else if should_render_settled || (camera.is_changed() && !is_moving) || !rendered_fullres {
            scene.camera_forward = camera.forward;
            scene.camera_right = camera.right;
            scene.camera_up = camera.up;
            let start = std::time::Instant::now();
            render(&scene, &camera, &mut framebuffer, 1);
            let elapsed = start.elapsed();
            let total_billboards = scene.billboards.len() + scene.grass_billboards.len() + scene.smoke_billboards.len() + scene.undergrowth_billboards.len();
            println!("Render (reposo full res): {:.2} ms (cubos: {}, billboards: {}, ground_sprites: {})",
                elapsed.as_secs_f64() * 1000.0, scene.cubes.len(), total_billboards, scene.ground_sprites.len());
            let _ = render_texture.update_texture(framebuffer_as_bytes(&framebuffer));
            needs_fullres = false;
            is_moving = false;
            rendered_fullres = true;

            if !screenshot_saved && headless {
                let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        img.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
                    }
                }
                let out_file = if toma6_on {
                    "screenshot_toma6_n_on.png"
                } else if toma6_off {
                    "screenshot_toma6_n_off.png"
                } else if toma5 {
                    "screenshot_toma5.png"
                } else if toma7 {
                    "screenshot_toma7.png"
                } else if toma4_masamune {
                    "screenshot_masamune.png"
                } else if glint_view {
                    "screenshot_glint.png"
                } else if wasd_view {
                    "screenshot_wasd.png"
                } else if elevated {
                    "screenshot_elevated.png"
                } else {
                    "screenshot.png"
                };
                img.export_image(out_file);
                println!("Captura guardada en: {}", out_file);
                break;
            }
        }

        // Presentación en GPU mediante Texture2D (un solo draw call)
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&render_texture, 0, 0, Color::WHITE);
        d.draw_fps(10, 10);
    }
}

/// Lanza rayos paralelizados con rayon usando chunks de 4 filas para balancear carga.
/// Soporta resolución progresiva según el factor `scale` (1 = full res, 2 = 1/2 res con vecino más cercano).
fn render(scene: &Scene, camera: &Camera, framebuffer: &mut [Color], scale: i32) {
    let aspect = WIDTH as f32 / HEIGHT as f32;
    let fov = 60.0_f32.to_radians();
    let tan_half_fov = (fov / 2.0).tan();

    if scale <= 1 {
        let chunk_rows = 4;
        let chunk_pixels = (WIDTH * chunk_rows) as usize;

        framebuffer
            .par_chunks_mut(chunk_pixels)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let start_y = (chunk_idx as i32) * chunk_rows;
                let actual_rows = (chunk.len() / WIDTH as usize) as i32;

                for local_y in 0..actual_rows {
                    let y = start_y + local_y;
                    let py = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * tan_half_fov;

                    for x in 0..WIDTH {
                        let px = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect * tan_half_fov;
                        let dir = camera
                            .basis_change(&Vector3::new(px, py, -1.0))
                            .normalized();

                        let color = raytrace::trace_ray(scene, camera.eye, dir, 0);
                        chunk[(local_y * WIDTH + x) as usize] = to_raylib_color(color);
                    }
                }
            });
    } else {
        let low_w = WIDTH / scale;
        let low_h = HEIGHT / scale;
        let chunk_rows = 4;
        let chunk_pixels = (low_w * chunk_rows) as usize;

        let mut low_fb = vec![Color::BLACK; (low_w * low_h) as usize];

        low_fb
            .par_chunks_mut(chunk_pixels)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let start_y = (chunk_idx as i32) * chunk_rows;
                let actual_rows = (chunk.len() / low_w as usize) as i32;

                for local_y in 0..actual_rows {
                    let y = start_y + local_y;
                    let py = (1.0 - 2.0 * (y as f32 + 0.5) / low_h as f32) * tan_half_fov;

                    for x in 0..low_w {
                        let px = (2.0 * (x as f32 + 0.5) / low_w as f32 - 1.0) * aspect * tan_half_fov;
                        let dir = camera
                            .basis_change(&Vector3::new(px, py, -1.0))
                            .normalized();

                        let color = raytrace::trace_ray(scene, camera.eye, dir, 0);
                        chunk[(local_y * low_w + x) as usize] = to_raylib_color(color);
                    }
                }
            });

        // Escalado con vecino más cercano al framebuffer original
        for y in 0..HEIGHT {
            let ly = (y / scale).min(low_h - 1);
            for x in 0..WIDTH {
                let lx = (x / scale).min(low_w - 1);
                framebuffer[(y * WIDTH + x) as usize] = low_fb[(ly * low_w + lx) as usize];
            }
        }
    }
}

fn to_raylib_color(c: Vector3) -> Color {
    Color::new(
        (c.x.clamp(0.0, 1.0) * 255.0) as u8,
        (c.y.clamp(0.0, 1.0) * 255.0) as u8,
        (c.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}

fn framebuffer_as_bytes(fb: &[Color]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(fb.as_ptr() as *const u8, fb.len() * std::mem::size_of::<Color>())
    }
}

/// Verificación automática del layout para los 7 personajes y la llama de la fogata.
/// Traza 5x5 rayos primarios por entidad (meta >= 85% de visibilidad no ocluida).
/// Evalúa el solape proyectado en pantalla entre cada par de personajes (meta <= 10.0%).
fn run_check_layout(scene: &Scene, camera: &Camera) {
    println!("=== VERIFICACIÓN AUTOMÁTICA DE LAYOUT (--check-layout) ===");
    println!(
        "Cámara de prueba (Toma 1): Eye = ({:.2}, {:.2}, {:.2}), Target = ({:.2}, {:.2}, {:.2})",
        camera.eye.x, camera.eye.y, camera.eye.z, camera.center.x, camera.center.y, camera.center.z
    );

    let aspect = WIDTH as f32 / HEIGHT as f32;
    let fov = 60.0_f32.to_radians();
    let tan_half_fov = (fov / 2.0).tan();

    let project_point = |p: Vector3| -> Option<(f32, f32)> {
        let v = p - camera.eye;
        let depth = v.dot(camera.forward);
        if depth <= 0.1 {
            return None;
        }
        let x_cam = v.dot(camera.right);
        let y_cam = v.dot(camera.up);
        let ndc_x = x_cam / (depth * tan_half_fov * aspect);
        let ndc_y = y_cam / (depth * tan_half_fov);
        let screen_x = (ndc_x + 1.0) * 0.5 * WIDTH as f32;
        let screen_y = (1.0 - ndc_y) * 0.5 * HEIGHT as f32;
        Some((screen_x, screen_y))
    };

    struct EntityCheck {
        name: &'static str,
        center: Vector3,
        width: f32,
        height: f32,
        is_flame: bool,
    }

    let ground_y = scene
        .cubes
        .iter()
        .filter(|c| {
            (c.min.x..=c.max.x).contains(&0.0)
                && (c.min.z..=c.max.z).contains(&0.0)
                && !c.is_tree
                && c.casts_shadow
        })
        .map(|c| c.max.y)
        .fold(4.0f32, f32::max);

    let mut entities: Vec<EntityCheck> = Vec::new();
    let names = ["Robo", "Lucca", "Frog", "Ayla", "Magus", "Marle", "Chrono"];
    for (i, &name) in names.iter().enumerate() {
        if i < scene.billboards.len() {
            let bb = &scene.billboards[i];
            let center = bb.position + camera.up * (bb.height * 0.5);
            entities.push(EntityCheck {
                name,
                center,
                width: bb.width,
                height: bb.height,
                is_flame: false,
            });
        }
    }
    // Llama de la fogata
    entities.push(EntityCheck {
        name: "Llama",
        center: Vector3::new(0.0, ground_y + 0.64, 0.0),
        width: 0.90,
        height: 1.28,
        is_flame: true,
    });

    println!("\n--- 1. Visibilidad por Rayos Primarios (5x5 por entidad, meta >= 85%) ---");
    let mut all_vis_passed = true;
    for ent in &entities {
        let mut visible_rays = 0;
        let total_rays = 25;
        let mut samples = Vec::new();

        for gy in 0..5 {
            let v = (gy as f32 + 0.5) / 5.0;
            for gx in 0..5 {
                let u = (gx as f32 + 0.5) / 5.0;
                let sample_pos = ent.center
                    + camera.right * ((u - 0.5) * ent.width)
                    + camera.up * ((0.5 - v) * ent.height);
                samples.push(sample_pos);
            }
        }

        for &pt in &samples {
            let ray_dir = (pt - camera.eye).normalized();
            let target_dist = (pt - camera.eye).length();
            let mut occluded = false;

            // Test de cubos
            if let Some((t_cube, cube_idx, _, _)) = scene.grid.intersect_closest(&scene.cubes, camera.eye, ray_dir, None) {
                if t_cube < target_dist - 0.05 {
                    let c = &scene.cubes[cube_idx];
                    // Los cubos de cristal transparente (gemas) no bloquean la visión de la llama
                    if c.material.transparency > 0.1 {
                        // Vidrio / gema transparente: no ocluye
                    } else if ent.is_flame {
                        // Ignorar cubos de la fogata misma
                        if c.casts_shadow && c.material.emission.length() < 0.1 {
                            occluded = true;
                        }
                    } else {
                        occluded = true;
                    }
                }
            }

            // Test de billboards delante
            if !occluded {
                for (other_idx, other_bb) in scene.billboards.iter().enumerate() {
                    if !ent.is_flame && other_idx < names.len() && names[other_idx] == ent.name {
                        continue;
                    }
                    if let Some((_, u, v)) = other_bb.intersect(
                        camera.eye,
                        ray_dir,
                        camera.forward,
                        camera.right,
                        camera.up,
                        target_dist - 0.05,
                    ) {
                        let (_, alpha) = scene.textures.sample_uv_rgba(other_bb.texture, u, v);
                        if alpha >= 0.5 {
                            occluded = true;
                            break;
                        }
                    }
                }
            }

            if !occluded {
                visible_rays += 1;
            }
        }

        let vis_pct = (visible_rays as f32 / total_rays as f32) * 100.0;
        let status = if vis_pct >= 85.0 { "PASS" } else { all_vis_passed = false; "FAIL" };
        println!(
            "  - {:<7}: {:>2}/25 rayos no ocluidos ({:>5.1}%) -> [{}]",
            ent.name, visible_rays, vis_pct, status
        );
    }

    println!("\n--- 2. Solape Proyectado en Pantalla entre Pares (meta <= 10.0%) ---");
    let mut all_overlap_passed = true;
    let mut bounding_boxes = Vec::new();
    for ent in &entities {
        let p0 = ent.center - camera.right * (ent.width * 0.5) - camera.up * (ent.height * 0.5);
        let p1 = ent.center + camera.right * (ent.width * 0.5) - camera.up * (ent.height * 0.5);
        let p2 = ent.center + camera.right * (ent.width * 0.5) + camera.up * (ent.height * 0.5);
        let p3 = ent.center - camera.right * (ent.width * 0.5) + camera.up * (ent.height * 0.5);

        if let (Some(s0), Some(s1), Some(s2), Some(s3)) = (
            project_point(p0),
            project_point(p1),
            project_point(p2),
            project_point(p3),
        ) {
            let min_x = s0.0.min(s1.0).min(s2.0).min(s3.0);
            let max_x = s0.0.max(s1.0).max(s2.0).max(s3.0);
            let min_y = s0.1.min(s1.1).min(s2.1).min(s3.1);
            let max_y = s0.1.max(s1.1).max(s2.1).max(s3.1);
            bounding_boxes.push((ent.name, min_x, max_x, min_y, max_y));
        }
    }

    let mut overlap_count = 0;
    for i in 0..bounding_boxes.len() {
        for j in (i + 1)..bounding_boxes.len() {
            let (name_a, min_xa, max_xa, min_ya, max_ya) = bounding_boxes[i];
            let (name_b, min_xb, max_xb, min_yb, max_yb) = bounding_boxes[j];

            let inter_min_x = min_xa.max(min_xb);
            let inter_max_x = max_xa.min(max_xb);
            let inter_min_y = min_ya.max(min_yb);
            let inter_max_y = max_ya.min(max_yb);

            let overlap_pct = if inter_min_x < inter_max_x && inter_min_y < inter_max_y {
                let inter_area = (inter_max_x - inter_min_x) * (inter_max_y - inter_min_y);
                let area_a = (max_xa - min_xa) * (max_ya - min_ya);
                let area_b = (max_xb - min_xb) * (max_yb - min_yb);
                let min_area = area_a.min(area_b);
                (inter_area / min_area) * 100.0
            } else {
                0.0
            };

            let status = if overlap_pct <= 10.0 { "PASS" } else { all_overlap_passed = false; "FAIL" };
            if overlap_pct > 0.0 || status == "FAIL" {
                println!(
                    "  - {:<7} vs {:<7}: solape = {:>5.1}% -> [{}]",
                    name_a, name_b, overlap_pct, status
                );
                overlap_count += 1;
            }
        }
    }
    if overlap_count == 0 {
        println!("  - Ningún par de personajes presenta solape en pantalla (todos 0.0%).");
    }

    println!("\n=== RESUMEN DE LAYOUT ===");
    println!(
        "Visibilidad (>= 85%): {}",
        if all_vis_passed { "TODO PASS" } else { "FAIL DETECTADO" }
    );
    println!(
        "Solape entre pares (<= 10%): {}",
        if all_overlap_passed { "TODO PASS" } else { "FAIL DETECTADO" }
    );
    println!(
        "Resultado global: {}",
        if all_vis_passed && all_overlap_passed {
            "PASS (Layout óptimo)"
        } else {
            "FAIL (Ajuste requerido)"
        }
    );
}
