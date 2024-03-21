extern crate glfw;
use renderer::Renderer;
use simulator::Simulator;
use glfw::{Action, Context, GlfwReceiver, Key, PWindow};

pub mod renderer;
pub mod simulator;
pub mod web;
pub mod webgen;


pub fn open_window(glfw: &mut glfw::Glfw) -> (PWindow, GlfwReceiver<(f64, glfw::WindowEvent)>) {
    // Set the OpenGL version to 3.3
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(800, 600, "Spiderweb Simulator", glfw::WindowMode::Windowed)
        .expect("Couldn't make the window");
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }
    (window, events)

}

fn main() {
    let mut glfw = glfw::init_no_callbacks().unwrap();
    let (mut window, events) = open_window(&mut glfw);
    let mut renderer = Renderer::new();
    let web = webgen::construct_web();
    let mut simulator = Simulator::new(0.01, web);

    renderer.init();

    unsafe {
        gl::UseProgram(renderer.shader_program);
    }
    // window.set_scroll_callback((move |_, _, y| {
    //     renderer.zoom *= 1.0 + y * 1.0;
    //     println!("Scrolling, new scroll is {}", renderer.zoom);
    // }));

    let mut started = false;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    started = !started;
                },
                glfw::WindowEvent::Scroll(_, y) => {
                    println!("Scrolling");
                    renderer.zoom += y * 0.01;
                },
                _ => {}
            }
        }
        if !started {
            continue;
        }

        simulator.step();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            renderer.draw(&mut simulator, &window);
        }
        window.set_title(&format!("Spiderweb Simulator - Simulation time: {:.2}", simulator.sim_time));

        window.swap_buffers();
    }
}