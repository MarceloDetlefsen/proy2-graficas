use raylib::prelude::Vector3;
use crate::cube::Cube;

/// Estructura de aceleración basada en un Grid 3D uniforme con recorrido DDA 3D (Amanatides-Woo).
/// Acelera masivamente el trazado de rayos primarios, reflejados, refractados y de sombra.
pub struct VoxelGrid {
    pub min: Vector3,
    pub max: Vector3,
    pub cell_size: f32,
    pub dim_x: usize,
    pub dim_y: usize,
    pub dim_z: usize,
    pub cells: Vec<Vec<usize>>,
}

impl VoxelGrid {
    pub fn build(cubes: &[Cube]) -> Self {
        if cubes.is_empty() {
            return VoxelGrid {
                min: Vector3::zero(),
                max: Vector3::one(),
                cell_size: 1.0,
                dim_x: 1,
                dim_y: 1,
                dim_z: 1,
                cells: vec![Vec::new()],
            };
        }

        let mut min_bound = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_bound = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for cube in cubes {
            min_bound.x = min_bound.x.min(cube.min.x);
            min_bound.y = min_bound.y.min(cube.min.y);
            min_bound.z = min_bound.z.min(cube.min.z);
            max_bound.x = max_bound.x.max(cube.max.x);
            max_bound.y = max_bound.y.max(cube.max.y);
            max_bound.z = max_bound.z.max(cube.max.z);
        }

        let cell_size = 1.0f32;
        let min_x = min_bound.x.floor() as i32 - 1;
        let min_y = min_bound.y.floor() as i32 - 1;
        let min_z = min_bound.z.floor() as i32 - 1;
        let max_x = max_bound.x.ceil() as i32 + 1;
        let max_y = max_bound.y.ceil() as i32 + 1;
        let max_z = max_bound.z.ceil() as i32 + 1;

        let dim_x = (max_x - min_x).max(1) as usize;
        let dim_y = (max_y - min_y).max(1) as usize;
        let dim_z = (max_z - min_z).max(1) as usize;

        let grid_min = Vector3::new(min_x as f32, min_y as f32, min_z as f32);
        let grid_max = grid_min + Vector3::new(dim_x as f32, dim_y as f32, dim_z as f32) * cell_size;

        let mut cells = vec![Vec::new(); dim_x * dim_y * dim_z];

        for (cube_idx, cube) in cubes.iter().enumerate() {
            let c_min_x = (((cube.min.x - grid_min.x) / cell_size).floor() as i32).clamp(0, dim_x as i32 - 1) as usize;
            let c_max_x = (((cube.max.x - grid_min.x) / cell_size).ceil() as i32).clamp(1, dim_x as i32) as usize;
            let c_min_y = (((cube.min.y - grid_min.y) / cell_size).floor() as i32).clamp(0, dim_y as i32 - 1) as usize;
            let c_max_y = (((cube.max.y - grid_min.y) / cell_size).ceil() as i32).clamp(1, dim_y as i32) as usize;
            let c_min_z = (((cube.min.z - grid_min.z) / cell_size).floor() as i32).clamp(0, dim_z as i32 - 1) as usize;
            let c_max_z = (((cube.max.z - grid_min.z) / cell_size).ceil() as i32).clamp(1, dim_z as i32) as usize;

            for z in c_min_z..c_max_z {
                for y in c_min_y..c_max_y {
                    for x in c_min_x..c_max_x {
                        let idx = z * (dim_x * dim_y) + y * dim_x + x;
                        cells[idx].push(cube_idx);
                    }
                }
            }
        }

        VoxelGrid {
            min: grid_min,
            max: grid_max,
            cell_size,
            dim_x,
            dim_y,
            dim_z,
            cells,
        }
    }

    /// Intersección con el AABB completo del grid (slab method).
    #[inline(always)]
    fn intersect_grid_aabb(&self, origin: Vector3, dir: Vector3) -> Option<(f32, f32)> {
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;

        // Eje X
        if dir.x.abs() > 1e-8 {
            let inv_dx = 1.0 / dir.x;
            let mut t1 = (self.min.x - origin.x) * inv_dx;
            let mut t2 = (self.max.x - origin.x) * inv_dx;
            if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
            t_near = t_near.max(t1);
            t_far = t_far.min(t2);
            if t_near > t_far { return None; }
        } else if origin.x < self.min.x || origin.x > self.max.x {
            return None;
        }

        // Eje Y
        if dir.y.abs() > 1e-8 {
            let inv_dy = 1.0 / dir.y;
            let mut t1 = (self.min.y - origin.y) * inv_dy;
            let mut t2 = (self.max.y - origin.y) * inv_dy;
            if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
            t_near = t_near.max(t1);
            t_far = t_far.min(t2);
            if t_near > t_far { return None; }
        } else if origin.y < self.min.y || origin.y > self.max.y {
            return None;
        }

        // Eje Z
        if dir.z.abs() > 1e-8 {
            let inv_dz = 1.0 / dir.z;
            let mut t1 = (self.min.z - origin.z) * inv_dz;
            let mut t2 = (self.max.z - origin.z) * inv_dz;
            if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
            t_near = t_near.max(t1);
            t_far = t_far.min(t2);
            if t_near > t_far { return None; }
        } else if origin.z < self.min.z || origin.z > self.max.z {
            return None;
        }

        if t_far < 1e-4 {
            None
        } else {
            Some((t_near, t_far))
        }
    }

    /// Recorrido DDA 3D (Amanatides-Woo) buscando el impacto más cercano.
    /// Devuelve (t, cube_index, u, v) si impacta algún cubo.
    pub fn intersect_closest(&self, cubes: &[Cube], origin: Vector3, dir: Vector3) -> Option<(f32, usize, f32, f32)> {
        let (t_near, _t_far) = self.intersect_grid_aabb(origin, dir)?;

        let t_start = t_near.max(0.0);
        let p_start = origin + dir * (t_start + 1e-4);

        let mut gx = (((p_start.x - self.min.x) / self.cell_size).floor() as i32).clamp(0, self.dim_x as i32 - 1);
        let mut gy = (((p_start.y - self.min.y) / self.cell_size).floor() as i32).clamp(0, self.dim_y as i32 - 1);
        let mut gz = (((p_start.z - self.min.z) / self.cell_size).floor() as i32).clamp(0, self.dim_z as i32 - 1);

        let (step_x, mut t_next_x, t_delta_x) = if dir.x > 1e-8 {
            let next_boundary = self.min.x + (gx + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.x) / dir.x, self.cell_size / dir.x)
        } else if dir.x < -1e-8 {
            let next_boundary = self.min.x + gx as f32 * self.cell_size;
            (-1, (next_boundary - origin.x) / dir.x, -self.cell_size / dir.x)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        let (step_y, mut t_next_y, t_delta_y) = if dir.y > 1e-8 {
            let next_boundary = self.min.y + (gy + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.y) / dir.y, self.cell_size / dir.y)
        } else if dir.y < -1e-8 {
            let next_boundary = self.min.y + gy as f32 * self.cell_size;
            (-1, (next_boundary - origin.y) / dir.y, -self.cell_size / dir.y)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        let (step_z, mut t_next_z, t_delta_z) = if dir.z > 1e-8 {
            let next_boundary = self.min.z + (gz + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.z) / dir.z, self.cell_size / dir.z)
        } else if dir.z < -1e-8 {
            let next_boundary = self.min.z + gz as f32 * self.cell_size;
            (-1, (next_boundary - origin.z) / dir.z, -self.cell_size / dir.z)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        let mut closest_hit: Option<(f32, usize, f32, f32)> = None;

        loop {
            let cell_idx = (gz as usize) * (self.dim_x * self.dim_y) + (gy as usize) * self.dim_x + (gx as usize);
            let cell_cubes = &self.cells[cell_idx];

            for &ci in cell_cubes {
                if let Some((t, u, v)) = cubes[ci].intersect(origin, dir) {
                    if t > 1e-4 {
                        if let Some((best_t, _, _, _)) = closest_hit {
                            if t < best_t {
                                closest_hit = Some((t, ci, u, v));
                            }
                        } else {
                            closest_hit = Some((t, ci, u, v));
                        }
                    }
                }
            }

            let t_next_cell = t_next_x.min(t_next_y).min(t_next_z);
            if let Some((hit_t, _, _, _)) = closest_hit {
                if hit_t <= t_next_cell {
                    return closest_hit;
                }
            }

            if t_next_x < t_next_y {
                if t_next_x < t_next_z {
                    gx += step_x;
                    if gx < 0 || gx >= self.dim_x as i32 { break; }
                    t_next_x += t_delta_x;
                } else {
                    gz += step_z;
                    if gz < 0 || gz >= self.dim_z as i32 { break; }
                    t_next_z += t_delta_z;
                }
            } else {
                if t_next_y < t_next_z {
                    gy += step_y;
                    if gy < 0 || gy >= self.dim_y as i32 { break; }
                    t_next_y += t_delta_y;
                } else {
                    gz += step_z;
                    if gz < 0 || gz >= self.dim_z as i32 { break; }
                    t_next_z += t_delta_z;
                }
            }
        }

        closest_hit
    }

    /// Recorrido DDA 3D para rayos de sombra con corte en primer impacto (any-hit).
    /// Devuelve `true` inmediatamente en cuanto un cubo opaco bloquea la luz.
    pub fn is_occluded(&self, cubes: &[Cube], origin: Vector3, dir: Vector3, max_dist: f32) -> bool {
        let (t_near, _t_far) = match self.intersect_grid_aabb(origin, dir) {
            Some(range) => range,
            None => return false,
        };

        if t_near >= (max_dist - 1e-3) {
            return false;
        }

        let t_start = t_near.max(0.0);
        let p_start = origin + dir * (t_start + 1e-4);

        let mut gx = (((p_start.x - self.min.x) / self.cell_size).floor() as i32).clamp(0, self.dim_x as i32 - 1);
        let mut gy = (((p_start.y - self.min.y) / self.cell_size).floor() as i32).clamp(0, self.dim_y as i32 - 1);
        let mut gz = (((p_start.z - self.min.z) / self.cell_size).floor() as i32).clamp(0, self.dim_z as i32 - 1);

        let (step_x, mut t_next_x, t_delta_x) = if dir.x > 1e-8 {
            let next_boundary = self.min.x + (gx + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.x) / dir.x, self.cell_size / dir.x)
        } else if dir.x < -1e-8 {
            let next_boundary = self.min.x + gx as f32 * self.cell_size;
            (-1, (next_boundary - origin.x) / dir.x, -self.cell_size / dir.x)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        let (step_y, mut t_next_y, t_delta_y) = if dir.y > 1e-8 {
            let next_boundary = self.min.y + (gy + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.y) / dir.y, self.cell_size / dir.y)
        } else if dir.y < -1e-8 {
            let next_boundary = self.min.y + gy as f32 * self.cell_size;
            (-1, (next_boundary - origin.y) / dir.y, -self.cell_size / dir.y)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        let (step_z, mut t_next_z, t_delta_z) = if dir.z > 1e-8 {
            let next_boundary = self.min.z + (gz + 1) as f32 * self.cell_size;
            (1, (next_boundary - origin.z) / dir.z, self.cell_size / dir.z)
        } else if dir.z < -1e-8 {
            let next_boundary = self.min.z + gz as f32 * self.cell_size;
            (-1, (next_boundary - origin.z) / dir.z, -self.cell_size / dir.z)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        };

        loop {
            let cell_idx = (gz as usize) * (self.dim_x * self.dim_y) + (gy as usize) * self.dim_x + (gx as usize);
            let cell_cubes = &self.cells[cell_idx];

            for &ci in cell_cubes {
                let cube = &cubes[ci];
                if !cube.casts_shadow || cube.material.transparency > 0.7 {
                    continue;
                }
                if let Some((t, _, _)) = cube.intersect(origin, dir) {
                    if t > 1e-3 && t < (max_dist - 1e-3) {
                        return true; // Any-hit inmediato
                    }
                }
            }

            let t_next_cell = t_next_x.min(t_next_y).min(t_next_z);
            if t_next_cell >= (max_dist - 1e-3) {
                return false;
            }

            if t_next_x < t_next_y {
                if t_next_x < t_next_z {
                    gx += step_x;
                    if gx < 0 || gx >= self.dim_x as i32 { break; }
                    t_next_x += t_delta_x;
                } else {
                    gz += step_z;
                    if gz < 0 || gz >= self.dim_z as i32 { break; }
                    t_next_z += t_delta_z;
                }
            } else {
                if t_next_y < t_next_z {
                    gy += step_y;
                    if gy < 0 || gy >= self.dim_y as i32 { break; }
                    t_next_y += t_delta_y;
                } else {
                    gz += step_z;
                    if gz < 0 || gz >= self.dim_z as i32 { break; }
                    t_next_z += t_delta_z;
                }
            }
        }

        false
    }
}
