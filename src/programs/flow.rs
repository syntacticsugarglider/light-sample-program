use core::{future::Future, iter::repeat};

use crate::{
    util::{gradient, next_tick},
    LedExt,
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn flow() -> Program {
    async move {
        let mut leds = crate::leds();
        leds.fill_from(
            repeat(gradient![
                [255, 0, 0] => [0, 0, 255], 37;
                [0, 0, 255] => [255, 0, 0], 38;
            ])
            .flatten(),
        )
        .scale(0.1);
        loop {
            leds.rotate_left(1);
            next_tick().await;
        }
    }
}
