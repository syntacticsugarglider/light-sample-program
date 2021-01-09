// THIS FILE IS AUTO-GENERATED, DO NOT EDIT 

#[cfg(not(any(
  feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow"
)))]
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}

#[cfg(not(any(
  feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow"
)))]
pub use _editor_shim::*;
#[cfg(not(any(feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod example;
#[cfg(feature = "example")]
pub use example::{example as program, Program};
#[cfg(not(any(feature = "example", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod tides;
#[cfg(feature = "tides")]
pub use tides::{tides as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod gradients;
#[cfg(feature = "gradients")]
pub use gradients::{gradients as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod twinkle;
#[cfg(feature = "twinkle")]
pub use twinkle::{twinkle as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "oops", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod current;
#[cfg(feature = "current")]
pub use current::{current as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "receive", feature = "nonlinear", feature = "flow")))]
mod oops;
#[cfg(feature = "oops")]
pub use oops::{oops as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "nonlinear", feature = "flow")))]
mod receive;
#[cfg(feature = "receive")]
pub use receive::{receive as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "flow")))]
mod nonlinear;
#[cfg(feature = "nonlinear")]
pub use nonlinear::{nonlinear as program, Program};
#[cfg(not(any(feature = "example", feature = "tides", feature = "gradients", feature = "twinkle", feature = "current", feature = "oops", feature = "receive", feature = "nonlinear")))]
mod flow;
#[cfg(feature = "flow")]
pub use flow::{flow as program, Program};
