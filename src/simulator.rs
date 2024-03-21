use nalgebra::Vector3;

use crate::web::{Particle, SilkStrand, Spiderweb};

fn calculate_spring_force(
    particle: &Particle,
    connected_particle: &Particle,
    silk_strand: &SilkStrand,
) -> Vector3<f64> {
    let pos_diff = particle.position - connected_particle.position;
    let vel_diff = particle.velocity - connected_particle.velocity;

    let pos_diff_len = pos_diff.norm().max(1e-9);
    let spring_force = pos_diff * (silk_strand.stiffness * (silk_strand.length - pos_diff_len) / pos_diff_len);
    let damp_force = pos_diff * (-silk_strand.damping * vel_diff.dot(&pos_diff) / (pos_diff_len * pos_diff_len));

    spring_force + damp_force
}

pub struct Simulator {
    web: Spiderweb,
    timestep: f64,
    pub sim_time: f64,
    gravity: Vector3<f64>,
    drag_coefficient: f64,
    pub wind_fn: fn(&Self, Vector3<f64>) -> Vector3<f64>,
    pub bugs: Vec<Particle>,
}

impl Simulator {
    fn zero_wind_fn(&self, _particle_pos: Vector3<f64>) -> Vector3<f64> {
        Vector3::zeros()
    }
    fn default_wind_fn(&self, particle_pos: Vector3<f64>) -> Vector3<f64> {
        // Simple wind to blow the web around a bit depending on position
        let wind_strength = 0.1;
        let wind_dir = Vector3::new(0.8 *(self.sim_time).sin(), 0.05 * (self.sim_time * 0.1).sin(), 0.1 * (self.sim_time * 0.3).sin());
        wind_dir * (particle_pos.y * wind_strength)
        
    }

    pub fn new(timestep: f64, web: Spiderweb) -> Self {
        Self {
            web,
            timestep,
            sim_time: 0.0,
            gravity: Vector3::new(0.0, -0.1, 0.0),
            drag_coefficient: 0.47,
            wind_fn: Self::default_wind_fn,
            bugs: Vec::new(),
        }
    }

    pub fn add_bug(&mut self, position: Vector3<f64>, velocity: Vector3<f64>, mass: f64) {
        let bug = Particle::new(position, velocity, mass, false);
        self.bugs.push(bug);
    }

    fn update_particle(&self, particle: &Particle) -> (Vector3<f64>, Vector3<f64>) {
        if particle.fixed {
            return (particle.position, particle.velocity);
        }

        // Gravity
        let mut total_force = self.gravity * particle.mass;
        // Drag
        total_force += particle.velocity * (-self.drag_coefficient);

        for silk_strand in &self.web.strands {

            let connected_particle = if self.web.particles[silk_strand.start] == *particle {
                &self.web.particles[silk_strand.end]
            } else if self.web.particles[silk_strand.end] == *particle {
                &self.web.particles[silk_strand.start]
            } else {
                continue;
            };

            let force = calculate_spring_force(
                particle,
                connected_particle,
                silk_strand,
            );
            
            total_force += force;
        }

        let wind_force = (self.wind_fn)(&self, particle.position);
        total_force += wind_force;

        let drag_force = particle.velocity * -self.drag_coefficient;
        total_force += drag_force;

        let acceleration = total_force / particle.mass;
        let new_position = 2.0 * particle.position - particle.prev_position + acceleration * self.timestep * self.timestep;
        let new_velocity = (new_position - particle.prev_position) / (2.0 * self.timestep);
        (new_position, new_velocity)
    }

    pub fn step(&mut self) {
        self.sim_time += self.timestep;

        let mut new_positions = vec![Vector3::zeros(); self.web.particles.len()];
        let mut new_velocities = vec![Vector3::zeros(); self.web.particles.len()];
        for (i, particle) in self.web.particles.iter().enumerate() {
            let (new_position, new_velocity) = self.update_particle(particle);
            new_positions[i] = new_position;
            new_velocities[i] = new_velocity;
        }

        for (i, particle) in self.web.particles.iter_mut().enumerate() {
            if particle.fixed {
                continue;
            }
            particle.prev_position = particle.position;
            particle.position = new_positions[i];
            particle.velocity = new_velocities[i];
        }
    }

    pub fn get_web(&mut self) -> &mut Spiderweb {
        &mut self.web
    }
}