//! Traits and types representing things that can be sentover a breadboard wire

use std::iter;

use super::{Line, LineInner};

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


// FIXME: verify line group is from same breadboard
pub trait InputGroup<T: LineValue> {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner>;

    fn as_vec(&self) -> Vec<LineInner> {
        self.iter_inputs().collect()
    }
}

impl<T: LineValue> InputGroup<T> for Line<'_, T> {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.inner)
    }
}

impl<T: LineValue> InputGroup<T> for (Line<'_, T>,) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
    }
}

impl<T: LineValue> InputGroup<T> for (Line<'_, T>, Line<'_, T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<'_, T>, Line<'_, T>, Line<'_, T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<'_, T>, Line<'_, T>, Line<'_, T>, Line<'_, T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
            .chain(iter::once(self.3.inner))
    }
}

impl<T: LineValue> InputGroup<T> for (Line<'_, T>, Line<'_, T>, Line<'_, T>, Line<'_, T>, Line<'_, T>) {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        iter::once(self.0.inner)
            .chain(iter::once(self.1.inner))
            .chain(iter::once(self.2.inner))
            .chain(iter::once(self.3.inner))
            .chain(iter::once(self.4.inner))
    }
}

impl<T: LineValue> InputGroup<T> for [Line<'_, T>] {
    fn iter_inputs(&self) -> impl Iterator<Item = LineInner> {
        self.iter().map(|line| line.inner)
    }
}