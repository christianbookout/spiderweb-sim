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

    pub fn push_particle(&mut self, particle : Particle) {
        self.particles.push(particle);
    }

    pub fn push_strand(&mut self, strand : SilkStrand) {
        self.strands.push(strand);
    }
}