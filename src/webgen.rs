use nalgebra::Vector3;
use rand::{random, thread_rng, Rng};

use crate::web::{Particle, SilkStrand, Spiderweb, ParticleType};


/* List of genes:
* Gene 01 - Number of first radii
* Gene 02 - Phase angle offset of north direction
* Gene 03 - Variability factor of first radii angles
* Gene 04 - Biases for (N, E, S, W) directions
* Gene 05 - Function type
* Gene 06 - Influence factor
* Gene 07 - etc
* Gene 08 - 
* Gene 09 - 
* Gene 10 - 
* Gene 11 - 
*/
const MASS : f64 = 1.0;
const STIFFNESS : f64 = 1000.0;
const DAMPING : f64 = 10.0;
#[derive(Clone)]
struct Genes {
    num_first_radii: usize,
    phase_angle_offset: f64,
    variability_factor: (f64, f64),
    direction_biases: (f64, f64, f64, f64),
    function_type: bool,
    influence_factor: f64,
    sub_radii_bias: (f64, f64, f64, f64),
    first_radial_point_offset: f64,
    radial_point_offset: f64,
    deviation_value: f64,
} 

pub struct Webgen {
    web : Spiderweb,
    genes : Genes,
    /// The fixed radii connecting the web to the environment
    base_radii : Vec<usize>,
    /// A list of all radii in the web (excluding the base radii)
    all_radii : Vec<usize>,
    /// A list of all radial (capture) points from the center to the end
    radial_points : Vec<usize>,
}

impl Webgen {
    pub fn new() -> Self {
        Webgen {
            web : Spiderweb::new(),
            genes : Genes {
                num_first_radii: 3,
                phase_angle_offset: 60.0,
                variability_factor: (-5.0, 5.0),
                direction_biases: (0.1, 0.1, 0.1, 1.0),
                function_type: false,
                influence_factor: 0.5,  
                sub_radii_bias: (30.0, 30.0, 30.0, 15.0), 
                first_radial_point_offset: 0.04,
                radial_point_offset: 0.002,
                deviation_value: 0.02,
            },
            base_radii : Vec::new(),
            all_radii : Vec::new(),
            radial_points : Vec::new(),
        }
    }
    fn new_particle(&mut self, pos : Vector3<f64>) -> usize {
        let new_particle = Particle::new(pos, Vector3::zeros(), MASS, false, ParticleType::Silk);
        self.web.push_particle(new_particle);
        self.web.particles.len() - 1
    }

    fn get_len(&self, a : usize, b : usize) -> f64 {
        (self.web.particles[a].position - self.web.particles[b].position).norm()
    }

    fn new_strand(&mut self, a : usize, b : usize) -> usize {
        let len = self.get_len(a, b);
        let strand = SilkStrand::new(a, b, len, STIFFNESS, DAMPING);
        self.web.push_strand(strand);
        self.web.strands.len() - 1
    }

    fn interpolate_bias(&self, angle: f64) -> f64 {
        let normalized_angle = angle % 360.0;
        let quadrant_size = 90.0;
        let quadrant = (normalized_angle / quadrant_size).floor() as usize;
        let angle_within_quadrant = normalized_angle % quadrant_size;
        
        let biases = &self.genes.direction_biases;
        let (bias_start, bias_end) = match quadrant {
            0 => (biases.0, biases.1),
            1 => (biases.1, biases.2),
            2 => (biases.2, biases.3),
            3 => (biases.3, biases.0),
            _ => unreachable!(),
        };

        let interpolation = angle_within_quadrant / quadrant_size;
        bias_start + (bias_end - bias_start) * interpolation
    }

    fn new_base_strand(&mut self, b: usize) {
        let further_particle = self.new_particle(self.web.particles[b].position * 2.0);
        self.web.particles[further_particle].fixed = true;
        self.new_strand(b, further_particle);
    }

    /// Initial radii and frame construction
    fn stage_1(&mut self) {
        let center = self.new_particle(Vector3::new(0.0, 0.0, 0.0));
        let base_angle = 90.0 - self.genes.phase_angle_offset;
        let mut cur_angle = base_angle;
        let mut rand = thread_rng();
        let spacing = 360.0 / self.genes.num_first_radii as f64;
        let mut prev_particle = center;
        let mut start_particle = center;
        for i in 0..self.genes.num_first_radii {
            let rand_offset = rand.gen_range(self.genes.variability_factor.0 .. self.genes.variability_factor.1);
            cur_angle += rand_offset + spacing;
            let bias = self.interpolate_bias(cur_angle);
            let base_radius = 1.0;
            let adjusted_radius = base_radius * (1.0 + bias);
            
            let x = adjusted_radius * cur_angle.to_radians().cos();
            let y = adjusted_radius * cur_angle.to_radians().sin();
            let pos = Vector3::new(x, y, 0.0);
            let particle = self.new_particle(pos);
            let len = self.get_len(center, particle);
            let strand = SilkStrand::new(center, particle, len, STIFFNESS, DAMPING);
            self.web.push_strand(strand);
            self.base_radii.push(particle);
            if i > 0 {
                self.new_strand(prev_particle, particle);
            } else {
                // When i is at 0, we need to connect the last particle to the first
                start_particle = particle;
            }
            prev_particle = particle;
        }
        self.new_strand(prev_particle, start_particle);
    }

    fn angle_btwn_points(&self, x : usize, y: usize) -> f64 {
        let center = self.web.particles[0].position;
        let x_pos = self.web.particles[x].position - center;
        let y_pos = self.web.particles[y].position - center;
        let angle = y_pos.angle(&x_pos);
        angle.to_degrees()
    }

    /// Additional radii are filled into the space between the initial radii
    fn stage_2(&mut self) {
        // For every radii given, place intermediate particles between the radii
        let mut cur_angle = 90.0 - self.genes.phase_angle_offset;
        for i in 1..self.genes.num_first_radii + 1 {
            self.all_radii.push(i);
            
            let next_idx = if i == self.genes.num_first_radii { 1 } else { i + 1 };
            let angle_between_points = self.angle_btwn_points(i, next_idx);
            let angle_to_next = cur_angle + angle_between_points;
            let start_angle = cur_angle;
            // Ensure there aren't too many particles (number is arbitrary)
            for _ in 0..1000 {
                let normalized_angle = cur_angle % 360.0;
                let quadrant_size = 90.0;
                let quadrant = (normalized_angle / quadrant_size).floor() as usize;
                
                let biases = &self.genes.sub_radii_bias;
                let bias = match quadrant {
                    0 => biases.0,
                    1 => biases.1,
                    2 => biases.2,
                    3 => biases.3,
                    _ => unreachable!(),
                };

                if cur_angle + bias + 5.0 > angle_to_next {
                    break;
                }
                cur_angle += bias;
                
                let ratio = (cur_angle - start_angle) / angle_between_points;
                let new_pos = Vector3::lerp(&self.web.particles[i].position, &self.web.particles[next_idx].position, ratio);
                let particle = self.new_particle(new_pos);
                self.all_radii.push(particle);
                let len = self.get_len(0, particle);
                let strand = SilkStrand::new(0, particle, len, STIFFNESS, DAMPING);

                let closest_strand = self.web.get_closest_strand(new_pos);
                self.web.insert_particle_into_web(self.web.particles[particle], closest_strand, true);

                self.web.push_strand(strand);
            }
        }
        // Connect all of the base radii to fixed points (anchors)
        for &i in &self.base_radii.clone() {
            self.new_base_strand(i);
        }
    }

    /// Construction of the first loop of the capture spiral
    fn stage_3(&mut self) {
        let mut radii_magnitude = self.genes.first_radial_point_offset;
        let start_radii = [self.all_radii[0]];
        for (indx, &i) in self.all_radii.clone().iter().chain(start_radii.iter()).enumerate() {
            let point = self.web.particles[i].position.normalize() * radii_magnitude;
            let particle = self.new_particle(point);

            let closest_strand = self.web.get_closest_strand(point);
            self.web.insert_particle_into_web(self.web.particles[particle], closest_strand, false);
            if indx > 0 {
                self.new_strand(particle, particle-1);
            }
            self.radial_points.push(particle);
            radii_magnitude += self.genes.radial_point_offset;
        }
        // Radial points will be 1 greater than all_radii since we need a full looping connection to be able to
        // calculate last_dist in stage_4
    }

    /// 
    fn stage_4(&mut self) {
        let first_part = self.web.particles[self.radial_points[0]];
        let last_part = self.web.particles[self.radial_points[self.radial_points.len() - 1]];
        let mut last_dist = (first_part.position - last_part.position).norm();
        let mut has_flipped = false;
        let mut just_flipped = false;
        let base_size = self.all_radii.len() as i32;
        // This is set to (cur point idx - 1) when we flip, then decremented by 1 for every new point
        let mut last_dist_particle_indx: i32 = 0;
        let mut sign = 1;
        loop {
            if has_flipped {
                last_dist_particle_indx += 1 * sign;
            }
            if last_dist_particle_indx < 0 {
                break;
            }
            let i = self.radial_points[last_dist_particle_indx as usize];
            let last_particle_pos = self.web.particles[i].position;
            let dir = last_particle_pos.normalize();
            let new_dir = last_dist * dir;
            let new_dist = new_dir.norm() + thread_rng().gen_range(-self.genes.deviation_value..self.genes.deviation_value);
            let new_pos = last_particle_pos + new_dir;

            let perimeter_particle = self.all_radii[i % base_size as usize];
            let radii_pos = self.web.particles[perimeter_particle].position;
            if new_pos.norm() > radii_pos.norm() {
                println!("Flipped because new_pos: {:?} at i: {} is greater than radii_pos: {:?} at i: {}", new_pos, i, radii_pos, i % base_size as usize);
                if just_flipped {
                    break;
                }
                sign *= -1;
                has_flipped = true;
                just_flipped = true;
                last_dist_particle_indx -= 1;
                continue;
            }
            just_flipped = false;

            let particle = self.new_particle(new_pos);
            self.radial_points.push(particle);

            let closest_strand = self.web.get_closest_strand(new_pos);
            self.web.insert_particle_into_web(self.web.particles[particle], closest_strand, false);
            self.new_strand(particle, particle - 1);
            
            last_dist = new_dist;
            if !has_flipped {
                last_dist_particle_indx += 1;
            }
        }
    }

    pub fn realistic_web(&mut self) -> Spiderweb {
        self.web = Spiderweb::new();
        self.stage_1();
        self.stage_2();
        self.stage_3();
        self.stage_4();
        self.web.clone()
    }

    pub fn simple_web(&mut self) -> Spiderweb {
        self.web = Spiderweb::new();

        let num_rings = 5; 
        let particles_per_ring = 5;
        let center = Vector3::new(0.0, 0.0, 0.0);
        let ring_spacing = 0.15;
        let stiffness = 1.0;
        let damping = 0.2;
        let mass = 0.1;

        for ring in 0..num_rings {
            let radius = ring_spacing * (ring as f64 + 1.0);
            for i in 0..particles_per_ring {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (particles_per_ring as f64);
                let position = Vector3::new(center.x + radius * angle.cos(), center.y + radius * angle.sin(), 0.0);
                let particle = Particle::new(position, Vector3::zeros(), mass, ring == particles_per_ring-1, ParticleType::Silk);
                self.web.push_particle(particle);
            }
        }

        for ring in 0..num_rings {
            for i in 0..particles_per_ring {
                let start = ring * particles_per_ring + i;
                let end = ring * particles_per_ring + (i + 1) % particles_per_ring;
                let length = (self.web.particles[start].position - self.web.particles[end].position).norm();
                let strand = SilkStrand::new(start, end, length, stiffness, damping);
                self.web.push_strand(strand);
            }
        }

        for i in 0..particles_per_ring {
            for ring in 0..(num_rings - 1) {
                let start = ring * particles_per_ring + i;
                let end = (ring + 1) * particles_per_ring + i;
                let length = (self.web.particles[start].position - self.web.particles[end].position).norm();
                let strand = SilkStrand::new(start, end, length, stiffness, damping);
                self.web.push_strand(strand);
            }
        }
        self.web.clone()

    }
}