use crate::gl_call;
use crate::shader::ShaderProgram;
use std::cmp::Ordering;
use std::os::raw::c_void;
use std::collections::HashMap;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct QuadProps{
    pub position : (f32, f32, f32),
    pub size : (f32, f32),
    pub texture_id : u32,
    pub texture_coords: (f32, f32, f32, f32),
}

pub struct Renderer{
    texture_units: u32,
    quads: HashMap<u32, Vec<QuadProps>>,
    vertices : Vec<f32>,
    vbo : u32,
    vao : u32,
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer::new(1_000_000)
    }
}

impl Renderer{
    pub fn new(capacity: usize) -> Self{
        let mut texture_units = 0;
        gl_call!(gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut texture_units));
        assert!(texture_units > 0);

        let texture_units = texture_units as u32;
        let quads : HashMap<u32, Vec<QuadProps>> = HashMap::new();
        
        let mut vertices = Vec::new();
        vertices.reserve(capacity);

        // VBO 설정
        let mut vbo = 0;
        gl_call!(gl::CreateBuffers(1, &mut vbo));

        gl_call!(gl::NamedBufferData(
            vbo, 
            (capacity * std::mem::size_of::<f32>()) as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW
        ));

        // VAO 설정
        let mut vao : u32 = 0;
        let binding_index_pos = 0;
        let binding_index_color = 1;

        gl_call!(gl::CreateVertexArrays(1, &mut vao));

        // x,y 위치
        gl_call!(gl::EnableVertexArrayAttrib(vao, 0));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,    // vao 객체
            0,      // 속성 번호
            3,      // 크기 (float 개수)
            gl::FLOAT, // 데이터 타입
            gl::FALSE, // 정규화 여부
            0       // VBO에서의 시작 위치
        ));

        gl_call!(gl::VertexArrayAttribBinding(vao, 0 ,binding_index_pos));
        gl_call!(gl::VertexArrayVertexBuffer(
            vao, // VAO
            binding_index_pos, // 속성 인덱스
            vbo, // VBO
            0, // offset
            (6 * std::mem::size_of::<f32>()) as i32 // 정점 1개의 데이터 크기
        ));
        // 텍스쳐 정보
        gl_call!(gl::EnableVertexArrayAttrib(vao, 1));

        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as u32
        ));

        gl_call!(gl::VertexArrayAttribBinding(vao, 1 ,binding_index_color));

        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            binding_index_color,
            vbo,
            0,
            (6 * std::mem::size_of::<f32>()) as i32
        ));

        Renderer {
            texture_units,
            quads,
            vertices, 
            vbo, 
            vao,
        }
    }
    pub fn begin_batch(&mut self){
        self.quads.clear();
        self.vertices.clear();
    }

    pub fn submit_quad(&mut self, quad_props : QuadProps){
        match self.quads.get_mut(&quad_props.texture_id){
            Some(quads) => quads,
            None => {
                self.quads.insert(quad_props.texture_id, Vec::new());
                self.quads.get_mut(&quad_props.texture_id).unwrap()
            }
        }.push(quad_props);
    }

    pub fn end_batch(&mut self,  program: &mut ShaderProgram) {
        let mut draw_calls = 0;

        // TODO: Handle quads without textures

        for vec in self.quads.values_mut() {
            vec.sort_by(|a, b| {
                if a.position.2 < b.position.2 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
        }
        // 청크 단위로 나누어서 텍스쳐를 불러옴
        let chunks = &self.quads.keys().chunks(self.texture_units as usize);

        for chunk in chunks {
            let mut tex_units = Vec::new();
            self.vertices.clear();

            for (tex_unit, &texture_id) in chunk.enumerate() {
                for quad in &self.quads[&texture_id] {
                    let QuadProps {
                        position: (x, y, z),
                        size: (w, h),
                        texture_id: _,
                        texture_coords: (tex_x_min, tex_y_min, tex_x_max, tex_y_max),
                    } = *quad;

                    let tex_unit = tex_unit as f32;
                    self.vertices
                        .extend_from_slice(&[x, y, z, tex_unit, tex_x_min, tex_y_min]);
                    self.vertices
                        .extend_from_slice(&[x + w, y, z, tex_unit, tex_x_max, tex_y_min]);
                    self.vertices.extend_from_slice(&[
                        x + w,
                        y + h,
                        z,
                        tex_unit,
                        tex_x_max,
                        tex_y_max,
                    ]);
                    self.vertices.extend_from_slice(&[
                        x + w,
                        y + h,
                        z,
                        tex_unit,
                        tex_x_max,
                        tex_y_max,
                    ]);
                    self.vertices
                        .extend_from_slice(&[x, y + h, z, tex_unit, tex_x_min, tex_y_max]);
                    self.vertices
                        .extend_from_slice(&[x, y, z, tex_unit, tex_x_min, tex_y_min]);
                }

                gl_call!(gl::BindTextureUnit(tex_unit as u32, texture_id));
                tex_units.push(tex_unit as i32);
            }

            program.set_uniform1iv("textures", tex_units.as_slice());

            gl_call!(gl::NamedBufferSubData(
                self.vbo,
                0 as isize,
                (self.vertices.len() * std::mem::size_of::<f32>()) as isize,
                self.vertices.as_ptr() as *mut c_void
            ));

            gl_call!(gl::BindVertexArray(self.vao));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32));

            draw_calls += 1;
        }

        println!("Total draw calls: {draw_calls}");
    }
}