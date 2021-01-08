use core::future::Future;

use crate::Receiver;
use futures::StreamExt;

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn receive() -> Program {
    let mut receiver = Receiver!(3).exact();

    async move {
        let mut leds = crate::leds();

        while let Some(color) = receiver.next().await {
            leds.fill(color);
        }
    }
}
