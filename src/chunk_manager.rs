use crate::shader::ShaderProgram;
use crate::UVCoords;
use crate::{
    chunk::{BlockID, Chunk},
    shapes::write_unit_cube_to_ptr,
};
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use noise::{NoiseFn, SuperSimplex};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use crate::block_texture_sides::{BlockFaces, get_uv_every_side};
use rand::random;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_VOLUME: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub type Sides = [bool; 6];

pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl ChunkManager {
    pub fn new() -> ChunkManager {
        ChunkManager {
            loaded_chunks: HashMap::new(),
        }
    }

    pub fn preload_some_chunks(&mut self) {
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    self.loaded_chunks.insert((x, y, z), Chunk::random());
                }
            }
        }
    }

    pub fn simplex(&mut self) {
        let ss = SuperSimplex::new(1296);
        let n = 10;

        for y in 0..16 {
            for z in -n..=n {
                for x in -n..=n {
                    self.loaded_chunks.insert((x, y, z), Chunk::empty());
                }
            }
        }

        for x in -16 * n..16 * n {
            for z in -16 * n..16 * n {
                let (xf, zf) = (x as f64 / 64.0, z as f64 / 64.0);
                let y = ss.get([xf, zf]);
                let y = (16.0 * (y + 1.0)) as i32;

                self.set_block(x, y, z, BlockID::GrassBlock);
                self.set_block(x, y - 1, z, BlockID::Dirt);
                self.set_block(x, y - 2, z, BlockID::Dirt);
                self.set_block(x, y - 3, z, BlockID::Cobblestone);

                if random::<u32>() % 100 == 0 {
                    let h = 5;
                    for i in y+1..y+1+h{
                        self.set_block(x, i, z, BlockID::OakLog)
                    }

                    for yy in y + h - 2..=y + h - 1{
                        for xx in x - 2..=x + 2{
                            for zz in z - 2..=z + 2{
                                if xx != x || zz != z {
                                    self.set_block(xx, yy, zz, BlockID::OakLeaves);
                                }
                            }
                        }
                    }

                    for xx in x - 1..=x + 1{
                        for zz in z - 1..=z + 1{
                            if xx != x || zz != z {
                                self.set_block(xx, y + h, zz, BlockID::OakLeaves);
                            }
                        }
                    }

                    self.set_block(x, y + h + 1, z, BlockID::OakLeaves);
                    self.set_block(x + 1, y + h + 1, z, BlockID::OakLeaves);
                    self.set_block(x - 1, y + h + 1, z, BlockID::OakLeaves);
                    self.set_block(x, y + h + 1, z + 1, BlockID::OakLeaves);
                    self.set_block(x, y + h + 1, z - 1, BlockID::OakLeaves);
                }
            }
        }
    }

    fn get_chunk_and_block_coords(x: i32, y: i32, z: i32) -> (i32, i32, i32, u32, u32, u32) {
        let chunk_x = if x < 0 { (x + 1) / 16 - 1 } else { x / 16 };
        let chunk_y = if y < 0 { (y + 1) / 16 - 1 } else { y / 16 };
        let chunk_z = if z < 0 { (z + 1) / 16 - 1 } else { z / 16 };

        let block_x = x.rem_euclid(16) as u32; // (음수) % (양수)의 결과를 0 또는 양수로 고정
        let block_y = y.rem_euclid(16) as u32;
        let block_z = z.rem_euclid(16) as u32;

        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z)
    }

    fn get_global_coords(
        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z): (i32, i32, i32, u32, u32, u32),
    ) -> (i32, i32, i32) {
        let x = 16 * chunk_x + block_x as i32;
        let y = 16 * chunk_y + block_y as i32;
        let z = 16 * chunk_z + block_z as i32;

        (x, y, z)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockID> {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_and_block_coords(x, y, z);

        self.loaded_chunks
            .get((chunk_x, chunk_y, chunk_z).borrow())
            .and_then(|chunk| Some(chunk.get_block(block_x, block_y, block_z)))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockID) {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_and_block_coords(x, y, z);

        self.loaded_chunks
            .get_mut((chunk_x, chunk_y, chunk_z).borrow())
            .map(|chunk| chunk.set_block(block_x, block_y, block_z, block));
    }

    pub fn rebuild_dirty_chunks(&mut self, uv_map: &HashMap<BlockID, BlockFaces<UVCoords>>) {
        let mut dirty_chunks = HashSet::new();

        // Nearby chunks can be also dirty if the change happens at the edge
        for (&(x, y, z), chunk) in self.loaded_chunks.iter() {
            if chunk.dirty {
                dirty_chunks.insert((x, y, z));
            }

            for &(dx, dy, dz) in chunk.dirty_neighbours.iter() {
                dirty_chunks.insert((x + dx, y + dy, z + dz));
            }
        }

        let mut active_sides: HashMap<(i32, i32, i32), Vec<Sides>> = HashMap::new();

        for &coords in dirty_chunks.iter() {
            let (cx, cy, cz) = coords;
            let chunk = self.loaded_chunks.get(&coords);

            if let Some(chunk) = chunk {
                let sides_vec = active_sides.entry(coords).or_default();

                for by in 0..CHUNK_SIZE {
                    for bz in 0..CHUNK_SIZE {
                        for bx in 0..CHUNK_SIZE {
                            if !chunk.get_block(bx, by, bz).is_air(){
                                let (gx, gy, gz) =
                                    ChunkManager::get_global_coords((cx, cy, cz, bx, by, bz));
                                sides_vec.push(self.get_active_sides_of_block(gx, gy, gz));
                            }
                        }
                    }
                }
            }
        }

        for coords in dirty_chunks.iter() {
            let chunk = self.loaded_chunks.get_mut(coords);

            if let Some(chunk) = chunk {
                chunk.dirty = false;
                chunk.dirty_neighbours.clear();
                chunk.vertices_drawn = 0;

                let sides = active_sides.get(coords).unwrap();
                let n_visible_faces = sides.iter().map(|faces| faces.iter().fold(0, |acc, &x| acc + x as u32)).fold(0, |acc,  n| acc + n); 

                if n_visible_faces == 0 {
                    continue;
                }

                gl_call!(gl::NamedBufferData(
                    chunk.vbo,
                    (180 * std::mem::size_of::<f32>() * n_visible_faces as usize) as isize,
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW
                ));

                let vbo_ptr = gl_call!(gl::MapNamedBuffer(chunk.vbo, gl::WRITE_ONLY)) as *mut f32;
                let mut idx = 0;

                let sides_vec = active_sides.get(coords).unwrap();
                let mut cnt = 0;

                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let block = chunk.get_block(x, y, z);

                            if block != BlockID::Air {
                                let active_sides = sides_vec[cnt];

                                let uvs = uv_map.get(&block).unwrap().clone();
                                let uvs = get_uv_every_side(uvs);

                                let copied_vertices = unsafe { write_unit_cube_to_ptr(
                                    vbo_ptr.offset(idx), (x as f32, y as f32, z as f32), uvs, active_sides)};

                                chunk.vertices_drawn += copied_vertices;
                                idx += copied_vertices as isize * 5;
                                cnt += 1;
                            }
                        }
                    }
                }
                gl_call!(gl::UnmapNamedBuffer(chunk.vbo));
            }
        }
    }

    pub fn get_active_sides_of_block(&self, x: i32, y: i32, z: i32) -> Sides {
        let right = self
            .get_block(x + 1, y, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let left = self
            .get_block(x - 1, y, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let top = self
            .get_block(x, y + 1, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let bottom = self
            .get_block(x, y - 1, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let front = self
            .get_block(x, y, z + 1)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let back = self
            .get_block(x, y, z - 1)
            .filter(|&b| !b.is_transparent())
            .is_none();

        [right, left, top, bottom, front, back]
    }

    pub fn render_loaded_chunks(&mut self, program: &mut ShaderProgram) {
        for ((x, y, z), chunk) in &self.loaded_chunks {
            // skip rendering the chunk if there is nothing to draw
            if chunk.vertices_drawn == 0 {
                continue;
            }
            let model_matrix = {
                let translate_matrix =
                    Matrix4::new_translation(&vec3(*x as f32, *y as f32, *z as f32).scale(16.0));
                let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);
                let scale_matrix = Matrix4::new_nonuniform_scaling(&vec3(1.0f32, 1.0f32, 1.0f32));

                translate_matrix * rotate_matrix * scale_matrix
            };

            gl_call!(gl::BindVertexArray(chunk.vao));
            program.set_uniform_matrix4fv("model", model_matrix.as_ptr());
            gl_call!(gl::DrawArrays(
                gl::TRIANGLES,
                0,
                chunk.vertices_drawn as i32
            ));
        }
    }
}
