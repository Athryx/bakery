//! Traits and types representing things that can be sentover a breadboard wire

use std::iter;
use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul, Div, Rem, Neg, Not, BitAnd, BitOr};

use super::Breadboard;

/// Represents the output line of a certain breadboard component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineInner {
    pub(crate) component_index: usize,
    pub(crate) output_index: usize,
}

pub struct Line<T: LineValue + ?Sized> {
    pub(crate) inner: LineInner,
    // the line pretends it owns a type T inside the breadboard in its wires
    pub(crate) breadboard: Breadboard,
    _marker: PhantomData<T>,
}

impl<T: LineValue + ?Sized> Line<T> {
    pub(crate) fn new(breadboard: Breadboard, component_index: usize, output_index: usize) -> Self {
        Line {
            inner: LineInner {
                component_index,
                output_index,
            },
            breadboard,
            _marker: PhantomData,
        }
    }
}

impl<T: LineValue + ?Sized> Clone for Line<T> {
    fn clone(&self) -> Self {
        Line {
            inner: self.inner,
            breadboard: self.breadboard.clone(),
            _marker: PhantomData,
        }
    }
}

macro_rules! line_op {
    ($trait:ident, $method:ident, $bb_method:ident, $lhs:ty, $rhs:ty, $out:ty) => {
        impl $trait<&Line<$rhs>> for &Line<$lhs> {
            type Output = Line<$out>;

            fn $method(self, rhs: &Line<$rhs>) -> Self::Output {
                self.breadboard.$bb_method(self.clone(), rhs.clone())
            }
        }
    };
    (Add, $bb_method:tt, $lhs:tt, $rhs:tt, $out:tt) => {
        line_op!(Add, add, $bb_method, $lhs, $rhs, $out);
    };
    (Sub, $bb_method:tt, $lhs:tt, $rhs:tt, $out:tt) => {
        line_op!(Sub, sub, $bb_method, $lhs, $rhs, $out);
    };
    (Mul, $bb_method:tt, $lhs:tt, $rhs:tt, $out:tt) => {
        line_op!(Mul, mul, $bb_method, $lhs, $rhs, $out);
    };
    (Div, $bb_method:tt, $lhs:tt, $rhs:tt, $out:tt) => {
        line_op!(Div, div, $bb_method, $lhs, $rhs, $out);
    };
}

line_op!(Add, add, BNumber, BNumber, BNumber);
line_op!(Add, addv, BVector3, BVector3, BVector3);

line_op!(Sub, sub, BNumber, BNumber, BNumber);
line_op!(Sub, subv, BVector3, BVector3, BVector3);

line_op!(Mul, mul, BNumber, BNumber, BNumber);
line_op!(Mul, rotate, BVector3, BQuaternion, BVector3);
// TODO: make this work with vector * number also
line_op!(Mul, scale, BNumber, BVector3, BVector3);
line_op!(Mul, compose_rotations, BQuaternion, BQuaternion, BQuaternion);

line_op!(Div, div, BNumber, BNumber, BNumber);
line_op!(Div, scaler_div, BVector3, BNumber, BVector3);

line_op!(Rem, rem, modulo, BNumber, BNumber, BNumber);

line_op!(BitAnd, bitand, and, BNumber, BNumber, BNumber);
line_op!(BitOr, bitor, or, BNumber, BNumber, BNumber);

impl Not for &Line<BNumber> {
    type Output = Line<BNumber>;

    fn not(self) -> Self::Output {
        self.breadboard.not(self.clone())
    }
}

impl Neg for &Line<BNumber> {
    type Output = Line<BNumber>;

    fn neg(self) -> Self::Output {
        self.breadboard.negate(self.clone())
    }
}

impl Line<BVector3> {
    pub fn cross(&self, rhs: &Self) -> Line<BVector3> {
        self.breadboard.cross(self.clone(), rhs.clone())
    }

    pub fn dot(&self, rhs: &Self) -> Line<BNumber> {
        self.breadboard.dot(self.clone(), rhs.clone())
    }

    pub fn x(&self) -> Line<BNumber> {
        self.breadboard.x(self.clone())
    }

    pub fn y(&self) -> Line<BNumber> {
        self.breadboard.y(self.clone())
    }

    pub fn z(&self) -> Line<BNumber> {
        self.breadboard.z(self.clone())
    }

    pub fn magnitude(&self) -> Line<BNumber> {
        self.breadboard.magnitude(self.clone())
    }

    pub fn square_magnitude(&self) -> Line<BNumber> {
        self.breadboard.square_magnitude(self.clone())
    }
}

impl Line<BQuaternion> {
    pub fn inverse(&self) -> Line<BQuaternion> {
        self.breadboard.rotation_inverse(self.clone())
    }
}

pub fn b_if<T: LineValue + ?Sized>(condition: &Line<BNumber>, true_value: &Line<T>, false_value: &Line<T>) -> Line<T> {
    condition.breadboard.b_if(condition.clone(), true_value.clone(), false_value.clone())
}


mod private {
    pub trait Sealed {}
}

pub trait LineValue: private::Sealed {}

pub struct BNumber;

impl private::Sealed for BNumber {}
impl LineValue for BNumber {}

pub struct BVector3;

impl private::Sealed for BVector3 {}
impl LineValue for BVector3 {}

pub struct BQuaternion;

impl private::Sealed for BQuaternion {}
impl LineValue for BQuaternion {}

pub struct BString;

impl private::Sealed for BString {}
impl LineValue for BString {}


// FIXME: verify line group is from same breadboard
pub trait InputGroup<T: LineValue> {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner>;

    fn as_vec(&self) -> Vec<LineInner> {
        self.iter_inputs().collect()
    }
}

impl<T: LineValue> InputGroup<T> for Line<T> {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.inner)
    }
}

impl<T: LineValue> InputGroup<T> for (Line<T>,) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
    }
}

impl<T: LineValue> InputGroup<T> for (Line<T>, Line<T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<T>, Line<T>, Line<T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<T>, Line<T>, Line<T>, Line<T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
            .chain(iter::once(self.3.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<T>, Line<T>, Line<T>, Line<T>, Line<T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
            .chain(iter::once(self.3.inner))
            .chain(iter::once(self.4.inner))
    }
}

impl<T: LineValue> InputGroup<T> for [Line<T>] {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        self.iter().map(|line| line.inner)
    }
}