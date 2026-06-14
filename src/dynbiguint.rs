use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul};

#[derive(Clone)]
pub struct DynBigUint {
    values: Vec<u64>,
    limbs: usize,
}

impl FibNum for DynBigUint {
    fn zero() -> Self {
        DynBigUint::new(0)
    }
    fn one() -> Self {
        DynBigUint::new(1)
    }
}

impl FibNumInplace for DynBigUint {}

impl Display for DynBigUint {
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

impl Add for DynBigUint {
    type Output = Self;

    fn add(self, other: DynBigUint) -> DynBigUint {
        &self + &other
    }
}

impl Add for &DynBigUint {
    type Output = DynBigUint;

    fn add(self, other: &DynBigUint) -> DynBigUint {
        let max = self.limbs.max(other.limbs);
        let limbs = if self.check_if_overflows(other) {
            max + 1
        } else {
            max
        };

        let mut result = DynBigUint::new_with_limbs(0, limbs);
        let mut carry = false;

        let min = self.limbs.min(other.limbs);
        let a = &self.values[0..min];
        let b = &other.values[0..min];
        let r = &mut result.values[0..min];
        for (r, (a, b)) in r.iter_mut().zip(a.iter().zip(b.iter())) {
            (*r, carry) = a.carrying_add(*b, carry);
        }
        carry = add_remaining_array(
            self,
            &mut result,
            self.limbs.min(other.limbs),
            self.limbs,
            carry,
        );
        carry = add_remaining_array(
            other,
            &mut result,
            self.limbs.min(other.limbs),
            other.limbs,
            carry,
        );
        if carry {
            result.values[result.limbs - 1] += 1;
        }
        result
    }
}

impl AddAssign<&DynBigUint> for DynBigUint {
    fn add_assign(&mut self, other: &DynBigUint) {
        let mut carry = false;

        let min = self.limbs.min(other.limbs);
        let a = &mut self.values[0..min];
        let b = &other.values[0..min];
        for (a, b) in a.iter_mut().zip(b.iter()) {
            (*a, carry) = a.carrying_add(*b, carry);
        }
        if self.limbs > other.limbs {
            for i in self.limbs.min(other.limbs)..self.limbs {
                (self.values[i], carry) = self.values[i].carrying_add(0u64, carry);
            }
        } else {
            for i in self.limbs.min(other.limbs)..other.limbs {
                let sum;
                (sum, carry) = other.values[i].carrying_add(0u64, carry);
                self.values.push(sum);
                self.limbs += 1;
            }
        }
        if carry {
            self.values.push(1);
            self.limbs += 1;
        }
    }
}

fn add_remaining_array(
    a: &DynBigUint,
    r: &mut DynBigUint,
    min: usize,
    max: usize,
    mut carry: bool,
) -> bool {
    for i in min..max {
        (r.values[i], carry) = r.values[i].carrying_add(a.values[i], carry);
    }
    carry
}

impl Mul for &DynBigUint {
    type Output = DynBigUint;

    fn mul(self, other: &DynBigUint) -> DynBigUint {
        let mut result = DynBigUint::new_with_limbs(0, self.limbs + other.limbs);
        for i in 0..self.limbs {
            let mut carry = 0u64;
            for j in 0..other.limbs {
                (result.values[j + i], carry) =
                    self.values[i].carrying_mul_add(other.values[j], carry, result.values[j + i]);
            }
            if carry != 0 {
                result.values[other.limbs + i] = carry;
            }
        }

        while result.limbs > 1 && result.values[result.limbs - 1] == 0 {
            result.values.pop();
            result.limbs -= 1;
        }
        result
    }
}

impl DynBigUint {
    fn new(value: u64) -> Self {
        let mut values = vec![0; 1];
        values[0] = value;
        DynBigUint { values, limbs: 1 }
    }

    fn new_with_limbs(value: u64, limbs: usize) -> Self {
        let mut values = vec![0; limbs];
        values[0] = value;
        DynBigUint { values, limbs }
    }

    fn is_zero(&self) -> bool {
        self.values.iter().all(|&v| v == 0)
    }

    fn check_if_overflows(&self, other: &DynBigUint) -> bool {
        let (sum, carry) = match self.limbs.cmp(&other.limbs) {
            std::cmp::Ordering::Greater => (self.values[self.limbs - 1], false),
            std::cmp::Ordering::Less => (other.values[other.limbs - 1], false),
            std::cmp::Ordering::Equal => {
                self.values[self.limbs - 1].overflowing_add(other.values[other.limbs - 1])
            }
        };
        let (_, carry2) = sum.overflowing_add(1u64);
        carry || carry2
    }

    fn div_rem(&self, d: u64) -> (DynBigUint, u64) {
        let mut result = DynBigUint::new_with_limbs(0, self.limbs);
        let mut remainder = 0u64;
        for i in (0..self.limbs).rev() {
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
        let a = DynBigUint::new(10_000_000_000_000_000_000u64);
        let b = DynBigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }
}
