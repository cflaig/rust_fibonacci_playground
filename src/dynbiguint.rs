use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul};

pub trait MulStrategy<T, O> {
    fn mul(lhs: T, rhs: T) -> O;
}

pub struct StandardMul;
pub struct UnrolledMul;

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
}

#[cfg(test)]
mod tests {
    use crate::dynbiguint::DynBigUint;

    #[test]
    fn test_display_zero_padding() {
        // 10^19 + 1: der niedrige Chunk ist 1, muss aber als "0000000000000000001" gedruckt werden
        let a: DynBigUint = DynBigUint::new(10_000_000_000_000_000_000u64);
        let b: DynBigUint = DynBigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }
}
