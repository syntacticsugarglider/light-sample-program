use core::{future::Future, iter::repeat};

use crate::{
    projection::{
        Cartesian2dExt, CartesianSpatialExt, LinearSpatialExt, SpatialExt, SwitchbackGrid,
    },
    util::{
        gradient,
        interpolate::{Interpolate, Linear},
        next_tick,
    },
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn lines() -> Program {
    async move {
        let mut leds = crate::leds();

        let mut line = leds
            .clone()
            .project(SwitchbackGrid::<_, u8>::new(8).constrain_height(9).unwrap())
            .into_line((1, 1), (5, 7))
            .unwrap();

        let mut frame_idx = 0;
        let a1 = [255, 0, 0];
        let b1 = [0, 0, 255];
        let a2 = [0, 255, 0];
        let b2 = [255, 0, 0];
        let l = 8;
        let len = (l as f32) * 4.;
        let mut increment = 1;
        let mut raw_idx = 0;
        loop {
            leds.clear();
            let ratio = frame_idx as f32 / len;
            let a = Linear::interpolate(&a1, &a2, ratio);
            let b = Linear::interpolate(&b1, &b2, ratio);
            raw_idx += 1;
            frame_idx = ((frame_idx as isize) + increment) as usize;
            if raw_idx == l {
                raw_idx = 0;
            }
            if frame_idx == l * 4 || frame_idx == 0 {
                increment = -increment;
            }
            line.fill_from(
                repeat(gradient![
                    b => a, Linear, 16;
                    a => b, Linear, 16;
                ])
                .flatten()
                .skip(frame_idx),
            );
            next_tick().await;
        }
    }
}
