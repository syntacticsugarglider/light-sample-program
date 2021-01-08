// THIS FILE IS AUTO-GENERATED, DO NOT EDIT 

#[cfg(not(any(
  feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "receive"
)))]
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}

#[cfg(not(any(
  feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "receive"
)))]
pub use _editor_shim::*;
#[cfg(not(any(feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "receive")))]
mod gradients;
#[cfg(feature = "gradients")]
pub use gradients::{gradients as program, Program};
#[cfg(not(any(feature = "gradients", feature = "oops", feature = "twinkle", feature = "flow", feature = "receive")))]
mod example;
#[cfg(feature = "example")]
pub use example::{example as program, Program};
#[cfg(not(any(feature = "gradients", feature = "example", feature = "twinkle", feature = "flow", feature = "receive")))]
mod oops;
#[cfg(feature = "oops")]
pub use oops::{oops as program, Program};
#[cfg(not(any(feature = "gradients", feature = "example", feature = "oops", feature = "flow", feature = "receive")))]
mod twinkle;
#[cfg(feature = "twinkle")]
pub use twinkle::{twinkle as program, Program};
#[cfg(not(any(feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "receive")))]
mod flow;
#[cfg(feature = "flow")]
pub use flow::{flow as program, Program};
#[cfg(not(any(feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow")))]
mod receive;
#[cfg(feature = "receive")]
pub use receive::{receive as program, Program};
