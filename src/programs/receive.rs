use core::{future::Future, iter::repeat};

use crate::Receiver;
use futures::StreamExt;

pub type Program = impl Future<Output = ()>;

#[allow(dead_code)]
pub unsafe fn example() -> Program {
    let mut receiver = Receiver!(3).exact();

    async move {
        let mut leds = crate::leds();

        while let Some(color) = receiver.next().await {
            leds.fill_from(repeat(color));
        }
    }
}
