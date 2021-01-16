use core::{future::Future, iter::repeat};

use crate::{
    projection::LinearSpatialExt,
    util::{
        gradient,
        interpolate::{Interpolate, Linear},
        next_tick,
    },
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn current() -> Program {
    async move {
        let mut leds = crate::leds();

        let mut frame_idx = 0;
        let a1 = [255, 0, 0];
        let b1 = [0, 0, 255];
        let a2 = [0, 255, 0];
        let b2 = [255, 0, 0];
        let len = (leds.len() as f32) * 10.;
        let mut increment = 1;
        let mut raw_idx = 0;
        loop {
            let ratio = frame_idx as f32 / len;
            let a = Linear::interpolate(&a1, &a2, ratio);
            let b = Linear::interpolate(&b1, &b2, ratio);
            leds.fill_from(
                repeat(gradient![
                    a => b, Linear, leds.len() / 2;
                    b => a, Linear, leds.len() / 2;
                ])
                .flatten()
                .skip(raw_idx),
            );
            raw_idx += 1;
            frame_idx = ((frame_idx as isize) + increment) as usize;
            if raw_idx == leds.len() {
                raw_idx = 0;
            }
            if frame_idx == leds.len() * 10 || frame_idx == 0 {
                increment = -increment;
            }
            next_tick().await;
        }
    }
}
