use core::{future::Future, iter::repeat};

use crate::util::{gradient, interpolate::Linear, next_tick};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn tides() -> Program {
    async move {
        let mut leds = crate::leds();
        leds.fill_from(
            repeat(gradient![
                [0, 255, 0] => [0, 128, 255], Linear, 37;
                [0, 128, 255] => [0, 255, 0], Linear, 38;
            ])
            .flatten(),
        );
        loop {
            for _ in 0..leds.len() {
                leds.rotate_left(1);
                next_tick().await;
            }
            for _ in 0..leds.len() {
                leds.rotate_right(1);
                next_tick().await;
            }
        }
    }
}
