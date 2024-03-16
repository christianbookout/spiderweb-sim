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
}

impl Simulator {
    pub fn new(timestep: f64, web: Spiderweb) -> Self {
        Self {
            web,
            timestep,
            sim_time: 0.0,
            gravity: Vector3::new(0.0, -0.01, 0.0),
            drag_coefficient: 0.47,
        }
    }

    pub fn step(&mut self) {
        self.sim_time += self.timestep;
        let dt = self.timestep;
        let dt2 = dt * dt;

        let mut new_positions = vec![Vector3::zeros(); self.web.particles.len()];
        let mut new_velocities = vec![Vector3::zeros(); self.web.particles.len()];
        let parts = self.web.particles.to_vec();
        for (i, particle) in self.web.particles.iter_mut().enumerate() {
            if particle.fixed {
                continue;
            }

            let mut total_force = self.gravity * particle.mass;
            total_force += particle.velocity * (-self.drag_coefficient);

            for silk_strand in &self.web.strands {

                let connected_particle = if parts[silk_strand.start] == *particle {
                    &parts[silk_strand.end]
                } else if parts[silk_strand.end] == *particle {
                    &parts[silk_strand.start]
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

            let acceleration = total_force / particle.mass;
            let new_position = 2.0 * particle.position - particle.prev_position + acceleration * dt2;
            new_positions[i] = new_position;

            new_velocities[i] = (new_position - particle.prev_position) / (2.0 * dt);
        }

        for (i, particle) in self.web.particles.iter_mut().enumerate() {
            particle.prev_position = particle.position;
            particle.position = new_positions[i];
            particle.velocity = new_velocities[i];
        }
    }

    pub fn get_web(&mut self) -> &mut Spiderweb {
        &mut self.web
    }
}