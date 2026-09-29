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

pub const PRESET_1_EYE: Vector3 = Vector3::new(0.50, 7.56, 9.32);
pub const PRESET_1_TARGET: Vector3 = Vector3::new(0.50, 4.20, -0.60);

pub fn preset_1_camera() -> Camera {
    Camera::new(PRESET_1_EYE, PRESET_1_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

pub const PRESET_2_EYE: Vector3 = Vector3::new(3.00, 16.50, 13.50);
pub const PRESET_2_TARGET: Vector3 = Vector3::new(0.50, 3.80, -0.60);

pub fn preset_2_camera() -> Camera {
    Camera::new(PRESET_2_EYE, PRESET_2_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// --- Cámaras de personajes (Teclas 3 a 9) ---
// 3 = Chrono
pub const CHRONO_TARGET: Vector3 = Vector3::new(2.40, 4.80, 2.40);
pub const CHRONO_EYE: Vector3 = Vector3::new(1.56, 5.10, 4.70);
pub fn chrono_camera() -> Camera {
    Camera::new(CHRONO_EYE, CHRONO_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 4 = Marle
pub const MARLE_TARGET: Vector3 = Vector3::new(-2.20, 4.755, 2.20);
pub const MARLE_EYE: Vector3 = Vector3::new(-1.40, 5.055, 4.39);
pub fn marle_camera() -> Camera {
    Camera::new(MARLE_EYE, MARLE_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 5 = Lucca
pub const LUCCA_TARGET: Vector3 = Vector3::new(0.90, 4.81, -3.20);
pub const LUCCA_EYE: Vector3 = Vector3::new(0.14, 5.11, -1.12);
pub fn lucca_camera() -> Camera {
    Camera::new(LUCCA_EYE, LUCCA_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 6 = Robo
pub const ROBO_TARGET: Vector3 = Vector3::new(-1.80, 4.99, -2.80);
pub const ROBO_EYE: Vector3 = Vector3::new(-1.25, 5.40, -0.40);
pub fn robo_camera() -> Camera {
    Camera::new(ROBO_EYE, ROBO_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 7 = Frog (+ Masamune)
pub const FROG_TARGET: Vector3 = Vector3::new(-4.30, 4.45, -0.40);
pub const FROG_EYE: Vector3 = Vector3::new(-3.61, 4.75, 1.51);
pub fn frog_camera() -> Camera {
    Camera::new(FROG_EYE, FROG_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 8 = Ayla
pub const AYLA_TARGET: Vector3 = Vector3::new(3.90, 4.90, -0.40);
pub const AYLA_EYE: Vector3 = Vector3::new(3.06, 5.20, 1.90);
pub fn ayla_camera() -> Camera {
    Camera::new(AYLA_EYE, AYLA_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// 9 = Magus
pub const MAGUS_TARGET: Vector3 = Vector3::new(5.60, 4.945, -2.60);
pub const MAGUS_EYE: Vector3 = Vector3::new(4.72, 5.245, -0.17);
pub fn magus_camera() -> Camera {
    Camera::new(MAGUS_EYE, MAGUS_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// --- Tomas especiales (F1 a F4) ---
// F1 = Gemas closeup
pub const F1_GEMS_TARGET: Vector3 = Vector3::new(0.00, 4.25, 0.40);
pub const F1_GEMS_EYE: Vector3 = Vector3::new(0.00, 4.75, 2.65);
pub fn f1_gems_camera() -> Camera {
    Camera::new(F1_GEMS_EYE, F1_GEMS_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// F2 = Tronco B (Toma 6)
pub const TOMA_6_EYE: Vector3 = Vector3::new(-2.15, 4.35, -0.40);
pub const TOMA_6_TARGET: Vector3 = Vector3::new(-2.85, 4.22, -0.40);
pub fn toma_6_camera() -> Camera {
    Camera::new(TOMA_6_EYE, TOMA_6_TARGET, Vector3::new(0.0, 1.0, 0.0))
}
pub fn f2_log_camera() -> Camera { toma_6_camera() }

// F3 = Humo hacia arriba
pub const F3_SMOKE_TARGET: Vector3 = Vector3::new(0.00, 7.80, 0.20);
pub const F3_SMOKE_EYE: Vector3 = Vector3::new(0.00, 4.30, 2.50);
pub fn f3_smoke_camera() -> Camera {
    Camera::new(F3_SMOKE_EYE, F3_SMOKE_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// F4 = Portal escondido (centrado en pantalla, ocupando >= 15% del ancho)
pub const F4_PORTAL_TARGET: Vector3 = Vector3::new(1.05, 4.55, -8.65);
pub const F4_PORTAL_EYE: Vector3 = Vector3::new(1.05, 4.55, -6.15);
pub fn f4_portal_camera() -> Camera {
    Camera::new(F4_PORTAL_EYE, F4_PORTAL_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

// Arroyo para reflejos ON/OFF
pub const ARROYO_TARGET: Vector3 = Vector3::new(3.00, 3.45, 5.20);
pub const ARROYO_EYE: Vector3 = Vector3::new(-4.00, 3.60, 4.35);
pub fn arroyo_camera() -> Camera {
    Camera::new(ARROYO_EYE, ARROYO_TARGET, Vector3::new(0.0, 1.0, 0.0))
}

#[derive(Clone, Copy)]
struct CameraTransition {
    start_eye: Vector3,
    start_target: Vector3,
    end_eye: Vector3,
    end_target: Vector3,
    elapsed: f32,
    duration: f32,
}

impl CameraTransition {
    fn new(start_eye: Vector3, start_target: Vector3, end_eye: Vector3, end_target: Vector3, duration: f32) -> Self {
        Self {
            start_eye,
            start_target,
            end_eye,
            end_target,
            elapsed: 0.0,
            duration,
        }
    }

    fn update(&mut self, dt: f32) -> (Vector3, Vector3, bool) {
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        // Smoothstep ease-in / ease-out: 3t^2 - 2t^3
        let ease = t * t * (3.0 - 2.0 * t);
        let eye = self.start_eye + (self.end_eye - self.start_eye) * ease;
        let target = self.start_target + (self.end_target - self.start_target) * ease;
        let finished = t >= 1.0;
        (eye, target, finished)
    }
}

/// Calcula la vista de cámara para el auto-orbit según la especificación:
/// 360 grados en 24 s, elevación mínima 22 grados, arrancando en el azimut de la tecla 1 (+Z mirando a -Z).
fn get_orbit_camera(orbit_time: f32) -> (Vector3, Vector3) {
    let target = Vector3::new(0.5, 4.0, -0.6);
    let period = 24.0f32;
    let theta = (orbit_time % period) / period * std::f32::consts::PI * 2.0;

    // Distancia fija constante (sin zoom-ins ni zoom-outs molestos)
    let dist = 11.0f32;

    // Elevación fija constante (rotación pura 360 grados, cumpliendo mínimo 22 grados)
    let deg2rad = std::f32::consts::PI / 180.0;
    let pitch = 26.0 * deg2rad;

    // Arranca en el azimut de la tecla 1 (yaw = +PI/2, mirando de +Z hacia -Z) y da la vuelta completa de 360°
    let yaw = theta + std::f32::consts::FRAC_PI_2;
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
        .log_level(raylib::consts::TraceLogLevel::LOG_WARNING)
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
    let docs_mode = args.iter().any(|a| a == "--docs");
    let f1_gem = args.iter().any(|a| a == "--f1" || a == "--gem" || a == "--toma5");
    let f2_on = args.iter().any(|a| a == "--f2" || a == "--toma6-on" || a == "--log-normal");
    let f2_off = args.iter().any(|a| a == "--toma6-off" || a == "--log-nonormal");
    let f3_smoke = args.iter().any(|a| a == "--f3" || a == "--smoke" || a == "--toma7");
    let f4_portal = args.iter().any(|a| a == "--f4" || a == "--portal");
    let chrono_view = args.iter().any(|a| a == "--chrono" || a == "--toma3");
    let marle_view = args.iter().any(|a| a == "--marle");
    let lucca_view = args.iter().any(|a| a == "--lucca");
    let robo_view = args.iter().any(|a| a == "--robo");
    let frog_view = args.iter().any(|a| a == "--frog" || a == "--masamune" || a == "--toma4");
    let ayla_view = args.iter().any(|a| a == "--ayla");
    let magus_view = args.iter().any(|a| a == "--magus");
    let arroyo_on = args.iter().any(|a| a == "--arroyo-on");
    let arroyo_off = args.iter().any(|a| a == "--arroyo-off");
    let elevated = args.iter().any(|a| a == "--elevated" || a == "--toma2");
    let overview = args.iter().any(|a| a == "--overview");
    let frames_arg = args.iter().position(|a| a == "--frames").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<usize>().ok());
    let moving_mode = args.iter().any(|a| a == "--moving");
    let headless = args.iter().any(|a| a == "--screenshot")
        || overview
        || check_layout
        || docs_mode
        || f1_gem
        || f2_on
        || f2_off
        || f3_smoke
        || f4_portal
        || chrono_view
        || marle_view
        || lucca_view
        || robo_view
        || frog_view
        || ayla_view
        || magus_view
        || arroyo_on
        || arroyo_off
        || elevated
        || frames_arg.is_some();

    // 2. Crear la escena y cargar texturas en CPU
    let mut scene = Scene::campfire_diorama();
    scene.load_textures(&mut rl, &thread);

    if f2_off {
        scene.use_normal_maps = false;
    }
    if arroyo_off {
        scene.use_reflections = false;
    }

    let mut camera = if f2_on || f2_off {
        f2_log_camera()
    } else if f1_gem {
        f1_gems_camera()
    } else if f3_smoke {
        f3_smoke_camera()
    } else if f4_portal {
        f4_portal_camera()
    } else if chrono_view {
        chrono_camera()
    } else if marle_view {
        marle_camera()
    } else if lucca_view {
        lucca_camera()
    } else if robo_view {
        robo_camera()
    } else if frog_view {
        frog_camera()
    } else if ayla_view {
        ayla_camera()
    } else if magus_view {
        magus_camera()
    } else if arroyo_on || arroyo_off {
        arroyo_camera()
    } else if elevated {
        preset_2_camera()
    } else if overview {
        Camera::new(Vector3::new(0.50, 10.0, 16.5), Vector3::new(0.50, 4.20, -0.60), Vector3::new(0.0, 1.0, 0.0))
    } else {
        preset_1_camera()
    };

    if check_layout {
        run_check_layout(&scene);
        return;
    }

    let mut framebuffer = vec![Color::BLACK; (WIDTH * HEIGHT) as usize];
    let mut screenshot_saved = false;

    // Procesar flag --docs para generar la suite oficial de capturas en docs/screenshots/
    if docs_mode {
        std::fs::create_dir_all("docs/screenshots").expect("Failed to create docs/screenshots directory");
        println!("Generando la suite completa de capturas oficiales de documentación (--docs)...");

        let shots: [(&str, Camera, bool, bool); 17] = [
            ("docs/screenshots/01_encuadre_inicial.png", preset_1_camera(), true, true),
            ("docs/screenshots/02_vista_elevada.png", preset_2_camera(), true, true),
            ("docs/screenshots/03_gemas_refraccion.png", f1_gems_camera(), true, true),
            ("docs/screenshots/04_tronco_normal_on.png", f2_log_camera(), true, true),
            ("docs/screenshots/05_tronco_normal_off.png", f2_log_camera(), false, true),
            ("docs/screenshots/06_humo_estrellas.png", f3_smoke_camera(), true, true),
            ("docs/screenshots/07_arroyo_reflejo_on.png", arroyo_camera(), true, true),
            ("docs/screenshots/07_arroyo_reflejo_off.png", arroyo_camera(), true, false),
            ("docs/screenshots/08_masamune.png", Camera::new(Vector3::new(-3.40, 4.80, 1.20), Vector3::new(-4.40, 4.30, -0.30), Vector3::new(0.0, 1.0, 0.0)), true, true),
            ("docs/screenshots/09_portal_escondido.png", f4_portal_camera(), true, true),
            ("docs/screenshots/10_chrono.png", chrono_camera(), true, true),
            ("docs/screenshots/11_marle.png", marle_camera(), true, true),
            ("docs/screenshots/12_lucca.png", lucca_camera(), true, true),
            ("docs/screenshots/13_robo.png", robo_camera(), true, true),
            ("docs/screenshots/14_frog.png", frog_camera(), true, true),
            ("docs/screenshots/15_ayla.png", ayla_camera(), true, true),
            ("docs/screenshots/16_magus.png", magus_camera(), true, true),
        ];

        for (path, mut cam, use_nm, use_refl) in shots {
            scene.use_normal_maps = use_nm;
            scene.use_reflections = use_refl;
            cam.resolve_collision(&scene.cubes);
            cam.update_basis_vectors();
            scene.camera_forward = cam.forward;
            scene.camera_right = cam.right;
            scene.camera_up = cam.up;
            scene.tree_cutaway_dist = None;

            render(&scene, &cam, &mut framebuffer, 1);

            let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    img.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
                }
            }
            img.export_image(path);
            println!("Captura guardada: {}", path);
        }
        println!("Generación de documentación completada exitosamente.");
        return;
    }

    // Procesar flag --frames N si fue especificado por línea de comandos
    if let Some(num_frames) = frames_arg {
        std::fs::create_dir_all("frames").expect("Failed to create frames directory");
        println!("Renderizando {} frames del auto-orbit a resolución completa (800x600)...", num_frames);
        let mut flagged_frames = Vec::new();
        for f in 0..num_frames {
            let t = f as f32 / num_frames as f32 * 24.0;
            let (orbit_eye, orbit_target) = get_orbit_camera(t);
            camera.set_view(orbit_eye, orbit_target);
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
    let mut transition: Option<CameraTransition> = None;
    let mut show_fps = true;

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

        // --- Toggle de reflexiones: tecla X ---
        if rl.is_key_pressed(KeyboardKey::KEY_X) {
            scene.use_reflections = !scene.use_reflections;
            println!("Reflexiones: {}", if scene.use_reflections { "ACTIVADAS" } else { "DESACTIVADAS" });
            camera.orbit(0.0, 0.0);
            needs_fullres = true;
            rendered_fullres = false;
        }

        // --- Toggle de contador de FPS: tecla F ---
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            show_fps = !show_fps;
            println!("Contador de FPS: {}", if show_fps { "ACTIVADO" } else { "DESACTIVADO" });
        }

        // --- Auto-órbita con tecla R: 360° en 24 s ---
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            auto_orbit = !auto_orbit;
            transition = None;
            println!("Auto-órbita: {}", if auto_orbit { "ACTIVADA" } else { "DESACTIVADA" });
        }

        // --- Presets de tomas: Teclas 1 a 9 y F1 a F4 ---
        let mut target_view: Option<(Vector3, Vector3, &'static str)> = None;

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            target_view = Some((PRESET_1_EYE, PRESET_1_TARGET, "1: Encuadre Inicial"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            target_view = Some((PRESET_2_EYE, PRESET_2_TARGET, "2: Vista Elevada"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            target_view = Some((CHRONO_EYE, CHRONO_TARGET, "3: Chrono"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            target_view = Some((MARLE_EYE, MARLE_TARGET, "4: Marle"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
            target_view = Some((LUCCA_EYE, LUCCA_TARGET, "5: Lucca"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
            target_view = Some((ROBO_EYE, ROBO_TARGET, "6: Robo"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_SEVEN) {
            target_view = Some((FROG_EYE, FROG_TARGET, "7: Frog & Masamune"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_EIGHT) {
            target_view = Some((AYLA_EYE, AYLA_TARGET, "8: Ayla"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_NINE) {
            target_view = Some((MAGUS_EYE, MAGUS_TARGET, "9: Magus"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            target_view = Some((F1_GEMS_EYE, F1_GEMS_TARGET, "F1: Closeup Gemas"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            target_view = Some((TOMA_6_EYE, TOMA_6_TARGET, "F2: Tronco Rasante (N ON/OFF)"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_F3) {
            target_view = Some((F3_SMOKE_EYE, F3_SMOKE_TARGET, "F3: Humo hacia Estrellas"));
        } else if rl.is_key_pressed(KeyboardKey::KEY_F4) {
            target_view = Some((F4_PORTAL_EYE, F4_PORTAL_TARGET, "F4: Portal Escondido"));
        }

        if let Some((dst_eye, dst_target, name)) = target_view {
            println!("Transición de cámara -> {}", name);
            transition = Some(CameraTransition::new(camera.eye, camera.center, dst_eye, dst_target, 0.5));
            auto_orbit = false;
            scene.tree_cutaway_dist = None;
        }

        // --- Actualización de transición suave de cámara (~0.5s con smoothstep ease in/out) ---
        if let Some(ref mut tr) = transition {
            let (eye, target, done) = tr.update(dt);
            camera.set_view(eye, target);
            camera.resolve_collision(&scene.cubes);
            camera.update_basis_vectors();
            moved = true;
            if done {
                transition = None;
            }
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
            transition = None;
            camera.move_target(fwd, rgt, upw, &scene.cubes);
            moved = true;
            auto_orbit = false;
        }

        // --- Órbita con flechas del teclado y zoom con scroll ---
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            transition = None;
            camera.orbit(-1.5 * dt, 0.0);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            transition = None;
            camera.orbit(1.5 * dt, 0.0);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            transition = None;
            camera.orbit(0.0, 1.0 * dt);
            moved = true;
            auto_orbit = false;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            transition = None;
            camera.orbit(0.0, -1.0 * dt);
            moved = true;
            auto_orbit = false;
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            transition = None;
            camera.zoom(wheel * 0.5);
            moved = true;
            auto_orbit = false;
        }

        if auto_orbit {
            orbit_time += dt;
            let (orbit_eye, orbit_target) = get_orbit_camera(orbit_time);
            camera.set_view(orbit_eye, orbit_target);
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
        // Mientras la cámara se mueve (manualmente, en transición o en auto-órbita): render a 1/2 de resolución (60 FPS fluidos).
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
            if let Some(ref m) = music {
                m.update_stream();
            }
            if moving_mode {
                println!("Render (movimiento 1/2 res): {:.2} ms (cubos: {})", elapsed.as_secs_f64() * 1000.0, scene.cubes.len());
            }
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
            if let Some(ref m) = music {
                m.update_stream();
            }
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
                let out_file = if f2_on {
                    "screenshot_f2_log_n_on.png"
                } else if f2_off {
                    "screenshot_f2_log_n_off.png"
                } else if f1_gem {
                    "screenshot_f1_gems.png"
                } else if f3_smoke {
                    "screenshot_f3_smoke.png"
                } else if f4_portal {
                    "screenshot_f4_portal.png"
                } else if chrono_view {
                    "screenshot_chrono.png"
                } else if marle_view {
                    "screenshot_marle.png"
                } else if lucca_view {
                    "screenshot_lucca.png"
                } else if robo_view {
                    "screenshot_robo.png"
                } else if frog_view {
                    "screenshot_frog.png"
                } else if ayla_view {
                    "screenshot_ayla.png"
                } else if magus_view {
                    "screenshot_magus.png"
                } else if arroyo_on {
                    "screenshot_arroyo_on.png"
                } else if arroyo_off {
                    "screenshot_arroyo_off.png"
                } else if elevated {
                    "screenshot_elevated.png"
                } else if overview {
                    "screenshot_overview.png"
                } else {
                    "screenshot.png"
                };
                img.export_image(out_file);
                println!("Captura guardada en: {}", out_file);
                break;
            }
        }

        // Presentación en GPU mediante Texture2D (un solo draw call)
        if let Some(ref m) = music {
            m.update_stream();
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&render_texture, 0, 0, Color::WHITE);
        if show_fps {
            d.draw_fps(10, 10);
        }
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

/// Verificación automática del layout para los 7 personajes, la llama, el portal escondido y la toma 6.
fn run_check_layout(scene: &Scene) {
    let camera = preset_1_camera();
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

    println!("\n--- 3. Verificación de Portal Escondido (5x5 rayos, Toma 1 = 0%, Órbita >= 60% en 2-5 frames) ---");
    let mut portal_passed = false;
    if let Some(portal_bb) = scene.billboards.iter().find(|b| b.texture == "assets/gate_vortex.png") {
        println!("Portal encontrado en posición ({:.2}, {:.2}, {:.2}), tamaño {:.2}",
            portal_bb.position.x, portal_bb.position.y, portal_bb.position.z, portal_bb.width);

        let test_portal_vis = |cam: &Camera, cutaway: Option<f32>| -> (usize, f32) {
            let center = portal_bb.position + cam.up * (portal_bb.height * 0.5);
            let mut vis_count = 0;
            for gy in 0..5 {
                let v = (gy as f32 + 0.5) / 5.0;
                for gx in 0..5 {
                    let u = (gx as f32 + 0.5) / 5.0;
                    let sample_pos = center
                        + cam.right * ((u - 0.5) * portal_bb.width)
                        + cam.up * ((0.5 - v) * portal_bb.height);
                    let ray_dir = (sample_pos - cam.eye).normalized();
                    let target_dist = (sample_pos - cam.eye).length();
                    let mut occluded = false;

                    if let Some((t_cube, _, _, _)) = scene.grid.intersect_closest(&scene.cubes, cam.eye, ray_dir, cutaway) {
                        if t_cube < target_dist - 0.05 {
                            occluded = true;
                        }
                    }
                    if !occluded {
                        for other_bb in &scene.billboards {
                            if other_bb.texture == portal_bb.texture {
                                continue;
                            }
                            if let Some((_, bu, bv)) = other_bb.intersect(
                                cam.eye,
                                ray_dir,
                                cam.forward,
                                cam.right,
                                cam.up,
                                target_dist - 0.05,
                            ) {
                                let (_, alpha) = scene.textures.sample_uv_rgba(other_bb.texture, bu, bv);
                                if alpha >= 0.5 {
                                    occluded = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !occluded {
                        vis_count += 1;
                    }
                }
            }
            (vis_count, (vis_count as f32 / 25.0) * 100.0)
        };

        let (toma1_vis_count, toma1_vis_pct) = test_portal_vis(&camera, None);
        let toma1_ok = toma1_vis_pct == 0.0;
        println!("  - Toma 1 (Encuadre inicial): {:>2}/25 rayos ({:>5.1}%) -> [{}]",
            toma1_vis_count, toma1_vis_pct, if toma1_ok { "PASS (Oculto 0%)" } else { "FAIL (Visible desde toma 1)" });

        let mut orbit_vis_pcts = Vec::with_capacity(24);
        let mut visible_frames = Vec::new();
        println!("  - Visibilidad en los 24 frames de la órbita (azimut inicial = Tecla 1):");
        for f in 0..24 {
            let t = f as f32;
            let (orbit_eye, orbit_target) = get_orbit_camera(t);
            let mut cam = Camera::new(orbit_eye, orbit_target, Vector3::new(0.0, 1.0, 0.0));
            // En órbita circular pura no se resuelve colisión contra copas (se usa cutaway)
            cam.update_basis_vectors();
            let cam_dist = (cam.eye - cam.center).length();
            let cutaway = Some(cam_dist - 1.5);

            let (vis_count, pct) = test_portal_vis(&cam, cutaway);
            orbit_vis_pcts.push(pct);
            if pct >= 60.0 {
                visible_frames.push(f);
            }
            println!("      Frame {:02}: {:>2}/25 rayos ({:>5.1}%){}",
                f, vis_count, pct, if pct >= 60.0 { " [VISIBLE >= 60%]" } else { "" });
        }

        let mut max_consecutive = 0;
        let mut curr_consecutive = 0;
        for &pct in &orbit_vis_pcts {
            if pct >= 60.0 {
                curr_consecutive += 1;
                if curr_consecutive > max_consecutive {
                    max_consecutive = curr_consecutive;
                }
            } else {
                curr_consecutive = 0;
            }
        }

        let orbit_ok = max_consecutive >= 2 && max_consecutive <= 5;
        println!("  - Frames con >= 60% visibilidad: {:?}", visible_frames);
        println!("  - Racha consecutiva máxima: {} frames (criterio: entre 2 y 5 frames) -> [{}]",
            max_consecutive, if orbit_ok { "PASS" } else { "FAIL" });

        portal_passed = toma1_ok && orbit_ok;
    } else {
        eprintln!("  ERROR: No se encontró el billboard del portal en la escena!");
    }

    println!("\n--- 4. Verificación de Toma 6 (Tronco B, meta >= 60% ocupación del cuadro) ---");
    let cam6 = toma_6_camera();
    println!("Cámara Toma 6: Eye = ({:.2}, {:.2}, {:.2}), Target = ({:.2}, {:.2}, {:.2})",
        cam6.eye.x, cam6.eye.y, cam6.eye.z, cam6.center.x, cam6.center.y, cam6.center.z);

    // Muestreo con grilla de rayos primarios sobre el cuadro de la cámara
    let mut hits_5x5 = 0;
    for gy in 0..5 {
        let py = (1.0 - 2.0 * (gy as f32 + 0.5) / 5.0) * tan_half_fov;
        for gx in 0..5 {
            let px = (2.0 * (gx as f32 + 0.5) / 5.0 - 1.0) * aspect * tan_half_fov;
            let dir = cam6.basis_change(&Vector3::new(px, py, -1.0)).normalized();

            let mut hit_is_log = false;
            let mut closest_t = f32::MAX;

            if let Some((t_cube, cube_idx, _, _)) = scene.grid.intersect_closest(&scene.cubes, cam6.eye, dir, None) {
                closest_t = t_cube;
                let c = &scene.cubes[cube_idx];
                // Tronco B está en x in [-3.25, -2.75], z in [-1.4, 0.6]
                if c.min.x < -2.7 && c.max.x > -3.3 && c.min.z < -1.3 && c.max.z > 0.5 && !c.is_tree {
                    hit_is_log = true;
                }
            }

            if hit_is_log {
                for bb in &scene.billboards {
                    if let Some((_, u, v)) = bb.intersect(cam6.eye, dir, cam6.forward, cam6.right, cam6.up, closest_t) {
                        let (_, alpha) = scene.textures.sample_uv_rgba(bb.texture, u, v);
                        if alpha >= 0.5 {
                            hit_is_log = false;
                            break;
                        }
                    }
                }
            }

            if hit_is_log {
                hits_5x5 += 1;
            }
        }
    }
    let pct_5x5 = (hits_5x5 as f32 / 25.0) * 100.0;

    let mut hits_fine = 0;
    let fine_n = 25;
    for gy in 0..fine_n {
        let py = (1.0 - 2.0 * (gy as f32 + 0.5) / fine_n as f32) * tan_half_fov;
        for gx in 0..fine_n {
            let px = (2.0 * (gx as f32 + 0.5) / fine_n as f32 - 1.0) * aspect * tan_half_fov;
            let dir = cam6.basis_change(&Vector3::new(px, py, -1.0)).normalized();

            let mut hit_is_log = false;
            let mut closest_t = f32::MAX;

            if let Some((t_cube, cube_idx, _, _)) = scene.grid.intersect_closest(&scene.cubes, cam6.eye, dir, None) {
                closest_t = t_cube;
                let c = &scene.cubes[cube_idx];
                if c.min.x < -2.7 && c.max.x > -3.3 && c.min.z < -1.3 && c.max.z > 0.5 && !c.is_tree {
                    hit_is_log = true;
                }
            }

            if hit_is_log {
                for bb in &scene.billboards {
                    if let Some((_, u, v)) = bb.intersect(cam6.eye, dir, cam6.forward, cam6.right, cam6.up, closest_t) {
                        let (_, alpha) = scene.textures.sample_uv_rgba(bb.texture, u, v);
                        if alpha >= 0.5 {
                            hit_is_log = false;
                            break;
                        }
                    }
                }
            }

            if hit_is_log {
                hits_fine += 1;
            }
        }
    }
    let pct_fine = (hits_fine as f32 / (fine_n * fine_n) as f32) * 100.0;
    let toma6_passed = pct_fine >= 60.0;

    println!("  - Ocupación grilla 5x5: {:>2}/25 rayos ({:>5.1}%)", hits_5x5, pct_5x5);
    println!("  - Ocupación grilla fina (25x25): {:>3}/625 rayos ({:>5.1}%) -> [{}]",
        hits_fine, pct_fine, if toma6_passed { "PASS (>= 60%)" } else { "FAIL (< 60%)" });

    println!("\n--- 5. Verificación de Toma F4 (Portal centrado, meta ancho >= 15% de pantalla = 120 px) ---");
    let mut cam_f4 = f4_portal_camera();
    cam_f4.resolve_collision(&scene.cubes);
    cam_f4.update_basis_vectors();
    println!("Cámara F4: Eye = ({:.2}, {:.2}, {:.2}), Target = ({:.2}, {:.2}, {:.2})",
        cam_f4.eye.x, cam_f4.eye.y, cam_f4.eye.z, cam_f4.center.x, cam_f4.center.y, cam_f4.center.z);

    let mut portal_min_x = i32::MAX;
    let mut portal_max_x = i32::MIN;
    let mut portal_min_y = i32::MAX;
    let mut portal_max_y = i32::MIN;

    let portal_bb = scene.billboards.iter().find(|b| b.texture == "assets/gate_vortex.png").expect("Portal billboard no encontrado");

    for y in 0..HEIGHT {
        let py = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * tan_half_fov;
        for x in 0..WIDTH {
            let px = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect * tan_half_fov;
            let dir = cam_f4.basis_change(&Vector3::new(px, py, -1.0)).normalized();

            if let Some((t_bb, u, v)) = portal_bb.intersect(cam_f4.eye, dir, cam_f4.forward, cam_f4.right, cam_f4.up, f32::MAX) {
                let (_, alpha) = scene.textures.sample_uv_rgba(portal_bb.texture, u, v);
                if alpha >= 0.5 {
                    let occluded = if let Some((t_cube, cube_idx, _, _)) = scene.grid.intersect_closest(&scene.cubes, cam_f4.eye, dir, None) {
                        t_cube < t_bb - 0.05 && scene.cubes[cube_idx].material.transparency < 0.1
                    } else {
                        false
                    };

                    if !occluded {
                        if x < portal_min_x { portal_min_x = x; }
                        if x > portal_max_x { portal_max_x = x; }
                        if y < portal_min_y { portal_min_y = y; }
                        if y > portal_max_y { portal_max_y = y; }
                    }
                }
            }
        }
    }

    let f4_bbox_w = if portal_max_x >= portal_min_x { portal_max_x - portal_min_x + 1 } else { 0 };
    let f4_bbox_h = if portal_max_y >= portal_min_y { portal_max_y - portal_min_y + 1 } else { 0 };
    let f4_pct_w = (f4_bbox_w as f32 / WIDTH as f32) * 100.0;
    let f4_center_x = if f4_bbox_w > 0 { (portal_min_x + portal_max_x) as f32 * 0.5 } else { 0.0 };
    let f4_center_y = if f4_bbox_h > 0 { (portal_min_y + portal_max_y) as f32 * 0.5 } else { 0.0 };
    let f4_passed = f4_pct_w >= 15.0;

    println!("  - Portal BBox en F4 (medido con rayos): [X: {}..{}, Y: {}..{}] -> {}x{} px",
        portal_min_x, portal_max_x, portal_min_y, portal_max_y, f4_bbox_w, f4_bbox_h);
    println!("  - Centro BBox: ({:.1}, {:.1}) (centro pantalla: 400.0, 300.0)", f4_center_x, f4_center_y);
    println!("  - Ancho relativo: {:.2}% (meta: >= 15.0%, min 120 px) -> [{}]",
        f4_pct_w, if f4_passed { "PASS" } else { "FAIL" });

    println!("\n--- 6. Verificación de Tomas 3 a 9 (Personajes individuales: meta >= 95% visible, 55-70% alto) ---");
    let char_tests = [
        ("Chrono", "assets/party/Chrono.png", chrono_camera()),
        ("Marle", "assets/party/Marle.png", marle_camera()),
        ("Lucca", "assets/party/Lucca.png", lucca_camera()),
        ("Robo", "assets/party/Robo.png", robo_camera()),
        ("Frog", "assets/party/Frog.png", frog_camera()),
        ("Ayla", "assets/party/Ayla.png", ayla_camera()),
        ("Magus", "assets/party/Magus.png", magus_camera()),
    ];

    let mut all_chars_passed = true;
    for (char_name, tex_path, mut cam) in char_tests {
        cam.resolve_collision(&scene.cubes);
        cam.update_basis_vectors();
        let (bb_idx, bb) = scene.billboards.iter().enumerate().find(|(_, b)| b.texture == tex_path).expect("Personaje no encontrado");
        let sprite_center = bb.position + Vector3::new(0.0, bb.height * 0.5, 0.0);

        // Rayos 5x5 sobre el sprite
        let mut vis_count = 0;
        for gy in 0..5 {
            let v = (gy as f32 + 0.5) / 5.0;
            for gx in 0..5 {
                let u = (gx as f32 + 0.5) / 5.0;
                let sample_pos = sprite_center
                    + cam.right * ((u - 0.5) * bb.width)
                    + cam.up * ((0.5 - v) * bb.height);
                let ray_dir = (sample_pos - cam.eye).normalized();
                let target_dist = (sample_pos - cam.eye).length();

                let mut occluded = false;
                if let Some((t_cube, cube_idx, _, _)) = scene.grid.intersect_closest(&scene.cubes, cam.eye, ray_dir, None) {
                    if t_cube < target_dist - 0.05 && scene.cubes[cube_idx].material.transparency < 0.1 {
                        occluded = true;
                    }
                }

                if !occluded {
                    for (oidx, obb) in scene.billboards.iter().enumerate() {
                        if oidx == bb_idx || obb.texture == "assets/gate_vortex.png" {
                            continue;
                        }
                        if let Some((_, bu, bv)) = obb.intersect(cam.eye, ray_dir, cam.forward, cam.right, cam.up, target_dist - 0.05) {
                            let (_, alpha) = scene.textures.sample_uv_rgba(obb.texture, bu, bv);
                            if alpha >= 0.5 {
                                occluded = true;
                                break;
                            }
                        }
                    }
                }

                if !occluded {
                    vis_count += 1;
                }
            }
        }

        let vis_pct = (vis_count as f32 / 25.0) * 100.0;
        let dist = (cam.eye - sprite_center).length();
        let frame_h_at_dist = 2.0 * dist * tan_half_fov;
        let height_pct = (bb.height / frame_h_at_dist) * 100.0;

        let char_ok = vis_pct >= 95.0 && height_pct >= 55.0 && height_pct <= 70.0;
        if !char_ok {
            all_chars_passed = false;
        }

        println!("  - {:<7}: Eye = ({:.2}, {:.2}, {:.2}), Target = ({:.2}, {:.2}, {:.2}) | Visibilidad: {:>2}/25 ({:>5.1}%) | Alto: {:>4.1}% -> [{}]",
            char_name, cam.eye.x, cam.eye.y, cam.eye.z, cam.center.x, cam.center.y, cam.center.z,
            vis_count, vis_pct, height_pct, if char_ok { "PASS" } else { "FAIL" });
    }

    println!("\n=== RESUMEN GLOBAL ===");
    println!("1. Visibilidad personajes inicial (>= 85%): {}", if all_vis_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("2. Solape entre personajes inicial (<= 10%): {}", if all_overlap_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("3. Portal escondido (Toma 1=0%, Órbita 2-5 frames): {}", if portal_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("4. Toma 6 ocupación tronco (>= 60%): {}", if toma6_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("5. Toma F4 portal ancho (>= 15%): {}", if f4_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("6. Tomas 3-9 personajes (vis >= 95%, alto 55-70%): {}", if all_chars_passed { "TODO PASS" } else { "FAIL DETECTADO" });
    println!("Resultado: {}",
        if all_vis_passed && all_overlap_passed && portal_passed && toma6_passed && f4_passed && all_chars_passed {
            "PASS (Todos los criterios cumplidos)"
        } else {
            "FAIL (Ajuste requerido)"
        }
    );
}
