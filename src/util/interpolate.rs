pub trait Interpolate {
    fn interpolate(a: &[u8; 3], b: &[u8; 3], factor: f32) -> [u8; 3];
}

#[derive(Clone, Copy)]
pub struct Linear;

impl Interpolate for Linear {
    fn interpolate(a: &[u8; 3], b: &[u8; 3], factor: f32) -> [u8; 3] {
        let delta = [
            b[0] as f32 - a[0] as f32,
            b[1] as f32 - a[1] as f32,
            b[2] as f32 - a[2] as f32,
        ];
        [
            (a[0] as f32 + (delta[0] * factor)) as u8,
            (a[1] as f32 + (delta[1] * factor)) as u8,
            (a[2] as f32 + (delta[2] * factor)) as u8,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct SinusoidalInOut;

impl Interpolate for SinusoidalInOut {
    fn interpolate(a: &[u8; 3], b: &[u8; 3], factor: f32) -> [u8; 3] {
        let factor = 0.5 * (1.0 - nikisas::cos(factor * core::f32::consts::PI));
        Linear::interpolate(a, b, factor)
    }
}

#[derive(Clone, Copy)]
pub struct SinusoidalOut;

impl Interpolate for SinusoidalOut {
    fn interpolate(a: &[u8; 3], b: &[u8; 3], factor: f32) -> [u8; 3] {
        let factor = nikisas::sin(factor * core::f32::consts::FRAC_PI_2);
        Linear::interpolate(a, b, factor)
    }
}

#[derive(Clone, Copy)]
pub struct SinusoidalIn;

impl Interpolate for SinusoidalIn {
    fn interpolate(a: &[u8; 3], b: &[u8; 3], factor: f32) -> [u8; 3] {
        let factor = nikisas::sin((factor - 1.0) * core::f32::consts::FRAC_PI_2) + 1.0;
        Linear::interpolate(a, b, factor)
    }
}
