use nalgebra::Vector3;

use crate::web::{Particle, ParticleType, SilkStrand, Spiderweb};

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
    pub wind_strength: f64,
    pub max_silk_strand_force: f64,
}

impl Simulator {
    pub fn new(timestep: f64, web: Spiderweb) -> Self {
        Self {
            web,
            timestep,
            sim_time: 0.0,
            gravity: Vector3::new(0.0, -1.0, 0.0),
            drag_coefficient: 0.0,
            wind_fn: Self::default_wind_fn,
            bugs: Vec::new(),
            wind_strength: 0.05,
            max_silk_strand_force: 10.0,
        }
    }

    /// Simple wind to blow the web around a bit depending on position and sins
    fn default_wind_fn(&self, particle_pos: Vector3<f64>) -> Vector3<f64> {
        let wind_dir = Vector3::new(0.8 *(self.sim_time).sin(), 0.05 * (self.sim_time * 0.1).sin(), 0.1 * (self.sim_time * 0.3).sin());
        wind_dir * (particle_pos.y * self.wind_strength)
        
    }

    /// Wind that blows in a loop, blowing greater when closer to the z-axis
    fn loopy_wind(&self, particle_pos: Vector3<f64>) -> Vector3<f64> {
        let z_pos = particle_pos.z.max(0.1);
        let wind_dir = Vector3::new(particle_pos.y/z_pos, -particle_pos.x/z_pos, particle_pos.z/4.0);
        wind_dir * self.wind_strength
    }

    pub fn add_bug(&mut self, position: Vector3<f64>, velocity: Vector3<f64>, mass: f64) {
        let bug = Particle::new(position, velocity, mass, false, ParticleType::Bug);
        self.bugs.push(bug);
    }

    fn calculate_verlet(&self, particle: &Particle, total_force: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
        let acceleration = total_force / particle.mass;
        let new_position = 2.0 * particle.position - particle.prev_position + acceleration * self.timestep * self.timestep;
        let new_velocity = (new_position - particle.prev_position) / (2.0 * self.timestep);
        (new_position, new_velocity)
    }

    fn update_particle(&self, particle: &Particle, strands_to_remove: &mut Vec<(usize, usize)>) -> (Vector3<f64>, Vector3<f64>) {
        if particle.fixed {
            return (particle.position, particle.velocity);
        }

        // Gravity
        let mut total_force = self.gravity * particle.mass;
        // Drag
        total_force += particle.velocity * (-self.drag_coefficient);

        for (i, silk_strand) in self.web.strands.iter().enumerate() {
            let connected_particle_idx = if self.web.particles[silk_strand.start] == *particle {
                silk_strand.end
            } else if self.web.particles[silk_strand.end] == *particle {
                silk_strand.start
            } else {
                continue;
            };
            let connected_particle = &self.web.particles[connected_particle_idx];

            let force = calculate_spring_force(
                particle,
                connected_particle,
                silk_strand,
            );

            if force.norm() > self.max_silk_strand_force {
                strands_to_remove.push((i, connected_particle_idx));
            }
            
            total_force += force;
        }

        let wind_force = (self.wind_fn)(&self, particle.position);
        total_force += wind_force;

        let drag_force = particle.velocity * -self.drag_coefficient;
        total_force += drag_force;

        self.calculate_verlet(particle, total_force)
    }

    // Stick a bug to a web by replacing a strand of the web with a strand connecting
    // from one particle to the bug, and from the bug to the other particle.
    fn stick_to_web(&mut self, bug_index: usize, strand_index: usize) {
        let mut bug = self.bugs[bug_index];
        bug.velocity = Vector3::zeros();
        self.web.insert_particle_into_web(bug, strand_index, true);
    }

    fn detect_collisions(&mut self) {
        let bug_radius = 0.03;
        let mut to_stick = Vec::new();

        for (bug_index, bug) in self.bugs.iter().enumerate() {
            for (strand_index, strand) in self.web.strands.iter().enumerate() {
                let start_particle = &self.web.particles[strand.start];
                let end_particle = &self.web.particles[strand.end];

                let max_distance = strand.length + bug_radius;
                let distance_to_start = (bug.position - start_particle.position).norm();
                let distance_to_end = (bug.position - end_particle.position).norm();

                // If the bug is too far from the silk strand then a collision cannot possible occur
                // and we don't need to do the math
                if distance_to_start > max_distance && distance_to_end > max_distance {
                    continue;
                }

                let strand_vector = end_particle.position - start_particle.position;
                let bug_to_start = bug.position - start_particle.position;
                let t = bug_to_start.dot(&strand_vector) / strand_vector.norm_squared();
                let t_clamped = t.clamp(0.0, 1.0);
                let closest_point = start_particle.position + strand_vector * t_clamped;

                let distance = (closest_point - bug.position).norm();
                // A collision occurred
                if distance <= bug_radius {
                    to_stick.push((bug_index, strand_index));
                    continue;
                }
            }
        }

        // Stick bugs to web for each detected collision
        for (bug_index, strand_index) in to_stick.iter() {
            self.stick_to_web(*bug_index, *strand_index);
        }
        for (bug_index, _) in to_stick.iter().rev() {
            self.bugs.remove(*bug_index);
        }
    }

    pub fn step(&mut self) {
        self.sim_time += self.timestep;
        self.detect_collisions();

        let mut new_positions = vec![Vector3::zeros(); self.web.particles.len()];
        let mut new_velocities = vec![Vector3::zeros(); self.web.particles.len()];
        let mut new_bug_positions = vec![Vector3::zeros(); self.bugs.len()];
        let mut new_bug_velocities = vec![Vector3::zeros(); self.bugs.len()];
        let mut strands_to_remove = Vec::new();

        for (i, particle) in self.web.particles.iter().enumerate() {
            let (new_position, new_velocity) = self.update_particle(particle, &mut strands_to_remove);
            new_positions[i] = new_position;
            new_velocities[i] = new_velocity;
        }

        for (i, bug) in self.bugs.iter().enumerate() {
            new_bug_positions[i] = bug.position + bug.velocity * self.timestep;
            new_bug_velocities[i] = bug.velocity;
        }

        for (i, particle) in self.web.particles.iter_mut().enumerate() {
            if particle.fixed {
                continue;
            }
            particle.prev_position = particle.position;
            particle.position = new_positions[i];
            particle.velocity = new_velocities[i];
        }

        for (i, bug) in self.bugs.iter_mut().enumerate() {
            bug.prev_position = bug.position;
            bug.position = new_bug_positions[i];
            bug.velocity = new_bug_velocities[i];
        }

        // TODO: Properly handle strand breakages
        // for (strand_index, conn_particle) in strands_to_remove {
        //     let new_particle = self.web.particles[conn_particle].clone();
        //     self.web.push_particle(new_particle);
        //     let mut new_strand = self.web.strands[strand_index].clone();
        //     if new_strand.start == conn_particle {
        //         new_strand.start = self.web.particles.len() - 1;
        //     } else {
        //         new_strand.end = self.web.particles.len() - 1;
        //     }
        //     self.web.strands[strand_index] = new_strand;
        // }
    }

    pub fn get_web(&mut self) -> &mut Spiderweb {
        &mut self.web
    }
}