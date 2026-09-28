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

    let scene = Scene::campfire_diorama();

    let mut camera = Camera::new(
        Vector3::new(0.0, 8.0, 16.0),
        Vector3::new(0.0, 2.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let mut framebuffer = vec![Color::BLACK; (WIDTH * HEIGHT) as usize];

    while !rl.window_should_close() {
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
            render(&scene, &camera, &mut framebuffer);
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
