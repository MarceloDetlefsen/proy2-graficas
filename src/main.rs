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
    let headless = args.iter().any(|a| a == "--screenshot");
    let rotated = args.iter().any(|a| a == "--rotated");
    let elevated = args.iter().any(|a| a == "--elevated");
    let closeup = args.iter().any(|a| a == "--closeup");
    let moving_mode = args.iter().any(|a| a == "--moving");

    // 2. Crear la escena y cargar texturas en CPU
    let mut scene = Scene::campfire_diorama();
    scene.load_textures(&mut rl, &thread);

    let mut camera = if closeup {
        // Primer plano de Marle acostada con cámara rotada ~90 grados
        let ground_y = scene::terrain_height_at(&scene.cubes, -1.35, 1.85);
        Camera::new(
            Vector3::new(-4.2, ground_y + 2.4, 1.85),
            Vector3::new(-1.35, ground_y + 0.1, 1.85),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if elevated {
        Camera::new(
            Vector3::new(2.5, 12.5, 10.5),
            Vector3::new(0.0, 2.5, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else if rotated {
        Camera::new(
            Vector3::new(8.5, 5.8, 0.0),
            Vector3::new(-0.8, 3.2, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else {
        // Encuadre inicial más elevado (~30-35 grados de inclinación), mirando a la fogata
        Camera::new(
            Vector3::new(0.0, 7.8, 8.8),
            Vector3::new(0.0, 2.5, -0.2),
            Vector3::new(0.0, 1.0, 0.0),
        )
    };

    let mut framebuffer = vec![Color::BLACK; (WIDTH * HEIGHT) as usize];
    let mut screenshot_saved = false;

    // Textura GPU para presentación rápida en 1 solo draw call (sin draw_pixel pixel por pixel)
    let initial_img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
    let mut render_texture = rl
        .load_texture_from_image(&thread, &initial_img)
        .expect("Error al crear Texture2D de presentación");

    let mut last_camera_move = std::time::Instant::now() - std::time::Duration::from_secs(1);
    let mut is_moving = moving_mode;
    let mut needs_fullres = !moving_mode;
    let mut rendered_fullres = false;

    while !rl.window_should_close() {
        if let Some(ref m) = music {
            m.update_stream();
        }

        // --- Toggle de depuración de mapas normales: tecla N ---
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            scene.use_normal_maps = !scene.use_normal_maps;
            println!("Mapas normales: {}", if scene.use_normal_maps { "ACTIVADOS" } else { "DESACTIVADOS" });
            camera.orbit(0.0, 0.0);
            needs_fullres = true;
            rendered_fullres = false;
        }

        // --- Input de cámara: rotación con mouse/flechas, zoom con scroll ---
        let dt = rl.get_frame_time();
        let mut moved = false;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(-1.5 * dt, 0.0);
            moved = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(1.5 * dt, 0.0);
            moved = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, 1.0 * dt);
            moved = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, -1.0 * dt);
            moved = true;
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            camera.zoom(wheel * 0.5);
            moved = true;
        }

        if moved {
            last_camera_move = std::time::Instant::now();
            is_moving = true;
            needs_fullres = true;
            rendered_fullres = false;
        }

        // --- Render progresivo ---
        // Mientras la cámara se mueve: render a 1/2 de resolución (escala con vecino más cercano).
        // Al pasar >= 150 ms sin movimiento: render a resolución completa (800x600).
        let should_render_moving = (is_moving && last_camera_move.elapsed() < std::time::Duration::from_millis(150)) || (moving_mode && !screenshot_saved);
        let should_render_settled = needs_fullres && last_camera_move.elapsed() >= std::time::Duration::from_millis(150);

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
            println!("Render (reposo full res): {:.2} ms (cubos: {})", elapsed.as_secs_f64() * 1000.0, scene.cubes.len());
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
                let out_file = if closeup {
                    "screenshot_closeup.png"
                } else if elevated {
                    "screenshot_elevated.png"
                } else if rotated {
                    "screenshot_rotated.png"
                } else {
                    "screenshot.png"
                };
                img.export_image(out_file);
                screenshot_saved = true;
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
