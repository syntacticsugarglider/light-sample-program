use core::future::Future;

use crate::{rand, util::gradient, LedExt};

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn example() -> Program {
    async move {
        let mut leds = crate::leds();
        let gradient = {
            let len = leds.len();
            move || gradient(*rand::color().normalize(), *rand::color().normalize(), len)
        };
        leds.fill_from(gradient());
        loop {
            leds.fade_to(gradient(), 100).await;
        }
    }
}
