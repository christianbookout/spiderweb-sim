extern crate gl;
extern crate glfw;

use std::ffi::CString;
use crate::simulator::Simulator;
use crate::web::{Particle, ParticleType, Spiderweb};
use std::fs::File;
use std::io::prelude::*;
use gl::types::*;
use nalgebra as na;

fn load_shader_source(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn compile_shader(src: &str, kind: GLenum) -> Result<GLuint, String> {
    let shader;
    unsafe {
        shader = gl::CreateShader(kind);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            return Err(String::from_utf8_lossy(&buf).into_owned());
        }
    }
    Ok(shader)
}

fn create_shader_program(vertex_path: &str, fragment_path: &str) -> Result<GLuint, String> {
    let vertex_shader_source = load_shader_source(vertex_path)
        .map_err(|e| e.to_string())?;
    let fragment_shader_source = load_shader_source(fragment_path)
        .map_err(|e| e.to_string())?;

    let vertex_shader = compile_shader(&vertex_shader_source, gl::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(&fragment_shader_source, gl::FRAGMENT_SHADER)?;

    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            return Err(String::from_utf8_lossy(&buf).into_owned());
        }

        gl::DetachShader(program, vertex_shader);
        gl::DetachShader(program, fragment_shader);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    Ok(program)
}

pub struct Renderer {
    pub shader_program: GLuint,
    pub zoom: f64,
    pub rotation: f64,
    pub camera_pos: na::Point3<f32>
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            shader_program: 0,
            zoom: 3.0,
            rotation: 135.0,
            camera_pos: na::Point3::new(0.0, 5.0, 8.0)
        }
    }

    pub fn init(&mut self) {
        let vertex_shader_path = "./src/shaders/vertex.glsl";
        let fragment_shader_path = "./src/shaders/fragment.glsl";
        match create_shader_program(vertex_shader_path, fragment_shader_path) {
            Ok(program) => {
                self.shader_program = program;
                println!("Shader program created successfully");
                let mut range = [0.0, 0.0];
                unsafe {
                    gl::GetFloatv(gl::LINE_WIDTH_RANGE, range.as_mut_ptr());
                }
                println!("Supported line width range: {} to {}", range[0], range[1]);
            },
            Err(e) => {
                eprintln!("Shader program creation failed: {}", e);
            }
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
        }
        self.init();
    }

    pub fn rotate(&mut self, angle: f64) {
        self.rotation += angle;
    }

    unsafe fn set_uniform_color(&self, color: [GLfloat; 4]) {
        let color_str = CString::new("inputColor").unwrap();
        let color_pos = gl::GetUniformLocation(self.shader_program, color_str.as_ptr());
        gl::Uniform4f(color_pos, color[0], color[1], color[2], color[3]);
    }

    unsafe fn draw_line(&self, vertices : &[GLfloat; 6], width: f32) {
        gl::LineWidth(width);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * std::mem::size_of::<GLfloat>()) as gl::types::GLsizeiptr,
                       vertices.as_ptr() as *const gl::types::GLvoid,
                       gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
        gl::DrawArrays(gl::LINES, 0, 2);
    }

    fn draw_xyz_lines(&self) {
        let xyz_vertices = [
            // x-axis (red)
             [0.0, 0.0, 0.0,
              1.0, 0.0, 0.0,],
            // y-axis (green)
             [0.0, 0.0, 0.0,
             0.0,  1.0, 0.0,],
            // z-axis (blue)
             [0.0, 0.0, 0.0,
             0.0, 0.0,  1.0,],
        ];

        let colours = [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ];
        
        unsafe {
            for (i, vertices) in xyz_vertices.iter().enumerate() {
                self.set_uniform_color(colours[i]);
                self.draw_line(vertices, 1.0);
            }
        }
    }

    unsafe fn draw_bug(&self, bug: &Particle) {
        let pos = bug.position;
        let gl_pos = [
            pos.x as GLfloat, pos.y as GLfloat, pos.z as GLfloat,
        ];
        self.set_uniform_color([0.0, 1.0, 0.0, 1.0]);
        
        gl::PointSize(10.0);
        gl::BufferData(gl::ARRAY_BUFFER,
                        (gl_pos.len() * std::mem::size_of::<GLfloat>()) as gl::types::GLsizeiptr,
                        gl_pos.as_ptr() as *const gl::types::GLvoid,
                        gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
        gl::DrawArrays(gl::POINTS, 0, 1);
    }

    unsafe fn draw_web(&self, web: &Spiderweb) {
        for strand in &web.strands {
            let pos = web.particles[strand.start].position;
            let end_pos = web.particles[strand.end].position;
            let vertices = [
                pos.x as GLfloat, pos.y as GLfloat, pos.z as GLfloat,
                end_pos.x as GLfloat, end_pos.y as GLfloat, end_pos.z as GLfloat,
            ];

            self.set_uniform_color([1.0, 1.0, 1.0, 1.0]);
            self.draw_line(&vertices, 10.0);
        }
        for particle in &web.particles {
            if particle.particle_type != ParticleType::Bug {
                continue;
            }
            self.draw_bug(particle)
        }
    }
    // Draw bugs as little green points
    unsafe fn draw_bugs(&self, bugs: &Vec<Particle>) {
        for bug in bugs {
            self.draw_bug(bug)
        }
    }

    pub unsafe fn draw(&mut self, sim: &mut Simulator, window : &glfw::Window) {
        
        let web = sim.get_web();
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);

        gl::Clear(gl::COLOR_BUFFER_BIT);

        let aspect_ratio = window.get_size().0 as f32 / window.get_size().1 as f32;
        let fov: f32 = 45.0f32.to_radians() / self.zoom as f32;
        let near_plane = 0.1;
        let far_plane = 100.0;

        let projection_matrix = na::Perspective3::new(aspect_ratio, fov, near_plane, far_plane).to_homogeneous();
        let dist_from_center = 8.0; // Radius of the circle around the Y-axis
        let rotation_radians = self.rotation.to_radians(); // Convert rotation angle to radians

        // Calculate new camera position for circular rotation around Y-axis
        let x = dist_from_center * rotation_radians.cos() as f32;
        let z = dist_from_center * rotation_radians.sin() as f32;
        self.camera_pos.x = x;
        self.camera_pos.z = z;
        let view_matrix = na::Isometry3::look_at_rh(
            &self.camera_pos,
            &na::Point3::new(0.0, 0.0, 0.0),
            &na::Vector3::new(0.0, 1.0, 0.0),
        ).to_homogeneous();
        let model_matrix: na::Matrix4<f32> = na::Matrix4::identity();

        let mvp = projection_matrix * view_matrix * model_matrix;

        let mvp_str = CString::new("MVP").unwrap();

        let mvp_pos = gl::GetUniformLocation(self.shader_program, mvp_str.as_ptr());

        gl::UniformMatrix4fv(mvp_pos, 1, gl::FALSE, mvp.as_ptr());

        self.draw_xyz_lines();
        self.draw_web(web);
        self.draw_bugs(&sim.bugs);

        gl::DisableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
}
