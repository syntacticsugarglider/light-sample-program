use core::{future::Future, iter::repeat};

use crate::util::{gradient, lerp, next_tick};

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
            let a = lerp(a1, a2, ratio);
            let b = lerp(b1, b2, ratio);
            leds.fill_from(
                repeat(gradient![
                    a => b, 38;
                    b => a, 38;
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
