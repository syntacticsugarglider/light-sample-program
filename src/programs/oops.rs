use core::future::Future;

use crate::{projection::LinearSpatialExt, util::next_tick};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn oops() -> Program {
    async move {
        let mut leds = crate::leds();
        let mut frame_idx = 0;

        loop {
            leds.clear();
            frame_idx += 1;
            leds.fill_from((0..leds.len()).map(|idx| {
                [
                    100,
                    (255 - idx as u8 * 2).wrapping_add(frame_idx),
                    (idx as u8 * 2).wrapping_add(frame_idx),
                ]
            }));
            next_tick().await;
        }
    }
}
