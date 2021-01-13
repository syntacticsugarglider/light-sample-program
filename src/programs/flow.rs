use core::{future::Future, iter::repeat};

use crate::{
    projection::LinearSpatialExt,
    util::{gradient, interpolate::Linear, next_tick},
};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn flow() -> Program {
    async move {
        let mut leds = crate::leds();
        leds.fill_from(
            repeat(gradient![
                [255, 0, 0] => [0, 0, 255], Linear, 37;
                [0, 0, 255] => [255, 0, 0], Linear, 38;
            ])
            .flatten(),
        );
        loop {
            leds.rotate_left(1);
            next_tick().await;
        }
    }
}
