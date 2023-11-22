pub mod renderer;
pub mod debugging;
pub mod shader;
pub mod texture;
pub mod ecs;
pub mod shapes;
pub mod util;

use crate::shader::{ShaderPart, ShaderProgram};
use crate::debugging::*;
use crate::util::forward;

use glfw::ffi::glfwSwapInterval;
use glfw::{Context, WindowHint, CursorMode, Action};
use std::ffi::CString;
use std::os::raw::c_void;
use nalgebra_glm::{vec3, pi};
use nalgebra::{Vector3, Matrix4, clamp};

fn main() {
    // glfw 초기화
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // glfw 힌트
    glfw.window_hint(WindowHint::ContextVersion(4, 6));
    glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core,));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));
    // 윈도우 크기 설정
    let window_size = (800, 800);
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
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_cursor_pos(400.0, 400.0);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    // 수직 동기화(Vsync)
    unsafe{glfwSwapInterval(0)};

    gl_call!(gl::Enable(gl::DEBUG_OUTPUT));
    gl_call!(gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
    gl_call!(gl::DebugMessageCallback(Some(debug_message_callback), 0 as *const c_void));
    gl_call!(gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, 0 as *const u32, gl::TRUE));

    gl_call!(gl::Enable(gl::CULL_FACE));
    // Backface culling
    gl_call!(gl::CullFace(gl::BACK));
    // enable depth test (z-buffer)
    gl_call!(gl::Enable(gl::DEPTH_TEST));
    gl_call!(gl::Enable(gl::BLEND));
    gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
    gl_call!(gl::Viewport(0, 0, 800, 800));

    let mut camera_position = vec3(0.0f32, 0.0, 0.0);
    let mut camera_rotation = vec3(0.0f32, 0.0, 0.0);

    let vert = ShaderPart::from_vert_source(&CString::new(include_str!("shaders/diffuse.vert")).unwrap()).unwrap();
    let frag = ShaderPart::from_frag_source(&CString::new(include_str!("shaders/diffuse.frag")).unwrap()).unwrap();
    let mut program = ShaderProgram::from_shaders(vert, frag).unwrap();

    let cobblestone = texture::create_texture("blocks/cobblestone.png");
    gl_call!(gl::ActiveTexture(gl::TEXTURE0));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, cobblestone));

    let cube = shapes::unit_cube_array();

    let mut cube_vbo = 0;
    gl_call!(gl::CreateBuffers(1, &mut cube_vbo));
    gl_call!(gl::NamedBufferData(cube_vbo, (cube.len() * std::mem::size_of::<f32>()) as isize, cube.as_ptr() as *mut c_void, gl::STATIC_DRAW));

    let mut cube_vao = 0;
    gl_call!(gl::CreateVertexArrays(1, &mut cube_vao));

    gl_call!(gl::EnableVertexArrayAttrib(cube_vao, 0));
    gl_call!(gl::EnableVertexArrayAttrib(cube_vao, 1));

    gl_call!(gl::VertexArrayAttribFormat(cube_vao, 0, 3 as i32, gl::FLOAT, gl::FALSE, 0));
    gl_call!(gl::VertexArrayAttribFormat(cube_vao, 1, 2 as i32, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as u32));

    gl_call!(gl::VertexArrayAttribBinding(cube_vao, 0, 0));
    gl_call!(gl::VertexArrayAttribBinding(cube_vao, 1, 0));

    gl_call!(gl::VertexArrayVertexBuffer(cube_vao, 0, cube_vbo, 0, 5 * std::mem::size_of::<f32>() as i32));

    gl_call!(gl::BindVertexArray(cube_vao));

    let mut prev_cursor_pos = (0.0, 0.0);

    // 메인 루프
    while !window.should_close() {
        // 이벤트를 받고 처리
        glfw.poll_events();

        for(_, event) in glfw::flush_messages(&events){
            match event{
                glfw::WindowEvent::CursorPos(x, y) => {
                    let rel_x = x - prev_cursor_pos.0;
                    let rel_y = y - prev_cursor_pos.1;

                    camera_rotation.y += rel_x as f32 / 100.0;
                    camera_rotation.x += rel_y as f32 / 100.0;

                    camera_rotation.x = clamp(
                        camera_rotation.x,
                         -pi::<f32>() / 2.0 + 0.0001,
                          pi::<f32>() / 2.0 - 0.0001,
                    );

                    prev_cursor_pos = (x, y);
                }

                glfw::WindowEvent::Key(key, _, action, _) => {
                    if action == Action::Press || action == Action::Repeat{
                        match key {
                            glfw::Key::W => {
                                camera_position += forward(&camera_rotation).scale(0.03f32);
                            }
                            glfw::Key::S => {
                                camera_position -= forward(&camera_rotation).scale(0.03f32);
                            }
                            glfw::Key::A => {
                                camera_position -= forward(&camera_rotation)
                                    .cross(&Vector3::y())
                                    .scale(0.03f32);
                            }
                            glfw::Key::D => {
                                camera_position += forward(&camera_rotation)
                                    .cross(&Vector3::y())
                                    .scale(0.03f32);
                            }
                            glfw::Key::Q => {
                                camera_position.y += 0.03;
                            }
                            glfw::Key::Z => {
                                camera_position.y -= 0.03;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let direction = forward(&camera_rotation);

        let view_matrix = nalgebra_glm::look_at(&camera_position, &(camera_position + direction), &Vector3::y());

        let projection_matrix = nalgebra_glm::perspective(1.0, pi::<f32>() / 2.0, 0.1, 1000.0);
        
        let model_matrix = {
            let translate_matrix = Matrix4::new_translation(&vec3(5.0f32, 0.0, 0.0));
            let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);
            let scale_matrix = Matrix4::new_nonuniform_scaling(&vec3(1.0f32, 1.0f32, 1.0f32));

            translate_matrix * rotate_matrix * scale_matrix
        };
        
        program.use_program();

        program.set_uniform_matrix4fv("model", model_matrix.as_ptr());
        program.set_uniform_matrix4fv("view", view_matrix.as_ptr());
        program.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        program.set_uniform1i("tex", 0);

        gl_call!(gl::ClearColor(0.74, 0.84, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));

        // 프론트 버퍼와 백 버퍼 교체 - 프리징 방지
        window.swap_buffers();
    }
}