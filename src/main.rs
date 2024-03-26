extern crate glfw;
use nalgebra::Vector3;
use renderer::Renderer;
use simulator::Simulator;
use rand::Rng;
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
    let web = webgen::simple_web();
    let timestep = 0.05;
    let mut simulator = Simulator::new(timestep, web);

    renderer.init();

    unsafe {
        gl::UseProgram(renderer.shader_program);
    }
    
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
                glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    started = false;
                    simulator = Simulator::new(timestep, webgen::simple_web());
                    renderer.reset()
                },
                glfw::WindowEvent::Key(Key::B, _, Action::Press, _) => {
                    let mut rnd = rand::thread_rng();
                    let particles = &simulator.get_web().particles;
                    let rand_pos = Vector3::new(rnd.gen_range(-1.0..1.0), rnd.gen_range(-1.0..1.0), rnd.gen_range(-1.0..1.0));
                    let rand_web_particle = particles[rnd.gen_range(0..particles.len())];
                    let velocity = (rand_web_particle.position - rand_pos).normalize() * 0.1;
                    simulator.add_bug(rand_pos, velocity, 1.0);
                },
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    renderer.rotate(10.0);
                },
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    renderer.rotate(-10.0);
                },
                glfw::WindowEvent::Key(Key::Equal, _, Action::Press, _) => {
                    renderer.zoom += 0.3;
                    renderer.zoom = renderer.zoom.min(10.0);
                },
                glfw::WindowEvent::Key(Key::Minus, _, Action::Press, _) => {
                    renderer.zoom -= 0.3;
                    renderer.zoom = renderer.zoom.max(1.0);
                },
                _ => {}
            }
        }
        if started {
            simulator.step();
        }
        window.set_title(&format!("Spiderweb Simulator - Simulation time: {:.2}", simulator.sim_time));
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            renderer.draw(&mut simulator, &window);
        }
        window.swap_buffers();


    }
}