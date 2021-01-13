use core::{future::Future, iter::repeat};

use crate::{
    projection::LinearSpatialExt,
    util::{
        colors,
        interpolate::{Interpolate, Linear, SinusoidalInOut},
    },
    LedExt, LedStrip,
};

pub type Program = impl Future<Output = ()>;

async fn fade<I: Interpolate>(leds: &mut LedStrip, start: [u8; 3]) {
    leds.fade_to::<_, I>(repeat(start), 40).await;
    leds.fade_to::<_, I>(repeat(colors::blue), 40).await;
}

#[allow(dead_code)]
pub unsafe fn nonlinear() -> Program {
    async move {
        let mut leds = crate::leds();
        leds.fill_from(repeat(colors::blue));

        loop {
            for _ in 0..3 {
                fade::<Linear>(&mut leds, colors::red).await;
            }
            for _ in 0..3 {
                fade::<SinusoidalInOut>(&mut leds, colors::green).await;
            }
        }
    }
}
