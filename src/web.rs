extern crate nalgebra as na;
use na::Vector3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParticleType {
    Bug,
    Silk,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Particle {
    pub position: Vector3<f64>,
    pub prev_position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub mass: f64,
    pub fixed: bool,
    pub particle_type: ParticleType,
}

impl Particle {
    pub fn new(position: Vector3<f64>, velocity: Vector3<f64>, mass: f64, fixed: bool, particle_type : ParticleType) -> Self {
        Particle {
            position,
            prev_position: position,
            velocity,
            mass,
            fixed,
            particle_type
        }
    }
}

#[derive(Copy, Clone)]
pub struct SilkStrand {
    pub start: usize,
    pub end: usize,
    pub length: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl SilkStrand {
    pub fn new(start: usize, end: usize, length: f64, stiffness: f64, damping: f64) -> Self {
        SilkStrand {
            start,
            end,
            length,
            stiffness,
            damping
        }
    }
}

#[derive(Clone)]
pub struct Spiderweb {
    pub particles: Vec<Particle>,
    pub strands: Vec<SilkStrand>,
}

impl Spiderweb {
    pub fn new() -> Self {
        Spiderweb {
            particles: Vec::new(),
            strands: Vec::new(),
        }
    }

    /// Attaches a spring from the start of the strand to the particle, and a spring
    /// from the particle to the end of the strand.
    ///
    /// * `particle`: The given particle to insert into the web
    /// * `strand_idx`: The index of the strand the particle is being inserted into
    /// * `preserve_Length`: True to preserve the original length of the spring
    ///   (ie sum of the rest lengths of two new springs == rest length of the
    ///   old spring). False to create lengths of the spring based on the
    ///   distance from the particle to the strand start and strand end,
    ///   starting the new spring in equilibrium.
    pub fn insert_particle_into_web(&mut self, particle : Particle, strand_idx : usize, preserve_length : bool) {
        self.push_particle(particle);
        let new_particle_idx = self.particles.len() - 1;
        let strand = self.strands.swap_remove(strand_idx);
        let start_particle = self.particles[strand.start];
        let end_particle = self.particles[strand.end];
        let strand_vector = end_particle.position - start_particle.position;

        let start_len;
        let end_len;
        if preserve_length {
            let start_pos_diff = (particle.position - start_particle.position).magnitude();
            let end_pos_diff =  (particle.position - end_particle.position).magnitude();
            start_len = start_pos_diff / (start_pos_diff + end_pos_diff) * strand.length;
            end_len = strand.length - start_len;
        } else {
            start_len = (particle.position - start_particle.position).magnitude();
            end_len = (particle.position - end_particle.position).magnitude();
        }

        let new_start_strand = SilkStrand::new(strand.start, new_particle_idx, start_len, strand.stiffness, strand.damping);
        let new_end_strand = SilkStrand::new(new_particle_idx, strand.end, end_len, strand.stiffness, strand.damping);

        self.strands.push(new_start_strand);
        self.strands.push(new_end_strand);
    }

    /// Finds the closest strand to the given position by finding the smallest
    /// distance to the silk strand using projection. 
    /// 
    /// Time complexity of O(# of strands)
    pub fn get_closest_strand(self, pos : Vector3<f64>) -> usize {
        let mut closest_strand_dist = f64::INFINITY;
        let mut closest_strand_idx = 0;
        for (idx, strand) in self.strands.iter().enumerate(){
            let p_start = self.particles[strand.start].position;
            let p_end = self.particles[strand.end].position;

            let v = p_end - p_start;
            let w = pos - p_start;

            let t = w.dot(&v) / v.dot(&v);
            let t_clamped = t.clamp(0.0, 1.0);

            let projection = p_start + v * t_clamped;
            let distance = (pos - projection).magnitude();
            if distance < closest_strand_dist {
                closest_strand_dist = distance;
                closest_strand_idx = idx;
            }
        }
        closest_strand_idx
    }

    pub fn push_particle(&mut self, particle : Particle) {
        self.particles.push(particle);
    }

    pub fn push_strand(&mut self, strand : SilkStrand) {
        self.strands.push(strand);
    }
}