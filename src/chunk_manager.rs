use crate::{chunk::{BlockID, Chunk}, shapes::unit_cube_array};
use std::collections::{HashMap, HashSet};
use crate::shader::ShaderProgram;
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use std::borrow::Borrow;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_VOLUME: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub type Sides = (bool, bool, bool, bool, bool, bool);

pub struct ChunkManager{
    pub loaded_chunks : HashMap<(i32, i32, i32), Chunk>,
}

impl ChunkManager{
    pub fn new() -> ChunkManager{
        ChunkManager{
            loaded_chunks : HashMap::new(),
        }
    }

    pub fn preload_some_chunks(&mut self){
        for y in 0..10 {
            for z in 0..10 {
                for x in 0..10 {
                    self.loaded_chunks.insert((x, y, z), Chunk::random());
                }
            }
        }
    }

    fn get_chunk_and_block_coords(x : i32, y : i32, z : i32) -> (i32, i32, i32, u32, u32, u32){
        let chunk_x = if x < 0 { (x + 1) / 16 - 1 } else { x / 16 };
        let chunk_y = if y < 0 { (y + 1) / 16 - 1 } else { y / 16 }; 
        let chunk_z = if z < 0 { (z + 1) / 16 - 1 } else { z / 16 };

        let block_x = x.rem_euclid(16) as u32; // (음수) % (양수)의 결과를 0 또는 양수로 고정
        let block_y = y.rem_euclid(16) as u32;
        let block_z = z.rem_euclid(16) as u32;

        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z)
    }

    fn get_global_coords((chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) : (i32, i32, i32, u32, u32, u32)) -> (i32, i32, i32){
        let x = 16 * chunk_x + block_x as i32;
        let y = 16 * chunk_y + block_y as i32;
        let z = 16 * chunk_z + block_z as i32;

        (x, y, z)
    }

    pub fn get(&self, x : i32, y : i32, z : i32) -> Option<BlockID> {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) = ChunkManager::get_chunk_and_block_coords(x, y, z);

        self.loaded_chunks
        .get((chunk_x, chunk_y, chunk_z).borrow())
        .and_then(|chunk| Some(chunk.get_block(block_x, block_y, block_z)))
    }

    pub fn set(&mut self, x : i32, y : i32, z : i32, block : BlockID){
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) = ChunkManager::get_chunk_and_block_coords(x, y, z);

        self.loaded_chunks.get_mut((chunk_x, chunk_y, chunk_z).borrow())
        .map(|chunk| chunk.set_block(block_x, block_y, block_z, block));
    }

    pub fn rebuild_dirty_chunks(&mut self, uv_map : &HashMap<BlockID, ((f32, f32), (f32, f32))>){
        let mut dirty_chunks = HashSet::new();

        // Nearby chunks can be also dirty if the change happens at the edge
        for (&(x, y, z), chunk) in self.loaded_chunks.iter(){
            if chunk.dirty{
                dirty_chunks.insert((x, y, z));
            }

            for &(dx, dy, dz) in chunk.dirty_neighbours.iter(){
                dirty_chunks.insert((x + dx, y + dy, z + dz));
            }
        }

        let mut active_sides : HashMap<(i32, i32, i32), Vec<Sides>> = HashMap::new();

        for &coords in dirty_chunks.iter() {
            let (cx, cy, cz) = coords;
            let chunk = self.loaded_chunks.get(&coords);

            if chunk.is_some() {
                let sides_vec = active_sides.entry(coords).or_default();

                for by in 0..CHUNK_SIZE {
                    for bz in 0..CHUNK_SIZE {
                        for bx in 0..CHUNK_SIZE {
                            let (gx, gy, gz) = ChunkManager::get_global_coords((cx, cy, cz, bx, by, bz));
                            sides_vec.push(self.get_active_sides_of_block(gx, gy, gz));
                        }
                    }
                }
            }
        }

        for coords in dirty_chunks.iter(){
            let mut idx = 0;
            let chunk = self.loaded_chunks.get_mut(coords);

            if let Some(chunk) = chunk {
                let sides_vec = active_sides.get(coords).unwrap();
                let mut cnt = 0;
                
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let block = chunk.get_block(x, y, z);

                            if block != BlockID::AIR {
                                let (uv_bl, uv_tr) = *uv_map.get(&block).unwrap();
                                let active_sides = sides_vec[cnt];
                                let cube_array = unit_cube_array(
                                    (x as f32, y as f32, z as f32), 
                                    uv_bl, 
                                    uv_tr, 
                                    active_sides,
                                );
                                gl_call!(gl::NamedBufferSubData(
                                    chunk.vbo,
                                    (idx * std::mem::size_of::<f32>()) as isize,
                                    (cube_array.len() * std::mem::size_of::<f32>()) as isize,
                                    cube_array.as_ptr() as *const std::ffi::c_void
                                ));
                                chunk.vertices_drawn += (cube_array.len() as u32) / 5;

                                idx += cube_array.len();
                            }

                            cnt += 1;
                        }
                    }
                }
                chunk.dirty = false;
                chunk.dirty_neighbours.clear();
            } 
        }
    }

    pub fn get_active_sides_of_block(&self, x : i32, y : i32, z : i32) -> Sides {
        let right = self.get(x + 1, y, z).filter(|&b| b != BlockID::AIR).is_none();
        let left = self.get(x - 1, y, z).filter(|&b| b != BlockID::AIR).is_none();
        let top = self.get(x, y + 1, z).filter(|&b| b != BlockID::AIR).is_none();
        let bottom = self.get(x, y - 1, z).filter(|&b| b != BlockID::AIR).is_none();
        let front = self.get(x, y, z + 1).filter(|&b| b != BlockID::AIR).is_none();
        let back = self.get(x, y, z - 1).filter(|&b| b != BlockID::AIR).is_none();

        (right, left, top, bottom, front, back)
    }

    pub fn render_loaded_chunks(&mut self, program : &mut ShaderProgram){
        for ((x, y , z), chunk) in &self.loaded_chunks {
            let model_matrix = {
                let translate_matrix = 
                    Matrix4::new_translation(&vec3(*x as f32, *y as f32, *z as f32).scale(16.0));
                let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);
                let scale_matrix = Matrix4::new_nonuniform_scaling(&vec3(1.0f32, 1.0f32, 1.0f32));

                translate_matrix * rotate_matrix * scale_matrix
            };

            gl_call!(gl::BindVertexArray(chunk.vao));
            program.set_uniform_matrix4fv("model", model_matrix.as_ptr());
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, chunk.vertices_drawn as i32));
        }
    }
}