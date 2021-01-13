use core::{
    cell::UnsafeCell,
    convert::{TryFrom, TryInto},
    iter::{repeat, Step},
    marker::PhantomData,
    ops::{Add, Bound, Index, IndexMut, RangeBounds, Rem, Sub},
};

use num_traits::{one, zero, One, Zero};

use crate::LedStrip;

pub trait Space {
    type Coordinate;
    type Target: Space;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate>;
}

pub trait Linear: Space {
    fn len(&self) -> Self::Coordinate;
}

#[derive(Clone)]
pub struct StripSpace(pub(super) usize);

impl Space for StripSpace {
    type Coordinate = usize;
    type Target = Self;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self as Space>::Coordinate> {
        Some(coord)
    }
}

impl Linear for StripSpace {
    fn len(&self) -> Self::Coordinate {
        self.0
    }
}

pub trait LinearSpatialExt: Spatial
where
    <Self as Spatial>::Space: Linear,
    <<Self as Spatial>::Space as Space>::Coordinate: Zero + Step,
{
    fn fill_from<T: IntoIterator<Item = [u8; 3]>>(&mut self, iter: T) -> &mut Self;

    fn fill(&mut self, color: [u8; 3]) -> &mut Self {
        self.fill_from(repeat(color))
    }

    fn clear(&mut self) -> &mut Self {
        self.fill_from(repeat([0, 0, 0]))
    }
}

impl<T: Spatial> LinearSpatialExt for T
where
    <Self as Spatial>::Space: Linear,
    <<Self as Spatial>::Space as Space>::Coordinate: Zero + Step,
{
    default fn fill_from<U: IntoIterator<Item = [u8; 3]>>(&mut self, iter: U) -> &mut Self {
        let zero: <<Self as Spatial>::Space as Space>::Coordinate = zero();
        let len = self.space().len();

        for (color, idx) in iter.into_iter().zip(zero..len) {
            if let Some(led) = self.get_mut(idx) {
                *led = color;
            } else {
                break;
            }
        }
        self
    }
}

#[derive(Clone)]
pub struct FlipX<T: Cartesian2d> {
    space: T,
    width: T::Axis,
    height: Option<T::Axis>,
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
    fn height(&self) -> Option<Self::Axis> {
        self.height
    }
}

impl<T: Cartesian2d> FlipX<T> {
    pub fn new(space: T) -> Option<Self> {
        Some(FlipX {
            width: space.width()?,
            height: space.height(),
            space,
        })
    }
}

#[derive(Clone)]
pub struct FlipY<T: Cartesian2d> {
    space: T,
    height: T::Axis,
    width: Option<T::Axis>,
}

impl<T: Cartesian2d> Space for FlipY<T>
where
    T::Axis: One + Copy + Sub<Output = T::Axis>,
{
    type Coordinate = (T::Axis, T::Axis);
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<T::Target as Space>::Coordinate> {
        Some(
            self.space
                .transform((coord.0, self.height - coord.1 - one()))?,
        )
    }
}

impl<T: Cartesian2d> Cartesian2d for FlipY<T>
where
    T::Axis: Copy + One + Sub<Output = T::Axis>,
{
    type Axis = T::Axis;

    fn width(&self) -> Option<Self::Axis> {
        self.width
    }
    fn height(&self) -> Option<Self::Axis> {
        Some(self.height)
    }
}

impl<T: Cartesian2d> FlipY<T> {
    pub fn new(space: T) -> Option<Self> {
        Some(FlipY {
            width: space.width(),
            height: space.height()?,
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

    fn flip_y(self) -> Option<FlipY<Self>>
    where
        Self: Sized,
    {
        FlipY::new(self)
    }

    fn constrain_height(self, height: Self::Axis) -> Option<ConstrainHeight<Self>>
    where
        Self: Sized,
        Self::Axis: Ord,
    {
        ConstrainHeight::new(self, height)
    }

    fn into_torus(self) -> Option<Torus<Self>>
    where
        Self: Sized,
    {
        Some(Torus {
            height: self.height()?,
            width: self.width()?,
            space: self,
        })
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

    fn into_line(
        self,
        start: <Self::Space as Space>::Coordinate,
        end: <Self::Space as Space>::Coordinate,
    ) -> Option<BresenhamLine<Self>>
    where
        Self: Sized,
        <<Self as Spatial>::Space as Cartesian2d>::Axis: TryFrom<i32> + TryInto<i32>,
    {
        if let Some(width) = self.space().width() {
            if start.0 > width || end.0 > width {
                return None;
            }
        }
        if let Some(height) = self.space().height() {
            if start.1 > height || end.1 > height {
                return None;
            }
        }

        let mut length = 0;
        bresenham(start, end, |_, _| {
            length += 1;
            false
        });

        Some(BresenhamLine {
            data: self,
            space: BresenhamSpace {
                _space: PhantomData,
                end,
                start,
                length,
            },
        })
    }
}

#[derive(Clone)]
pub struct Torus<T: Cartesian2d> {
    space: T,
    width: T::Axis,
    height: T::Axis,
}

impl<T: Cartesian2d> Space for Torus<T>
where
    T::Axis: Rem<Output = T::Axis> + Copy,
{
    type Coordinate = T::Coordinate;
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<T::Target as Space>::Coordinate> {
        self.space
            .transform((coord.0 % self.width, coord.1 % self.height))
    }
}

impl<T: Cartesian2d> Cartesian2d for Torus<T>
where
    <T as Cartesian2d>::Axis: Rem<Output = T::Axis> + Copy,
{
    type Axis = T::Axis;

    fn width(&self) -> Option<Self::Axis> {
        None
    }

    fn height(&self) -> Option<Self::Axis> {
        None
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

impl<T: Copy + Into<usize>> Space for SwitchbackGrid<StripSpace, T> {
    type Coordinate = (T, T);
    type Target = StripSpace;

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

impl<T> SwitchbackGrid<StripSpace, T> {
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

pub trait MapSpace<T: Space<Target = <Self::Space as Space>::Target>>: Spatial {
    type MapSpace: Spatial<Space = T>;

    fn map_space<F: FnOnce(Self::Space) -> Option<T>>(self, call: F) -> Option<Self::MapSpace>;
}

impl Spatial for LedStrip {
    type Space = StripSpace;
    type Range = LedStrip;

    fn range<T: RangeBounds<usize>>(&mut self, range: T) -> Option<LedStrip> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.0.get_mut().len(),
        };
        if end > self.0.get_mut().len() || end < start {
            return None;
        }
        Some(LedStrip(
            unsafe {
                UnsafeCell::new(core::slice::from_raw_parts_mut(
                    self.0.get_mut().as_mut_ptr().add(start),
                    end - start,
                ))
            },
            StripSpace(end - start),
        ))
    }

    fn space(&self) -> &StripSpace {
        &self.1
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut [u8; 3]> {
        self.0.get_mut().get_mut(idx)
    }
    fn get(&self, idx: usize) -> Option<&[u8; 3]> {
        unsafe { &*self.0.get() }.get(idx)
    }
}

#[derive(Clone)]
pub struct Transformed<T, U> {
    data: T,
    space: U,
}

impl<T, U> Transformed<T, U> {
    pub fn into_inner(self) -> T {
        self.data
    }
}

#[derive(Clone)]
pub struct CartesianRange<T, U: Cartesian2d> {
    data: T,
    space: U,
    start: (U::Axis, U::Axis),
}

#[derive(Clone)]
pub struct LinearRange<T, U: Linear> {
    data: T,
    space: U,
    start: U::Coordinate,
}

impl<T, U: Linear + Subspace> LinearRange<T, U> {
    pub fn shift_add(
        self,
        by: <<Self as Spatial>::Space as Space>::Coordinate,
    ) -> Option<LinearRange<T, LinearSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Coordinate: Add<Output = U::Coordinate> + PartialOrd + Copy,
        U::Parent: Linear<Coordinate = U::Coordinate> + Clone,
    {
        let parent = self.space.parent();
        let offset = self.space.offset();
        if self.space.len() + by + offset > parent.len() {
            return None;
        }
        Some(LinearRange {
            start: self.start + by,
            space: LinearSubspace {
                space: parent.clone(),
                offset: offset + by,
                length: self.space.len(),
            },
            data: self.data,
        })
    }

    pub fn shift_sub(
        self,
        by: <<Self as Spatial>::Space as Space>::Coordinate,
    ) -> Option<LinearRange<T, LinearSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Coordinate:
            Add<Output = U::Coordinate> + Sub<Output = U::Coordinate> + PartialOrd + Copy,
        U::Parent: Linear<Coordinate = U::Coordinate> + Clone,
    {
        let parent = self.space.parent();
        let offset = self.space.offset();
        if self.start < by {
            return None;
        }
        Some(LinearRange {
            start: self.start - by,
            space: LinearSubspace {
                space: parent.clone(),
                offset: offset - by,
                length: self.space.len(),
            },
            data: self.data,
        })
    }

    pub fn shift<V: ExtractSign<Output = U::Coordinate>>(
        self,
        by: V,
    ) -> Option<LinearRange<T, LinearSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Coordinate:
            Add<Output = U::Coordinate> + Sub<Output = U::Coordinate> + PartialOrd + Copy + One,
        U::Parent: Linear<Coordinate = U::Coordinate> + Clone,
        T: Clone + Spatial<Space = <<U as Subspace>::Parent as Space>::Target>,
    {
        let this = self;
        let this = match by.extract_sign() {
            (value, Sign::Positive) => this.shift_add(value)?,
            (value, Sign::Negative) => this.shift_sub(value)?,
        };
        Some(this)
    }
}

impl<T, U: Cartesian2d + Subspace> CartesianRange<T, U> {
    pub fn shift_add(
        self,
        by: <<Self as Spatial>::Space as Space>::Coordinate,
    ) -> Option<CartesianRange<T, CartesianSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Axis: Add<Output = U::Axis> + PartialOrd + Copy,
        U::Parent: Cartesian2d<Axis = U::Axis> + Clone,
    {
        let parent = self.space.parent();
        let offset = self.space.offset();
        if let (Some(this_width), Some(width)) = (self.space.width(), parent.width()) {
            if this_width + by.0 + offset.0 > width {
                return None;
            }
        }
        if let (Some(this_height), Some(height)) = (self.space.height(), parent.height()) {
            if this_height + by.1 + offset.1 > height {
                return None;
            }
        }
        Some(CartesianRange {
            start: (self.start.0 + by.0, self.start.1 + by.1),
            space: CartesianSubspace {
                space: parent.clone(),
                offset: (offset.0 + by.0, offset.1 + by.1),
                size: (self.space().width()?, self.space().height()?),
            },
            data: self.data,
        })
    }

    pub fn shift_sub(
        self,
        by: <<Self as Spatial>::Space as Space>::Coordinate,
    ) -> Option<CartesianRange<T, CartesianSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Axis: Add<Output = U::Axis> + Sub<Output = U::Axis> + PartialOrd + Copy,
        U::Parent: Cartesian2d<Axis = U::Axis> + Clone,
    {
        let parent = self.space.parent();
        let offset = self.space.offset();
        if self.start.0 < by.0 || self.start.1 < by.1 {
            return None;
        }
        Some(CartesianRange {
            start: (self.start.0 - by.0, self.start.1 - by.1),
            space: CartesianSubspace {
                space: parent.clone(),
                offset: (offset.0 - by.0, offset.1 - by.1),
                size: (self.space().width()?, self.space().height()?),
            },
            data: self.data,
        })
    }

    pub fn shift<V: ExtractSign<Output = U::Axis>>(
        self,
        by: (V, V),
    ) -> Option<CartesianRange<T, CartesianSubspace<U::Parent>>>
    where
        Self: Sized,
        Self: Spatial<Space = U>,
        U::Axis: Add<Output = U::Axis> + Sub<Output = U::Axis> + PartialOrd + Copy + Zero + One,
        U::Parent: Cartesian2d<Axis = U::Axis> + Clone,
        T: Clone + Spatial<Space = <<U as Subspace>::Parent as Space>::Target>,
    {
        let this = self;
        let this = match by.0.extract_sign() {
            (value, Sign::Positive) => this.shift_add((value, zero()))?,
            (value, Sign::Negative) => this.shift_sub((value, zero()))?,
        };
        let this = match by.1.extract_sign() {
            (value, Sign::Positive) => this.shift_add((zero(), value)),
            (value, Sign::Negative) => this.shift_sub((zero(), value)),
        };
        this
    }
}

pub enum Sign {
    Positive,
    Negative,
}

pub trait ExtractSign {
    type Output;

    fn extract_sign(self) -> (Self::Output, Sign);
}

macro_rules! impl_extract_sign {
    ($($a:ty, $b:ty),*) => {
        $(
            impl ExtractSign for $a {
                type Output = $b;

                fn extract_sign(self) -> (Self::Output, Sign) {
                    if self > 0 {
                        (self as $b, Sign::Positive)
                    } else {
                        ((-self) as $b, Sign::Negative)
                    }
                }
            }
        )*
    };
}

impl_extract_sign!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

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

pub trait Subspace: Space {
    type Parent: Space;

    fn parent(&self) -> &Self::Parent;
    fn offset(&self) -> <Self::Parent as Space>::Coordinate;
}

#[derive(Clone)]
pub struct CartesianSubspace<T: Cartesian2d> {
    space: T,
    offset: (T::Axis, T::Axis),
    size: (T::Axis, T::Axis),
}

impl<T: Cartesian2d> Subspace for CartesianSubspace<T>
where
    T::Axis: Add<Output = T::Axis> + Copy,
{
    type Parent = T;

    fn parent(&self) -> &Self::Parent {
        &self.space
    }

    fn offset(&self) -> <Self::Parent as Space>::Coordinate {
        self.offset
    }
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

#[derive(Clone)]
pub struct LinearSubspace<T: Linear> {
    space: T,
    offset: T::Coordinate,
    length: T::Coordinate,
}

impl<T: Linear> Subspace for LinearSubspace<T>
where
    T::Coordinate: Add<Output = T::Coordinate> + Copy,
{
    type Parent = T;

    fn parent(&self) -> &Self::Parent {
        &self.space
    }

    fn offset(&self) -> <Self::Parent as Space>::Coordinate {
        self.offset
    }
}

impl<T: Linear> Space for LinearSubspace<T>
where
    T::Coordinate: Add<Output = T::Coordinate> + Copy,
{
    type Coordinate = T::Coordinate;
    type Target = T::Target;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate> {
        Some(self.space.transform(coord + self.offset)?)
    }
}

impl<T: Linear> Linear for LinearSubspace<T>
where
    T::Coordinate: Add<Output = T::Coordinate> + Copy,
{
    fn len(&self) -> Self::Coordinate {
        self.length
    }
}

impl<T: Spatial + Clone, U: Space<Target = T::Space>> Spatial for Transformed<T, U>
where
    U: Cartesian2d + Clone,
    <U as Cartesian2d>::Axis: Zero
        + One
        + Add<Output = <U as Cartesian2d>::Axis>
        + Sub<Output = <U as Cartesian2d>::Axis>
        + Copy
        + PartialOrd,
{
    type Space = U;

    type Range = CartesianRange<T, CartesianSubspace<U>>;

    fn range<V: RangeBounds<U::Coordinate>>(&mut self, range: V) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Unbounded => (zero(), zero()),
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => (self.space.width()?, self.space.height()?),
        };
        if end.0 < start.0 || end.1 < start.1 {
            return None;
        }
        if let Some(width) = self.space.width() {
            if end.0 > width {
                return None;
            }
        }
        if let Some(height) = self.space.height() {
            if end.1 > height {
                return None;
            }
        }
        Some(CartesianRange {
            start,
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

impl<
        T: Spatial,
        S: Space<Target = T::Space> + Cartesian2d + Clone,
        U: Space<Target = <Self::Space as Space>::Target> + Clone + Cartesian2d<Axis = S::Axis>,
    > MapSpace<U> for Transformed<T, S>
where
    S::Axis: Zero + One + Sub<Output = S::Axis> + Copy + PartialOrd,
    T: Clone,
{
    type MapSpace = Transformed<T, U>;

    fn map_space<F: FnOnce(Self::Space) -> Option<U>>(self, call: F) -> Option<Self::MapSpace> {
        Some(Transformed {
            data: self.data,
            space: call(self.space)?,
        })
    }
}

impl<T: Spatial + Clone, U: Space<Target = T::Space>> Spatial for CartesianRange<T, U>
where
    U: Cartesian2d + Clone,
    <U as Cartesian2d>::Axis:
        Zero + One + Sub<Output = <U as Cartesian2d>::Axis> + Copy + PartialOrd,
{
    type Space = U;

    type Range = CartesianRange<T, CartesianSubspace<U>>;

    fn range<V: RangeBounds<U::Coordinate>>(&mut self, range: V) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Unbounded => (self.start.0, self.start.1),
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => (bound.0 + one(), bound.1 + one()),
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => (self.space.width()?, self.space.height()?),
        };
        if end.0 < start.0 || end.1 < start.1 {
            return None;
        }
        if let Some(width) = self.space.width() {
            if end.0 > width {
                return None;
            }
        }
        if let Some(height) = self.space.height() {
            if end.1 > height {
                return None;
            }
        };
        Some(CartesianRange {
            start: (self.start.0 + start.0, self.start.1 + start.1),
            data: self.data.clone(),
            space: CartesianSubspace {
                space: self.space.clone(),
                offset: start,
                size: (end.0 - start.0, end.1 - start.1),
            },
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

impl<T: Spatial + Clone, U: Space<Target = T::Space>> Spatial for LinearRange<T, U>
where
    U: Linear + Clone,
    <U as Space>::Coordinate:
        Zero + One + Sub<Output = <U as Space>::Coordinate> + Copy + PartialOrd,
{
    type Space = U;
    type Range = LinearRange<T, LinearSubspace<U>>;

    fn range<V: RangeBounds<U::Coordinate>>(&mut self, range: V) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + one(),
            Bound::Unbounded => self.start,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound + one(),
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.space.len(),
        };
        if end < start {
            return None;
        }
        if end > self.space.len() {
            return None;
        }

        Some(LinearRange {
            start: self.start + start,
            data: self.data.clone(),
            space: LinearSubspace {
                space: self.space.clone(),
                offset: start,
                length: end - start,
            },
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

impl<
        T: Spatial,
        S: Space<Target = T::Space> + Cartesian2d + Clone,
        U: Space<Target = <Self::Space as Space>::Target> + Clone + Cartesian2d<Axis = S::Axis>,
    > MapSpace<U> for CartesianRange<T, S>
where
    S::Axis: Zero + One + Sub<Output = S::Axis> + Copy + PartialOrd,
    T: Clone,
{
    type MapSpace = CartesianRange<T, U>;

    fn map_space<F: FnOnce(Self::Space) -> Option<U>>(self, call: F) -> Option<Self::MapSpace> {
        Some(CartesianRange {
            data: self.data,
            space: call(self.space)?,
            start: self.start,
        })
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

#[derive(Clone)]
pub struct BresenhamSpace<T: Cartesian2d> {
    _space: PhantomData<T>,
    start: T::Coordinate,
    end: T::Coordinate,
    length: usize,
}

impl<T: Cartesian2d> Space for BresenhamSpace<T>
where
    T::Axis: TryInto<i32> + TryFrom<i32>,
    T::Coordinate: Copy,
{
    type Coordinate = usize;
    type Target = T;

    fn transform(&self, coord: Self::Coordinate) -> Option<<Self::Target as Space>::Coordinate> {
        let mut ctr = 0;
        let mut out = None;
        bresenham(self.start, self.end, |x, y| {
            if ctr == coord {
                out = Some((x, y));
                return true;
            }
            ctr += 1;
            false
        })?;
        out
    }
}

impl<T: Cartesian2d> Linear for BresenhamSpace<T>
where
    T::Axis: TryInto<i32> + TryFrom<i32>,
    T::Coordinate: Copy,
{
    fn len(&self) -> Self::Coordinate {
        self.length
    }
}

pub struct BresenhamLine<T: Spatial>
where
    T::Space: Cartesian2d,
{
    data: T,
    space: BresenhamSpace<T::Space>,
}

impl<T: Spatial + Clone> Spatial for BresenhamLine<T>
where
    T::Space: Cartesian2d + Clone,
    <T::Space as Cartesian2d>::Axis: TryInto<i32> + TryFrom<i32> + Copy,
{
    type Space = BresenhamSpace<T::Space>;
    type Range = LinearRange<T, LinearSubspace<Self::Space>>;

    fn range<U: RangeBounds<<Self::Space as Space>::Coordinate>>(
        &mut self,
        range: U,
    ) -> Option<Self::Range> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.space.len(),
        };
        if end < start {
            return None;
        }
        if end > self.space.len() {
            return None;
        }

        Some(LinearRange {
            start,
            data: self.data.clone(),
            space: LinearSubspace {
                space: self.space.clone(),
                offset: start,
                length: end - start,
            },
        })
    }

    fn space(&self) -> &Self::Space {
        &self.space
    }

    fn get(&self, index: <Self::Space as Space>::Coordinate) -> Option<&[u8; 3]> {
        self.data.get(self.space.transform(index)?)
    }

    fn get_mut(&mut self, index: <Self::Space as Space>::Coordinate) -> Option<&mut [u8; 3]> {
        self.data.get_mut(self.space.transform(index)?)
    }
}

impl<T: Spatial + Clone> LinearSpatialExt for BresenhamLine<T>
where
    T::Space: Cartesian2d + Clone,
    <T::Space as Cartesian2d>::Axis: TryInto<i32> + TryFrom<i32> + Copy,
{
    fn fill_from<U: IntoIterator<Item = [u8; 3]>>(&mut self, iter: U) -> &mut Self {
        let mut iter = iter.into_iter();
        bresenham(self.space.start, self.space.end, |x, y| {
            if let Some(item) = self.data.get_mut((x, y)) {
                if let Some(value) = iter.next() {
                    *item = value;
                    false
                } else {
                    true
                }
            } else {
                false
            }
        });
        self
    }
}

pub fn bresenham<T: TryInto<i32> + TryFrom<i32>, F: FnMut(T, T) -> bool>(
    a: (T, T),
    b: (T, T),
    mut plot: F,
) -> Option<()> {
    let (mut x0, mut y0) = (a.0.try_into().ok()?, a.1.try_into().ok()?);
    let (x1, y1) = (b.0.try_into().ok()?, b.1.try_into().ok()?);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if plot(x0.try_into().ok()?, y0.try_into().ok()?) || x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    Some(())
}
