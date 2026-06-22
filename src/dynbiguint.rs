use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, Sub};

pub trait MulStrategy<T, O> {
    fn mul(lhs: T, rhs: T) -> O;
}

pub struct StandardMul;
pub struct UnrolledMul;

pub struct Karatsuba;

impl<M> MulStrategy<&DynBigUint<M>, DynBigUint<M>> for Karatsuba {
    fn mul(lhs: &DynBigUint<M>, other: &DynBigUint<M>) -> DynBigUint<M> {

        let min = lhs.values.len().min(other.values.len());
        if min <= 64 {
            return UnrolledMul::mul(lhs, other);
        }

        let half = min >> 1;
        let x0: DynBigUint<M> = DynBigUint::new_with_slice(&lhs.values[0..half]);
        let x1: DynBigUint<M> = DynBigUint::new_with_slice(&lhs.values[half..]);
        let y0: DynBigUint<M> = DynBigUint::new_with_slice(&other.values[0..half]);
        let y1: DynBigUint<M> = DynBigUint::new_with_slice(&other.values[half..]);

        let z0 = Karatsuba::mul(&x0, &y0);
        let z2 = Karatsuba::mul(&x1, &y1);
        let z3 = Karatsuba::mul(&(x0 + x1), &(y0 + y1));
        let z1 = &(&z3 - &z2) - &z0;

        let mut result = DynBigUint::new_with_limbs(0, lhs.values.len() + other.values.len() + 1);

        result.copy_slice_to(&z0.values[0..], 0);
        result.add_slice_at(&z1.values[0..], half);
        result.add_slice_at(&z2.values[0..], 2*half);

        while result.values.len() > 1 && result.values[result.values.len() - 1] == 0 {
            result.values.pop();
        }
        result
    }
}


impl<M> MulStrategy<&DynBigUint<M>, DynBigUint<M>> for StandardMul {
    fn mul(lhs: &DynBigUint<M>, other: &DynBigUint<M>) -> DynBigUint<M> {
        let mut result = DynBigUint::new_with_limbs(0, lhs.values.len() + other.values.len());
        for i in 0..lhs.values.len() {
            let mut carry = 0u64;
            for j in 0..other.values.len() {
                (result.values[j + i], carry) =
                    lhs.values[i].carrying_mul_add(other.values[j], carry, result.values[j + i]);
            }
            if carry != 0 {
                result.values[other.values.len() + i] = carry;
            }
        }

        while result.values.len() > 1 && result.values[result.values.len() - 1] == 0 {
            result.values.pop();
        }
        result
    }
}

impl<M> MulStrategy<&DynBigUint<M>, DynBigUint<M>> for UnrolledMul {
    fn mul(lhs: &DynBigUint<M>, other: &DynBigUint<M>) -> DynBigUint<M> {
        let mut result = DynBigUint::new_with_limbs(0, lhs.values.len() + other.values.len());

        for i in (0..lhs.values.len() - lhs.values.len() % 4).step_by(4) {
            let mut carry0 = 0u64;
            let mut carry1 = 0u64;
            let mut carry2 = 0u64;
            let mut carry3 = 0u64;

            for j in 0..other.values.len() {
                (result.values[j + i], carry0) =
                    lhs.values[i].carrying_mul_add(other.values[j], carry0, result.values[j + i]);
                (result.values[j + i + 1], carry1) = lhs.values[i + 1].carrying_mul_add(
                    other.values[j],
                    carry1,
                    result.values[j + i + 1],
                );
                (result.values[j + i + 2], carry2) = lhs.values[i + 2].carrying_mul_add(
                    other.values[j],
                    carry2,
                    result.values[j + i + 2],
                );
                (result.values[j + i + 3], carry3) = lhs.values[i + 3].carrying_mul_add(
                    other.values[j],
                    carry3,
                    result.values[j + i + 3],
                );
            }
            let mut carry;
            (result.values[other.values.len() + i], carry) =
                result.values[other.values.len() + i].overflowing_add(carry0);
            (result.values[other.values.len() + i + 1], carry) =
                result.values[other.values.len() + i + 1].carrying_add(carry1, carry);
            (result.values[other.values.len() + i + 2], carry) =
                result.values[other.values.len() + i + 2].carrying_add(carry2, carry);
            // Biggest limb is only written from the carry.
            result.values[other.values.len() + i + 3] = carry3 + carry as u64;
        }

        for i in (lhs.values.len() - lhs.values.len() % 4)..lhs.values.len() {
            let mut carry = 0u64;
            for j in 0..other.values.len() {
                (result.values[j + i], carry) =
                    lhs.values[i].carrying_mul_add(other.values[j], carry, result.values[j + i]);
            }
            if carry != 0 {
                result.values[other.values.len() + i] = carry;
            }
        }

        while result.values.len() > 1 && result.values[result.values.len() - 1] == 0 {
            result.values.pop();
        }
        result
    }
}

pub struct DynBigUint<M = UnrolledMul> {
    values: Vec<u64>,
    _m: PhantomData<M>,
}

impl<M> Clone for DynBigUint<M> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            _m: Default::default(),
        }
    }
}

impl<M> FibNum for DynBigUint<M> {
    fn zero() -> Self {
        Self::new(0)
    }
    fn one() -> Self {
        Self::new(1)
    }
}

impl<M> FibNumInplace for DynBigUint<M> {}

impl<M> Display for DynBigUint<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut n = self.clone();
        let big_ten = 10u64.pow(19);
        let mut buffer: Vec<String> = Vec::new();
        let mut rem: u64;
        while !n.is_zero() {
            (n, rem) = n.div_rem(big_ten);
            if n.is_zero() {
                buffer.push(rem.to_string());
            } else {
                buffer.push(format!("{:019}", rem));
            }
        }
        for b in buffer.iter().rev() {
            write!(f, "{}", b)?;
        }
        Ok(())
    }
}

impl<M> Add for DynBigUint<M> {
    type Output = Self;

    fn add(self, other: DynBigUint<M>) -> DynBigUint<M> {
        &self + &other
    }
}

impl<M> Add for &DynBigUint<M> {
    type Output = DynBigUint<M>;

    fn add(self, other: &DynBigUint<M>) -> DynBigUint<M> {
        let mut result = DynBigUint::new_with_limbs(0, self.values.len().max(other.values.len()));
        let mut carry = false;

        let min = self.values.len().min(other.values.len());
        let a = &self.values[0..min];
        let b = &other.values[0..min];
        let r = &mut result.values[0..min];
        for (r, (a, b)) in r.iter_mut().zip(a.iter().zip(b.iter())) {
            (*r, carry) = a.carrying_add(*b, carry);
        }
        carry = add_remaining_array(self, &mut result, min, self.values.len(), carry);
        carry = add_remaining_array(other, &mut result, min, other.values.len(), carry);
        if carry {
            result.values.push(1);
        }
        result
    }
}

impl<M> Sub for DynBigUint<M> {
    type Output = Self;

    fn sub(self, other: DynBigUint<M>) -> DynBigUint<M> {
        &self - &other
    }
}

impl<M> Sub for &DynBigUint<M> {
    type Output = DynBigUint<M>;

    fn sub(self, other: &DynBigUint<M>) -> DynBigUint<M> {
        if other.values.len() > self.values.len() {
            panic!("Subtraction of DynBigUint with larger values array");
        }
        let mut result = DynBigUint::new_with_limbs(0, self.values.len());
        let mut borrow = false;

        let min = other.values.len();
        let a = &self.values[0..];
        let b = &other.values[0..];
        let r = &mut result.values[0..];
        for (r, (a, b)) in r.iter_mut().zip(a.iter().zip(b.iter())) {
            (*r, borrow) = a.borrowing_sub(*b, borrow);
        }
        if self.values.len() > min {
            let a = &self.values[min..];
            let r = &mut result.values[min..];
            for (r, a) in r.iter_mut().zip(a.iter()) {
                (*r, borrow) = a.overflowing_sub(borrow as u64);
            }
        }

        if borrow {
            panic!("Subtraction results in a negative value");
        }

        while result.values.len() > 1 && result.values[result.values.len() - 1] == 0 {
            result.values.pop();
        }

        result
    }
}

impl<M> AddAssign<&DynBigUint<M>> for DynBigUint<M> {
    fn add_assign(&mut self, other: &DynBigUint<M>) {
        let mut carry = false;

        let min = self.values.len().min(other.values.len());
        let a = &mut self.values[0..min];
        let b = &other.values[0..min];
        for (a, b) in a.iter_mut().zip(b.iter()) {
            (*a, carry) = a.carrying_add(*b, carry);
        }
        if self.values.len() > other.values.len() {
            for i in min..self.values.len() {
                (self.values[i], carry) = self.values[i].carrying_add(0u64, carry);
            }
        } else {
            for i in min..other.values.len() {
                let sum;
                (sum, carry) = other.values[i].carrying_add(0u64, carry);
                self.values.push(sum);
            }
        }
        if carry {
            self.values.push(1);
        }
    }
}

fn add_remaining_array<M>(
    a: &DynBigUint<M>,
    r: &mut DynBigUint<M>,
    min: usize,
    max: usize,
    mut carry: bool,
) -> bool {
    for i in min..max {
        (r.values[i], carry) = r.values[i].carrying_add(a.values[i], carry);
    }
    carry
}

impl<M> Mul for &DynBigUint<M>
where
    for<'a> M: MulStrategy<&'a DynBigUint<M>, DynBigUint<M>>,
{
    type Output = DynBigUint<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        M::mul(self, rhs)
    }
}

impl<M> DynBigUint<M> {
    fn new(value: u64) -> Self {
        Self {
            values: vec![value],
            _m: Default::default(),
        }
    }

    fn new_with_limbs(value: u64, limbs: usize) -> Self {
        let mut values = vec![0; limbs];
        values[0] = value;
        Self {
            values,
            _m: Default::default(),
        }
    }

    fn new_with_slice(values: &[u64]) -> Self {
        Self {
            values: values.to_vec(),
            _m: Default::default(),
        }
    }

    fn is_zero(&self) -> bool {
        self.values.iter().all(|&v| v == 0)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    fn div_rem(&self, d: u64) -> (DynBigUint<M>, u64) {
        let mut result = DynBigUint::new_with_limbs(0, self.values.len());
        let mut remainder = 0u64;
        for i in (0..self.values.len()).rev() {
            let div = ((remainder as u128) << 64) + self.values[i] as u128;
            let (a, b) = (div / d as u128, div % d as u128);
            result.values[i] = a as u64;
            remainder = b as u64;
        }
        (result, remainder)
    }

    fn add_slice_at(&mut self, a: &[u64], limb_nr: usize) {
        let r = &mut self.values[limb_nr..];
        let mut carry = false;

        for (r, a) in r.iter_mut().zip(a.iter()) {
            (*r, carry) = r.carrying_add(*a, carry);
        }
        let mut pos = a.len() + limb_nr;
        while carry {
            (self.values[pos], carry) = self.values[pos].overflowing_add(1);
            pos = pos + 1;
        }
    }

    fn copy_slice_to(&mut self, a: &[u64], limb_nr: usize) {
        let r = &mut self.values[limb_nr..];

        for (r, a) in r.iter_mut().zip(a.iter()) {
            *r= *a;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dynbiguint::{DynBigUint, Karatsuba, StandardMul};

    #[test]
    fn test_display_zero_padding() {
        // 10^19 + 1: der niedrige Chunk ist 1, muss aber als "0000000000000000001" gedruckt werden
        let a: DynBigUint = DynBigUint::new(10_000_000_000_000_000_000u64);
        let b: DynBigUint = DynBigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }

    #[test]
    fn test_subtraction() {
        // 10^19 + 1: der niedrige Chunk ist 1, muss aber als "0000000000000000001" gedruckt werden
        let a: DynBigUint = DynBigUint::new(u64::MAX - 1);
        let b: DynBigUint = DynBigUint::new(10);
        let c: DynBigUint = DynBigUint::new(12);
        let tmp = a + b;
        let d = tmp - c;
        assert_eq!(d.to_string(),(u64::MAX - 1 - 12 + 10).to_string());
    }

    #[test]
    fn test_multiplicaiton() {

        let v = [u64::MAX,u64::MAX-2,u64::MAX-3,u64::MAX-4];
        let _v2 = [1u64,2,3,4];
        let a0: DynBigUint<StandardMul> = DynBigUint::new_with_slice(&v);
        let a1: DynBigUint<Karatsuba> = DynBigUint::new_with_slice(&v);
        let b0: DynBigUint<StandardMul> = DynBigUint::new_with_slice(&v);
        let b1: DynBigUint<Karatsuba> = DynBigUint::new_with_slice(&v);
        let result0 = &a0 * &b0;
        let result1 = &a1 * &b1;
        assert_eq!(result0.values,result1.values);
    }







}
