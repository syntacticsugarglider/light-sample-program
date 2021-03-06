static mut SEED: u32 = 123456789;
const M_F: f32 = core::u32::MAX as f32;
const M_2: u32 = core::u32::MAX / 2;
const A: u32 = 1103515245;
const C: u32 = 12345;

/// Generates a random u32 using a Linear Congruential Generator.
pub unsafe fn u32() -> u32 {
    SEED = (A * SEED + C) % core::u32::MAX;
    SEED
}

/// For threshold None, returns a 50/50 chance of true.
/// For any other threshold logit it gives the probability of true, i.e. 0.1 is a 10% chance.
pub unsafe fn bool(threshold: Option<f32>) -> bool {
    if let Some(threshold) = threshold {
        u32() > (M_F * (1. - threshold)) as u32
    } else {
        u32() > M_2
    }
}

/// Returns a random number between 0 and 1
pub unsafe fn logit() -> f32 {
    u32() as f32 / M_F
}

/// Returns a random u8
pub unsafe fn u8() -> u8 {
    (logit() * 255.) as u8
}

/// Returns a random color
pub unsafe fn color() -> [u8; 3] {
    [u8(), u8(), u8()]
}
