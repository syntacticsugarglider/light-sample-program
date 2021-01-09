use core::future::Future;

use crate::Receiver;
use futures::StreamExt;

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn receive() -> Program {
    let mut receiver = Receiver!(type ([u8; 3], u8));

    async move {
        let mut leds = crate::leds();

        while let Some((color, idx)) = receiver.next().await {
            if let Some(led) = leds.get_mut(idx as usize) {
                *led = color;
            }
        }
    }
}
