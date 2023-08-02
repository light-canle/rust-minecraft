pub mod renderer;
pub mod debugging;

use renderer::{QuadProps, Renderer};

use glfw::ffi::{glfwSwapInterval, glfwGetTime};
use glfw::Key;
use glfw::{Context, WindowHint};
use rand::Rng;

#[derive(Default)]
pub struct FrameRate{
    pub frame_count : u32,
    pub last_frame_time : f64
}

impl FrameRate{
    fn run(&mut self){
        self.frame_count += 1;
        
        let current_time = unsafe { glfwGetTime() };
        let delta : f64 = current_time - self.last_frame_time;

        if delta >= 1.0 {
            self.last_frame_time = current_time;
            println!("FPS : {}", f64::from(self.frame_count) / delta);
            self.frame_count = 0;
        }
    }
}

fn main() {
    // glfw 초기화
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // glfw 힌트
    glfw.window_hint(WindowHint::ContextVersion(4, 6));
    glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core,));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));
    // 윈도우 크기 설정
    let window_size = (500, 500);
    let window_title = "Minecraft";

    // 윈도우 창 생성
    let (mut window, events) = glfw
    .create_window(
        window_size.0,
        window_size.1, 
        window_title, 
        glfw::WindowMode::Windowed,
    )
    .expect("Failed to create GLFW window");

    // 윈도우의 context 설정
    window.make_current();
    // 이벤트 poll 설정
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_raw_mouse_motion(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    // 수직 동기화(Vsync)
    unsafe{glfwSwapInterval(0)};

    let mut renderer = Renderer::new(100000);

    // fps 객체 생성
    let mut framerate = FrameRate {
        frame_count : 0,
        last_frame_time : 0.0
    };

    let mut quads = Vec::new();
    let mut rng = rand::thread_rng();

    // 메인 루프
    while !window.should_close() {
        // 이벤트를 받고 처리
        glfw.poll_events();

        for(_, event) in glfw::flush_messages(&events){
            match event{
                glfw::WindowEvent::Key(Key::Space, _, _, _) => quads.push(QuadProps {
                    position : (
                        (window.get_cursor_pos().0 as f32).to_range(0.0, 500.0, -1.0, 1.0),
                        (window.get_cursor_pos().1 as f32).to_range(0.0, 500.0, 1.0, -1.0)
                    ),
                    size : (0.5, 0.5),
                    color : (
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        1.0
                    )
                }),
                _ => {}
            }
        }

        gl_call!(gl::ClearColor(1.0, 1.0, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT));

        renderer.begin_batch();

        for quad in &quads{
            renderer.submit_quad(quad.clone());
        }
        renderer.end_batch();
        // 프론트 버퍼와 백 버퍼 교체 - 프리징 방지
        window.swap_buffers();

        framerate.run();
    }
}

trait ToRange {
    fn to_range(&self, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32;
}

impl ToRange for f32 {
    fn to_range(&self, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
        let old_range = old_max - old_min;
        let new_range = new_max - new_min;

        (((self - old_min) * new_range) / old_range) + new_min
    }
}