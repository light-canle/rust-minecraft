use crate::gl_call;
use std::os::raw::c_void;

#[derive(Clone)]
pub struct QuadProps{
    pub position : (f32, f32),
    pub size : (f32, f32),
    pub color : (f32, f32, f32, f32),
}

pub struct Renderer{
    vertices : Vec<f32>,
    vbo : u32,
    vao : u32,
}

impl Renderer{
    pub fn new(capacity: usize) -> Self{
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
            2,      // 크기 (float 개수)
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
        // rgba 색상
        gl_call!(gl::EnableVertexArrayAttrib(vao, 1));

        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            (2 * std::mem::size_of::<f32>()) as u32
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
            vertices, 
            vbo, 
            vao, 
        }
    }
    pub fn begin_batch(&mut self){
        self.vertices.clear();
    }

    pub fn submit_quad(&mut self, quad_props : QuadProps){
        let QuadProps { position : (x, y), size : (w, h), color : (r, g, b, a)} = quad_props;
        // 사각형의 정보를 넣음
        self.vertices.extend_from_slice(&[x, y, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x, y, r, g, b, a]);
    }

    pub fn end_batch(&mut self){
        // 데이터를 VBO에 넣음
        gl_call!(gl::NamedBufferSubData(
            self.vbo,
            0 as isize,
            (self.vertices.len() * std::mem::size_of::<f32>()) as isize,
            self.vertices.as_ptr() as *mut c_void
        ));
        // VAO를 GPU에 연결
        gl_call!(gl::BindVertexArray(self.vao));
        // 도형을 실제로 그림
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32));
    }
}