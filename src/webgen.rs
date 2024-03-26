use nalgebra::Vector3;

use crate::web::{Particle, SilkStrand, Spiderweb, ParticleType};

pub fn construct_web() -> Spiderweb {
    let mut web = Spiderweb::new();

    let num_rings = 5; 
    let particles_per_ring = 5;
    let center = Vector3::new(0.0, 0.0, 0.0);
    let ring_spacing = 0.15;
    let stiffness = 1.0;
    let damping = 0.2;
    let mass = 1.0;

    for ring in 0..num_rings {
        let radius = ring_spacing * (ring as f64 + 1.0);
        for i in 0..particles_per_ring {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (particles_per_ring as f64);
            let position = Vector3::new(center.x + radius * angle.cos(), center.y + radius * angle.sin(), 0.0);
            let particle = Particle::new(position, Vector3::zeros(), mass, ring == particles_per_ring-1, ParticleType::Silk);
            web.push_particle(particle);
        }
    }

    for ring in 0..num_rings {
        for i in 0..particles_per_ring {
            let start = ring * particles_per_ring + i;
            let end = ring * particles_per_ring + (i + 1) % particles_per_ring;
            let length = (web.particles[start].position - web.particles[end].position).norm();
            let strand = SilkStrand::new(start, end, length, stiffness, damping);
            web.push_strand(strand);
        }
    }

    for i in 0..particles_per_ring {
        for ring in 0..(num_rings - 1) {
            let start = ring * particles_per_ring + i;
            let end = (ring + 1) * particles_per_ring + i;
            let length = (web.particles[start].position - web.particles[end].position).norm();
            let strand = SilkStrand::new(start, end, length, stiffness, damping);
            web.push_strand(strand);
        }
    }
    web

}