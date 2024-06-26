extern crate glfw;
use std::sync::mpsc::Receiver;

use imgui::{self, im_str};
use nalgebra::Vector3;
use renderer::Renderer;
use simulator::Simulator;
use rand::Rng;
use glfw::{Action, Context, Key, Window};
use webgen::Webgen;

pub mod renderer;
pub mod simulator;
pub mod web;
pub mod webgen;


pub fn open_window(glfw: &mut glfw::Glfw) -> (Window, Receiver<(f64, glfw::WindowEvent)>) {
    // Set the OpenGL version to 3.3
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(800, 600, "Spiderweb Simulator", glfw::WindowMode::Windowed)
        .expect("Couldn't make the window");
    window.make_current();
    window.set_all_polling(true); // 1.42


    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }
    (window, events)

}

fn add_bug(simulator: &mut Simulator) {
    let mut rnd = rand::thread_rng();
    let particles = &simulator.get_web().particles;
    let rand_pos = Vector3::new(rnd.gen_range(-1.0..1.0), rnd.gen_range(-1.0..1.0), rnd.gen_range(-1.0..1.0));
    let rand_web_particle = particles[rnd.gen_range(0..particles.len())];
    let velocity = (rand_web_particle.position - rand_pos).normalize() * 1.0;
    simulator.add_bug(rand_pos, velocity, 2.0);
}

fn fps_test(simulator: &Simulator) {
    let mut wtr = csv::Writer::from_path("fps_by_strands.csv").unwrap();
    wtr.write_record(&["Iteration", "Time", "Webgen Time", "Steps", "Strands"]).unwrap();
    let mut radial_point_offset = 0.0001;
    for i in 0..100 {
        println!("Iteration {}", i);
        let mut webgen = Webgen::new();
        webgen.genes.radial_point_offset = radial_point_offset;
        webgen.genes.deviation_value = 0.001 + radial_point_offset;
        radial_point_offset += 0.0001;
        let webgen_time = std::time::Instant::now();
        let web = webgen.realistic_web();
        let actual_webgen_time = webgen_time.elapsed().as_millis();
        let strand_count = web.strands.len();
        let mut sim = Simulator::new(simulator.timestep, web);
        let step_count = 5;
        let cur_time = std::time::Instant::now();
        for _ in 0..step_count {
            sim.step();
        }
        wtr.write_record(&[
            i.to_string(), 
            cur_time.elapsed().as_millis().to_string(), 
            actual_webgen_time.to_string(), 
            step_count.to_string(), 
            strand_count.to_string()
        ]).unwrap();
    }
    wtr.flush().unwrap();
}

fn fps_bug_test(simulator: &mut Simulator) {
    let mut wtr = csv::Writer::from_path("fps_by_bugs.csv").unwrap();
    wtr.write_record(&["Iteration", "Time", "Bug Count", "Steps", "Strands"]).unwrap();
    for i in 0..50 {
        println!("Iteration {}", i);
        let bugs_per_iter = 25;
        let total_bugs = i * bugs_per_iter;
        let web = simulator.get_web().clone();
        let strand_count = web.strands.len();
        let mut sim = Simulator::new(simulator.timestep, web);
        let step_count = 5;
        for _ in 0..total_bugs {
            add_bug(&mut sim);
        }
        let cur_time = std::time::Instant::now();
        for _ in 0..step_count {
            sim.step();
        }
        wtr.write_record(&[
            i.to_string(),
            cur_time.elapsed().as_millis().to_string(), 
            total_bugs.to_string(),
            step_count.to_string(), 
            strand_count.to_string()
        ]).unwrap();
    }
    wtr.flush().unwrap();
}
fn main() {
    let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = open_window(&mut glfw);

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| window.get_proc_address(s));
    let platform = imgui_glfw_support::GlfwPlatform::init(&mut imgui);

    let mut renderer = Renderer::new();
    let mut webgen = Webgen::new();
    let web = webgen.realistic_web();
    let timestep = 0.01;
    let mut simulator = Simulator::new(timestep, web);

    renderer.init();
    let (width, height) = window.get_size();
    imgui.io_mut().display_size = [width as f32, height as f32];
    imgui.io_mut().config_flags.insert(imgui::ConfigFlags::NAV_ENABLE_KEYBOARD);
    imgui.io_mut().config_flags.insert(imgui::ConfigFlags::NAV_ENABLE_SET_MOUSE_POS);
    
    unsafe {
        gl::UseProgram(renderer.shader_program);
    }

    let (window_width, window_height) = window.get_size();
    let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
    let scale_x = framebuffer_width as f32 / window_width as f32;
    let scale_y = framebuffer_height as f32 / window_height as f32;

    imgui.io_mut().display_framebuffer_scale = [scale_x, scale_y];

    let mut started = false;

    while !window.should_close() {
        glfw.poll_events();

        platform.prepare_frame(imgui.io_mut(), &mut window).expect("Failed to start frame");

        let ui = imgui.frame();


        let (window_width, window_height) = window.get_size();
        let controls_window_size = [200.0, window_height as f32 - 20.0];
        let controls_window_pos = [10.0, 10.0];
        let info_window_size = [200.0, 200.0];
        let info_window_pos = [
            (window_width as f32 - info_window_size[0]) - 10.0,
            (window_height as f32 - info_window_size[1]) - 10.0,
        ];

        imgui::Window::new(im_str!("Web Simulation Controls"))
            .size(controls_window_size, imgui::Condition::Always)
            .position(controls_window_pos, imgui::Condition::Always)
            .build(&ui, || {
                // imgui uses labels on their UI elements to identify them, thus
                // they all need to be unique. However, their label placements look
                // pretty bad (to the right of the input box), so I just put a text
                // element above them as the label, and the label as a unique amount
                // of spaces to the right of the text element (invisible space)
                ui.text(im_str!("### Simulation Controls ###\n"));
                ui.checkbox(im_str!("Simulation Running"), &mut started);
                if ui.button(im_str!("Reset"), [100.0, 20.0]) {
                    started = false;
                    simulator = Simulator::new(timestep, webgen.realistic_web());
                }
                if ui.button(im_str!("Add Bug"), [100.0, 20.0]) {
                    add_bug(&mut simulator);
                }

                ui.text(im_str!("\n## Simulation Parameters ##\n"));
                // Gravity
                let mut gravity = simulator.gravity.y as f32;
                ui.text(im_str!("Gravity"));
                ui.input_float(im_str!(" "), &mut gravity).build();
                simulator.gravity.y = gravity as f64;

                // Wind Strength
                let mut wind_strength = simulator.wind_strength as f32;
                ui.text(im_str!("Wind Strength"));
                ui.input_float(im_str!("  "), &mut wind_strength).build();
                simulator.wind_strength = wind_strength as f64;

                // Drag Coefficient
                let mut drag_coefficient = simulator.drag_coefficient as f32;
                ui.text(im_str!("Drag Coefficient"));
                ui.input_float(im_str!("   "), &mut drag_coefficient).build();
                simulator.drag_coefficient = drag_coefficient as f64;

                ui.text(im_str!("\n##### Web Generation ######\n"));

                let mut stiffness = webgen.stiffness as f32;
                ui.text(im_str!("Stiffness"));
                ui.input_float(im_str!("    "), &mut stiffness).build();
                webgen.stiffness = stiffness as f64;

                let mut damping = webgen.damping as f32;
                ui.text(im_str!("Damping"));
                ui.input_float(im_str!("     "), &mut damping).build();
                webgen.damping = damping as f64;

                let mut mass = webgen.mass as f32;
                ui.text(im_str!("Mass"));
                ui.input_float(im_str!("      "), &mut mass).build();
                webgen.mass = mass as f64;

                let mut radial_point_offset = webgen.genes.radial_point_offset as f32;
                ui.text(im_str!("Radial Spacing"));
                ui.input_float(im_str!("       "), &mut radial_point_offset).build();
                webgen.genes.radial_point_offset = radial_point_offset as f64;

                let mut num_first_radii = webgen.genes.num_first_radii as i32;
                ui.text(im_str!("First Radii Count"));
                ui.input_int(im_str!("        "), &mut num_first_radii).build();
                webgen.genes.num_first_radii = num_first_radii as usize;

                ui.text(im_str!("\n### Performance Testing ###"));
                if ui.button(im_str!("Test FPS"), [100.0, 20.0]) {
                    fps_test(&simulator);
                } 
                if ui.button(im_str!("Test Bugs"), [100.0, 20.0]) {
                    fps_bug_test(&mut simulator);
                }
            });
        imgui::Window::new(im_str!("Simulation Info"))
            .size(info_window_size, imgui::Condition::Always)
            .position(info_window_pos, imgui::Condition::Always)
            .build(&ui, || {
                ui.text(im_str!("Timestep: {}", timestep));
                ui.text(im_str!("Strands: {}", simulator.get_web().strands.len()));
                ui.text(im_str!("Bugs: {}", simulator.bugs.len()));
                ui.text(im_str!("Simulation Time: {}", simulator.sim_time));
                ui.text(im_str!("Zoom: {:.1}", renderer.zoom / 3.0));
            });
            
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            renderer.draw(&mut simulator, &window);
        }

        platform.prepare_render(&ui, &mut window);
        imgui_renderer.render(ui);
        
                
        for (_, event) in glfw::flush_messages(&events) {
            platform.handle_event(imgui.io_mut(), &window, &event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
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
        window.set_title("Spiderweb Simulator");
        window.swap_buffers();
    }
}