#![no_std]
#![feature(const_generics)]
#![feature(generic_associated_types)]
#![feature(const_evaluatable_checked)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![allow(incomplete_features)]

use arrayvec::ArrayVec;
use futures::Stream;

pub trait Receiver<const LEN: usize>: Stream<Item = ([u8; LEN], usize)> {
    type ExactStream: ExactReceiver<LEN> + Unpin;
    type ExtractStream<T: Unpin>: Stream<Item = T> + Unpin
    where
        T: for<'a> ByteCopy;

    fn exact(self) -> Self::ExactStream
    where
        Self: Sized;

    fn extract<T: Unpin>(self) -> Self::ExtractStream<T>
    where
        T: for<'a> ByteCopy,
        Lift<{ T::MIN_LENGTH <= LEN }>: True;
}

pub trait ByteCopy {
    const MIN_LENGTH: usize;
    const MAX_LENGTH: Option<usize>;

    /// Contract: if data length is between MIN_LENGTH and MAX_LENGTH
    /// the function must return Some successfully. Otherwise
    /// panics may occur.
    fn extract<'a>(data: &'a [u8]) -> Option<(Self, usize)>
    where
        Self: Sized;
}

const fn sized_predicate<T: ByteCopy>() -> bool {
    match T::MAX_LENGTH {
        Some(len) => len == T::MIN_LENGTH,
        None => false,
    }
}

pub trait ByteCopySized<const LEN: usize> {
    fn extract(data: [u8; LEN]) -> Self
    where
        Self: Sized;
}

impl<const LEN: usize, T: ByteCopy> ByteCopySized<LEN> for T
where
    Lift<{ sized_predicate::<T>() }>: True,
{
    fn extract(data: [u8; LEN]) -> Self
    where
        Self: Sized,
    {
        Self::extract(&data[..]).unwrap().0
    }
}

impl ByteCopy for u8 {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: Option<usize> = Some(1);

    fn extract<'a>(data: &'a [u8]) -> Option<(Self, usize)> {
        if !data.is_empty() {
            Some((data[0], 1))
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! fold_maybe_tuple {
    ( $head:expr, $($tail:expr,)* ) => {
        if let Some(head) = $head {
            if let Some(tail) = fold_maybe_tuple!($($tail,)*) {
                Some(head + tail)
            } else {
                None
            }
        } else {
            None
        }
    };
    () => { Some(0) };
}

macro_rules! tuple_impls {
    ( $head:ident, $( $tail:ident, )* ) => {
        impl<$head, $( $tail ),*> ByteCopy for ($head, $( $tail ),*)
        where
            $head: ByteCopy,
            $( $tail: ByteCopy ),*
        {
            const MIN_LENGTH: usize = { <$head>::MIN_LENGTH $(+ <$tail>::MIN_LENGTH)* };
            const MAX_LENGTH: Option<usize> = {
                crate::fold_maybe_tuple!(<$head>::MAX_LENGTH, $(<$tail>::MAX_LENGTH,)*)
            };

            fn extract<'a>(#[allow(unused_mut)] mut data: &'a [u8]) -> Option<(Self, usize)> {
                let mut consumed = 0;
                #[allow(non_snake_case, unused_assignments)]
                let $head: $head;
                $(
                    #[allow(non_snake_case, unused_assignments)]
                    let $tail: $tail;
                )*
                #[allow(unused_variables, unused_assignments)]
                let mut last_len = 0;
                #[allow(unused_assignments)]
                if let Some((output, len)) = ByteCopy::extract(data) {
                    consumed += len;
                    $head = output;
                    last_len = len;
                } else {
                    return None;
                }
                $(
                    if last_len >= data.len() {
                        return None;
                    }
                    data = &data[last_len..];
                    #[allow(unused_assignments)]
                    if let Some((output, len)) = ByteCopy::extract(data) {
                        $tail = output;
                        consumed += len;
                        last_len = len;
                    } else {
                        return None;
                    }
                )*
                Some((($head, $($tail, )*), consumed))
            }
        }
        tuple_impls!($( $tail, )*);
    };
    () => {};
}

tuple_impls!(A, B, C, D, E, F, G, H, I, J,);

impl<const LEN: usize, T: ByteCopy> ByteCopy for [T; LEN]
where
    [T; LEN]: arrayvec::Array<Item = T>,
    [Option<T>; LEN]: arrayvec::Array<Item = Option<T>>,
{
    const MAX_LENGTH: Option<usize> = {
        match T::MAX_LENGTH {
            Some(item) => Some(item * LEN),
            None => None,
        }
    };
    const MIN_LENGTH: usize = T::MIN_LENGTH * LEN;

    fn extract<'a>(mut data: &'a [u8]) -> Option<(Self, usize)>
    where
        Self: Sized,
    {
        let mut consumed = 0;
        let mut output = [None::<T>; LEN];
        let mut last_len = 0;
        for item in output.iter_mut() {
            if last_len >= data.len() {
                return None;
            }
            data = &data[last_len..];
            if let Some((output, len)) = ByteCopy::extract(data) {
                *item = Some(output);
                consumed += len;
                last_len = len;
            } else {
                break;
            }
        }
        let output: Option<ArrayVec<[T; LEN]>> = ArrayVec::from(output).into_iter().collect();
        output
            .map(ArrayVec::into_inner)
            .transpose()
            .unwrap_or_else(|_| panic!())
            .map(|item| (item, consumed))
    }
}

pub enum Lift<const EXPRESSION: bool> {}
mod predicate_sealed {
    #[allow(unused_braces)]
    impl TSealed for super::Lift<{ true }> {}
    pub trait TSealed {}
    #[allow(unused_braces)]
    impl FSealed for super::Lift<{ false }> {}
    pub trait FSealed {}
}
pub trait True: predicate_sealed::TSealed {}
#[allow(unused_braces)]
impl True for Lift<{ true }> {}
pub trait False: predicate_sealed::FSealed {}
#[allow(unused_braces)]
impl False for Lift<{ false }> {}

pub trait ExactReceiver<const LEN: usize>: Stream<Item = [u8; LEN]> {
    type Inner: Receiver<LEN> + Unpin;

    fn into_inner(self) -> Self::Inner
    where
        Self: Sized;
}
