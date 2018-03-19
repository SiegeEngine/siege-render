
const MU: f32 = 0.0; // offset from zero
const SIGMA: f32 = 1.6; // larger is more spread out
const IMMEDIATE_FALLOFF: f32 = 0.5; // nice for stars

use std::f32::consts::PI;

fn gaussian(x: f32) -> f32
{
    let ss: f32 = SIGMA.powi(2);
    let underradical: f32 = 2.0 * PI * ss;
    let exponent: f32 = (x - MU).powi(2) / (2.0 * ss);

    (-exponent).exp() / underradical.sqrt()
}

fn main() {
    for i in -5 .. 6 {
        println!("{}: {}", i, IMMEDIATE_FALLOFF * gaussian(i as f32) / gaussian(0.0));
    }
}
