// THIS FILE IS AUTO-GENERATED, DO NOT EDIT 

#[cfg(not(any(
  feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides"
)))]
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}

#[cfg(not(any(
  feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides"
)))]
pub use _editor_shim::*;
#[cfg(not(any(feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod line;
#[cfg(feature = "line")]
pub use line::{line as program, Program};
#[cfg(not(any(feature = "line", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod grid;
#[cfg(feature = "grid")]
pub use grid::{grid as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod gradients;
#[cfg(feature = "gradients")]
pub use gradients::{gradients as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod example;
#[cfg(feature = "example")]
pub use example::{example as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod oops;
#[cfg(feature = "oops")]
pub use oops::{oops as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod twinkle;
#[cfg(feature = "twinkle")]
pub use twinkle::{twinkle as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "current", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod flow;
#[cfg(feature = "flow")]
pub use flow::{flow as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "receive", feature = "nonlinear", feature = "tides")))]
mod current;
#[cfg(feature = "current")]
pub use current::{current as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "nonlinear", feature = "tides")))]
mod receive;
#[cfg(feature = "receive")]
pub use receive::{receive as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "tides")))]
mod nonlinear;
#[cfg(feature = "nonlinear")]
pub use nonlinear::{nonlinear as program, Program};
#[cfg(not(any(feature = "line", feature = "grid", feature = "gradients", feature = "example", feature = "oops", feature = "twinkle", feature = "flow", feature = "current", feature = "receive", feature = "nonlinear")))]
mod tides;
#[cfg(feature = "tides")]
pub use tides::{tides as program, Program};
