use core::{future::Future, iter::repeat};

use crate::{projection::LinearSpatialExt, util::next_tick};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn example() -> Program {
    async move {
        let mut leds = crate::leds();
        leds.fill_from(repeat([255, 255, 255]));

        loop {
            next_tick().await;
        }
    }
}
