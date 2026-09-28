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

    // 1. Asegurar que todos los assets requeridos existan en disco
    texture_gen::generate_all_assets();

    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--screenshot");
    let rotated = args.iter().any(|a| a == "--rotated");

    // 2. Crear la escena y cargar texturas en CPU
    let mut scene = Scene::campfire_diorama();
    scene.load_textures(&mut rl, &thread);

    let mut camera = if rotated {
        Camera::new(
            Vector3::new(8.5, 5.8, 0.0),
            Vector3::new(-0.8, 3.2, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
    } else {
        Camera::new(
            Vector3::new(0.0, 5.8, 8.5),
            Vector3::new(0.0, 3.2, -0.8),
            Vector3::new(0.0, 1.0, 0.0),
        )
    };

    let mut framebuffer = vec![Color::BLACK; (WIDTH * HEIGHT) as usize];
    let mut screenshot_saved = false;

    while !rl.window_should_close() {
        // --- Toggle de depuración de mapas normales: tecla N ---
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            scene.use_normal_maps = !scene.use_normal_maps;
            println!("Mapas normales: {}", if scene.use_normal_maps { "ACTIVADOS" } else { "DESACTIVADOS" });
            camera.orbit(0.0, 0.0); // Marca camera como cambiada para forzar re-render
        }

        // --- Input de cámara: rotación con mouse/flechas, zoom con scroll ---
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(-1.5 * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(1.5 * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, 1.0 * dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, -1.0 * dt);
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            camera.zoom(wheel * 0.5);
        }

        // --- Render (paralelizado por filas con rayon) ---
        if camera.is_changed() {
            scene.camera_forward = camera.forward;
            scene.camera_right = camera.right;
            scene.camera_up = camera.up;
            render(&scene, &camera, &mut framebuffer);

            if !screenshot_saved {
                let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        img.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
                    }
                }
                let out_file = if rotated { "screenshot_rotated.png" } else { "screenshot.png" };
                img.export_image(out_file);
                screenshot_saved = true;
                if headless {
                    break;
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                d.draw_pixel(x, y, framebuffer[(y * WIDTH + x) as usize]);
            }
        }
        d.draw_fps(10, 10);
    }
}

/// Lanza un rayo por cada pixel. Paralelizado por filas: cada fila del
/// framebuffer se calcula en un hilo distinto vía rayon, sin necesidad
/// de tocar la GPU.
fn render(scene: &Scene, camera: &Camera, framebuffer: &mut [Color]) {
    let aspect = WIDTH as f32 / HEIGHT as f32;
    let fov = 60.0_f32.to_radians();

    framebuffer
        .par_chunks_mut(WIDTH as usize)
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..WIDTH as usize {
                let px = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect * (fov / 2.0).tan();
                let py = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * (fov / 2.0).tan();

                let dir = camera
                    .basis_change(&Vector3::new(px, py, -1.0))
                    .normalized();

                let color = raytrace::trace_ray(scene, camera.eye, dir, 0);
                row[x] = to_raylib_color(color);
            }
        });
}

fn to_raylib_color(c: Vector3) -> Color {
    Color::new(
        (c.x.clamp(0.0, 1.0) * 255.0) as u8,
        (c.y.clamp(0.0, 1.0) * 255.0) as u8,
        (c.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}
