use std::ops::Mul;

use custos::prelude::*;

pub trait MulBuf<T, S: Shape = (), D: Device = Self>: Sized + Device {
    fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Self, S>;
}

impl<T, S, D> MulBuf<T, S, D> for CPU
where
    T: Mul<Output = T> + Copy,
    S: Shape,
    D: MainMemory,
{
    fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, CPU, S> {
        let mut out = self.retrieve(lhs.len(), (lhs, rhs));

        for ((lhs, rhs), out) in lhs.iter().zip(&*rhs).zip(&mut out) {
            *out = *lhs * *rhs;
        }

        out
    }
}

fn main() {
    let device = CPU::new();

    let lhs = Buffer::from((&device, &[-1, 2, 3, -4, 5, 9]));
    let rhs = Buffer::from((&device, &[4, -1, 7, 1, -2, 4]));

    let out = device.mul(&lhs, &rhs);
    assert_eq!(out.read(), [-4, -2, 21, -4, -10, 36]);
}
