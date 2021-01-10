use core::{
    cell::UnsafeCell,
    iter::Step,
    marker::PhantomData,
    ops::{Add, Bound, Index, IndexMut, RangeBounds, Sub},
};

use num_traits::{one, zero, One, Zero};

use crate::LedStrip;

pub trait Space {
    type Coordinate;
    type Target: Space;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate>;
}

#[derive(Clone)]
pub struct Linear;

impl Space for Linear {
    type Coordinate = usize;
    type Target = Self;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self as Space>::Coordinate> {
        Some(coord)
    }
}

#[derive(Clone)]
pub struct FlipX<T: Cartesian2d> {
    space: T,
    width: T::Axis,
}

impl<T: Cartesian2d> Space for FlipX<T>
where
    T::Axis: One + Copy + Sub<Output = T::Axis>,
{
    type Coordinate = (T::Axis, T::Axis);
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<T::Target as Space>::Coordinate> {
        Some(
            self.space
                .transform((self.width - coord.0 - one(), coord.1))?,
        )
    }
}

impl<T: Cartesian2d> Cartesian2d for FlipX<T>
where
    T::Axis: Copy + One + Sub<Output = T::Axis>,
{
    type Axis = T::Axis;

    fn width(&self) -> Option<Self::Axis> {
        Some(self.width)
    }
}

impl<T: Cartesian2d> FlipX<T> {
    pub fn new(space: T) -> Option<Self> {
        Some(FlipX {
            width: space.width()?,
            space,
        })
    }
}

pub trait Cartesian2dExt: Cartesian2d {
    fn flip_x(self) -> Option<FlipX<Self>>
    where
        Self: Sized,
    {
        FlipX::new(self)
    }

    fn constrain_height(self, height: Self::Axis) -> Option<ConstrainHeight<Self>>
    where
        Self: Sized,
        Self::Axis: Ord,
    {
        ConstrainHeight::new(self, height)
    }
}

pub trait CartesianSpatialExt: Spatial
where
    <Self as Spatial>::Space: Cartesian2d,
    <<Self as Spatial>::Space as Cartesian2d>::Axis: Step + Zero + Copy,
{
    fn fill(&mut self, color: [u8; 3]) -> Option<()> {
        for x in zero()..self.space().width()? {
            for y in zero()..self.space().height()? {
                *self.get_mut((x, y))? = color;
            }
        }
        Some(())
    }

    fn clear(&mut self) -> Option<()> {
        self.fill([0, 0, 0])
    }
}

impl<T: Spatial> CartesianSpatialExt for T
where
    <Self as Spatial>::Space: Cartesian2d,
    <<Self as Spatial>::Space as Cartesian2d>::Axis: Step + Zero + Copy,
{
}

impl<T: Cartesian2d> Cartesian2dExt for T {}

pub trait Cartesian2d:
    Space<Coordinate = (<Self as Cartesian2d>::Axis, <Self as Cartesian2d>::Axis)>
{
    type Axis;

    fn width(&self) -> Option<Self::Axis> {
        None
    }
    fn height(&self) -> Option<Self::Axis> {
        None
    }
}

impl<T: Space, U: Clone> Cartesian2d for SwitchbackGrid<T, U>
where
    Self: Space<Coordinate = (U, U)>,
{
    type Axis = U;

    fn width(&self) -> Option<Self::Axis> {
        Some(self.stride.clone())
    }
}

#[derive(Clone)]
pub struct SwitchbackGrid<T: Space, U> {
    stride: U,
    _marker: PhantomData<T>,
}

impl<T: Copy + Into<usize>> Space for SwitchbackGrid<Linear, T> {
    type Coordinate = (T, T);
    type Target = Linear;

    fn transform(&self, coord: Self::Coordinate) -> Option<usize> {
        if coord.0.into() >= self.stride.into() {
            return None;
        }
        Some(if coord.1.into() % 2 == 0 {
            coord.1.into() * self.stride.into() + coord.0.into()
        } else {
            (coord.1.into() + 1) * self.stride.into() - (1 + coord.0.into())
        })
    }
}

impl<T> SwitchbackGrid<Linear, T> {
    pub fn new(stride: T) -> Self {
        SwitchbackGrid {
            stride,
            _marker: PhantomData,
        }
    }
}

pub trait Spatial {
    type Space: Space;
    type Range: Spatial;

    fn range<U: RangeBounds<<Self::Space as Space>::Coordinate>>(
        &mut self,
        range: U,
    ) -> Option<Self::Range>;

    fn space(&self) -> &Self::Space;

    fn get(&self, index: <Self::Space as Space>::Coordinate) -> Option<&[u8; 3]>;
    fn get_mut(&mut self, index: <Self::Space as Space>::Coordinate) -> Option<&mut [u8; 3]>;
}

impl Spatial for LedStrip {
    type Space = Linear;
    type Range = LedStrip;

    fn range<T: RangeBounds<usize>>(&mut self, range: T) -> Option<LedStrip> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound - 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.0.get_mut().len(),
        };
        if end > self.0.get_mut().len() {
            return None;
        }
        Some(LedStrip(unsafe {
            UnsafeCell::new(core::slice::from_raw_parts_mut(
                self.0.get_mut().as_mut_ptr().add(start),
                end - start,
            ))
        }))
    }

    fn space(&self) -> &Linear {
        &Linear
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut [u8; 3]> {
        self.0.get_mut().get_mut(idx)
    }
    fn get(&self, idx: usize) -> Option<&[u8; 3]> {
        unsafe { &*self.0.get() }.get(idx)
    }
}

pub struct Transformed<T, U> {
    data: T,
    space: U,
}

impl<T, U> Transformed<T, U> {
    pub fn into_inner(self) -> T {
        self.data
    }
}

pub struct CartesianRange<T, U: Cartesian2d> {
    data: T,
    space: CartesianSubspace<U>,
    start: (U::Axis, U::Axis),
    end: (U::Axis, U::Axis),
}

#[derive(Clone)]
pub struct ConstrainHeight<T: Cartesian2d> {
    inner: T,
    height: T::Axis,
}

impl<T: Cartesian2d> ConstrainHeight<T>
where
    T::Axis: Ord,
{
    pub fn new(inner: T, height: T::Axis) -> Option<Self> {
        if let Some(original_height) = inner.height() {
            if original_height < height {
                return None;
            }
        }
        Some(ConstrainHeight { inner, height })
    }
}

impl<T: Cartesian2d> Space for ConstrainHeight<T> {
    type Coordinate = T::Coordinate;
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate> {
        self.inner.transform(coord)
    }
}

impl<T: Cartesian2d> Cartesian2d for ConstrainHeight<T>
where
    T::Axis: Copy,
{
    type Axis = T::Axis;

    fn width(&self) -> Option<Self::Axis> {
        self.inner.width()
    }

    fn height(&self) -> Option<Self::Axis> {
        Some(self.height)
    }
}

impl<T, U: Cartesian2d> CartesianRange<T, U> {
    pub fn into_inner(self) -> T {
        self.data
    }
}

#[derive(Clone)]
pub struct CartesianSubspace<T: Cartesian2d> {
    space: T,
    offset: (T::Axis, T::Axis),
    size: (T::Axis, T::Axis),
}

impl<T: Cartesian2d> Space for CartesianSubspace<T>
where
    T::Axis: Add<Output = T::Axis> + Copy,
{
    type Coordinate = T::Coordinate;
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate> {
        Some(
            self.space
                .transform((coord.0 + self.offset.0, coord.1 + self.offset.1))?,
        )
    }
}

impl<T: Cartesian2d> Cartesian2d for CartesianSubspace<T>
where
    T::Axis: Add<Output = T::Axis> + Copy,
{
    type Axis = T::Axis;

    fn width(&self) -> Option<Self::Axis> {
        Some(self.size.0)
    }

    fn height(&self) -> Option<Self::Axis> {
        Some(self.size.1)
    }
}

impl<T: Spatial + Clone, U: Space<Target = T::Space>> Spatial for Transformed<T, U>
where
    U: Cartesian2d + Clone,
    <U as Cartesian2d>::Axis: Zero
        + One
        + Add<Output = <U as Cartesian2d>::Axis>
        + Sub<Output = <U as Cartesian2d>::Axis>
        + Copy,
{
    type Space = U;

    type Range = CartesianRange<T, U>;

    fn range<V: RangeBounds<U::Coordinate>>(&mut self, range: V) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Unbounded => (zero(), zero()),
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => (bound.0 - one(), bound.1 - one()),
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => (self.space.width()?, self.space.height()?),
        };
        Some(CartesianRange {
            start,
            end,
            data: self.data.clone(),
            space: CartesianSubspace {
                space: self.space.clone(),
                offset: start,
                size: (end.0 - start.0, end.1 - start.1),
            },
        })
    }

    fn space(&self) -> &U {
        &self.space
    }

    fn get(&self, index: U::Coordinate) -> Option<&[u8; 3]> {
        self.data.get(self.space.transform(index)?)
    }
    fn get_mut(&mut self, index: U::Coordinate) -> Option<&mut [u8; 3]> {
        self.data.get_mut(self.space.transform(index)?)
    }
}

impl<T: Spatial + Clone, U: Space<Target = T::Space>> Spatial for CartesianRange<T, U>
where
    U: Cartesian2d + Clone,
    <U as Cartesian2d>::Axis: Zero
        + One
        + Add<Output = <U as Cartesian2d>::Axis>
        + Sub<Output = <U as Cartesian2d>::Axis>
        + Copy,
{
    type Space = CartesianSubspace<U>;

    type Range = CartesianRange<T, U>;

    fn range<V: RangeBounds<U::Coordinate>>(&mut self, range: V) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Unbounded => (zero(), zero()),
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => (bound.0 - one(), bound.1 - one()),
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => (zero(), zero()),
        };
        Some(CartesianRange {
            start: (self.start.0 + start.0, self.start.1 + start.1),
            end: (
                end.0 + (self.start.0 + start.0),
                end.1 + (self.start.1 + start.1),
            ),
            data: self.data.clone(),
            space: self.space.clone(),
        })
    }

    fn space(&self) -> &Self::Space {
        &self.space
    }

    fn get(&self, index: U::Coordinate) -> Option<&[u8; 3]> {
        self.data.get(self.space.transform(index)?)
    }
    fn get_mut(&mut self, index: U::Coordinate) -> Option<&mut [u8; 3]> {
        self.data.get_mut(self.space.transform(index)?)
    }
}

impl<T: Spatial, U: Space<Target = T::Space>> Index<U::Coordinate> for Transformed<T, U>
where
    Self: Spatial,
    <Self as Spatial>::Space: Space<Coordinate = U::Coordinate>,
{
    type Output = [u8; 3];

    fn index(&self, index: U::Coordinate) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: Spatial, U: Space<Target = T::Space>> IndexMut<U::Coordinate> for Transformed<T, U>
where
    Self: Spatial,
    <Self as Spatial>::Space: Space<Coordinate = U::Coordinate>,
{
    fn index_mut(&mut self, index: U::Coordinate) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: Spatial, U: Cartesian2d> Index<U::Coordinate> for CartesianRange<T, U>
where
    Self: Spatial,
    <Self as Spatial>::Space: Space<Coordinate = U::Coordinate>,
{
    type Output = [u8; 3];

    fn index(&self, index: U::Coordinate) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: Spatial, U: Cartesian2d> IndexMut<U::Coordinate> for CartesianRange<T, U>
where
    Self: Spatial,
    <Self as Spatial>::Space: Space<Coordinate = U::Coordinate>,
{
    fn index_mut(&mut self, index: U::Coordinate) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

pub trait SpatialExt: Spatial {
    fn project<U: Space<Target = Self::Space>>(self, space: U) -> Transformed<Self, U>
    where
        Self: Sized,
    {
        Transformed { data: self, space }
    }
}

impl<T: Spatial> SpatialExt for T {}
