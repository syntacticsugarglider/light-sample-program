use core::future::Future;

use crate::util::next_tick;

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn example() -> Program {
    async move {
        let mut leds = crate::leds();

        loop {
            leds.clear();
            leds.fill([255, 255, 255]);

            next_tick().await;
        }
    }
}
