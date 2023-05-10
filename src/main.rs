use glfw::Context;
use glfw::ffi::glfwSwapInterval;

fn main() {
    // glfw 초기화
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // glfw 힌트
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core,));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
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
    // 수직 동기화(Vsync)
    unsafe{glfwSwapInterval(1)};

    // 메인 루프
    while !window.should_close() {
        // 이벤트를 받고 처리
        glfw.poll_events();

        for(_, event) in glfw::flush_messages(&events){
            println!("{:?}", event);
        }
        // 프론트 버퍼와 백 버퍼 교체 - 프리징 방지
        window.swap_buffers();
    }
}
