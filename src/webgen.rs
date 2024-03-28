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
const MASS : f64 = 0.1;
const STIFFNESS : f64 = 1.0;
const DAMPING : f64 = 0.2;
#[derive(Clone)]
struct Genes {
    num_first_radii: usize,
    phase_angle_offset: f64,
    variability_factor: (f64, f64),
    direction_biases: (f64, f64, f64, f64),
    function_type: bool,
    influence_factor: f64,
    sub_radii_bias: (f64, f64, f64, f64),
} 

impl Genes {
    pub fn new(num_first_radii: usize, phase_angle_offset: f64, variability_factor: (f64, f64), direction_biases: (f64, f64, f64, f64), function_type: bool, influence_factor: f64, sub_radii_bias : (f64, f64, f64, f64)) -> Self {
        Genes {
            num_first_radii,
            phase_angle_offset,
            variability_factor,
            direction_biases,
            function_type,
            influence_factor,
            sub_radii_bias,
        }
    }
}

pub struct Webgen {
    web : Spiderweb,
    genes : Genes,
}

impl Webgen {
    pub fn new() -> Self {
        Webgen {
            web : Spiderweb::new(),
            genes : Genes {
                num_first_radii: 5,
                phase_angle_offset: 60.0,
                variability_factor: (-5.0, 5.0),
                direction_biases: (1.0, 0.1, 0.1, 0.1),
                function_type: false,
                influence_factor: 0.5,  
                sub_radii_bias: (10.0, 10.0, 10.0, 15.0), 
            },
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

    fn new_strand(&mut self, a : usize, b : usize) {
        let len = self.get_len(a, b);
        let strand = SilkStrand::new(a, b, len, STIFFNESS, DAMPING);
        self.web.push_strand(strand);
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

    /// Initial radii and frame construction
    fn stage_1(&mut self, genes: &Genes) {
        let center = self.new_particle(Vector3::new(0.0, 0.0, 0.0));
        let base_angle = 90.0 - genes.phase_angle_offset;
        let mut cur_angle = base_angle;
        let mut rand = thread_rng();
        let spacing = 360.0 / genes.num_first_radii as f64;
        let mut prev_particle = center;
        let start_particle = center;
        for i in 0..genes.num_first_radii {
            let rand_offset = rand.gen_range(genes.variability_factor.0 .. genes.variability_factor.1);
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
            if i > 0 {
                self.new_strand(prev_particle, particle);
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
    fn stage_2(&mut self, genes: &Genes) {
        // For every radii given, place intermediate particles between the radii
        let mut cur_angle = 90.0 - genes.phase_angle_offset;
        for i in 1..genes.num_first_radii + 1 {
            let next_idx = if i == genes.num_first_radii { 1 } else { i + 1 };
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

                if cur_angle + bias > angle_to_next {
                    break;
                }
                cur_angle += bias;
                
                let ratio = (cur_angle - start_angle) / angle_between_points;
                let new_pos = Vector3::lerp(&self.web.particles[i].position, &self.web.particles[next_idx].position, ratio);
                let particle = self.new_particle(new_pos);
                let len = self.get_len(0, particle);
                let strand = SilkStrand::new(0, particle, len, STIFFNESS, DAMPING);

                let closest_strand = self.web.get_closest_strand(new_pos);
                self.web.insert_particle_into_web(self.web.particles[particle], closest_strand, true);

                self.web.push_strand(strand);
                
            }
        }
    }

    /// Construction of the first loop of the capture spiral
    fn stage_3(&mut self, genes: &Genes) {

    }

    /// 
    fn stage_4(&mut self, genes: &Genes) {

    }

    pub fn realistic_web(&mut self) -> Spiderweb {
        self.web = Spiderweb::new();
        let stages: Vec<Box<dyn FnMut(&mut Self, &Genes)>> = vec![
            Box::new(|webgen, genes| webgen.stage_1(genes)),
            Box::new(|webgen, genes| webgen.stage_2(genes)),
            Box::new(|webgen, genes| webgen.stage_3(genes)),
            Box::new(|webgen, genes| webgen.stage_4(genes)),
        ];

        for mut stage in stages {
            stage(self, &self.genes.clone());
        }
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