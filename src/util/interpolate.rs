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
