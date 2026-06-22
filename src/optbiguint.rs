use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::ops::{Add, AddAssign};

const LIMBS: usize = 7600;

#[derive(Clone)]
pub struct OptBigUint {
    values: [u64; LIMBS],
    limbs: usize,
}

impl FibNum for OptBigUint {
    fn zero() -> Self {
        OptBigUint::new(0)
    }
    fn one() -> Self {
        OptBigUint::new(1)
    }
}

impl FibNumInplace for OptBigUint {}

impl Display for OptBigUint {
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

impl Add for OptBigUint {
    type Output = Self;

    fn add(self, other: OptBigUint) -> OptBigUint {
        &self + &other
    }
}

impl Add for &OptBigUint {
    type Output = OptBigUint;

    fn add(self, other: &OptBigUint) -> OptBigUint {
        let max = self.limbs.max(other.limbs);
        let limbs = if self.check_if_overflows(other) {
            max + 1
        } else {
            max
        };
        let mut result = OptBigUint::new(0);
        result.limbs = limbs;
        let mut carry = false;
        for i in 0..LIMBS.min(limbs) {
            (result.values[i], carry) = self.values[i].carrying_add(other.values[i], carry);
        }
        result
    }
}

impl AddAssign<&OptBigUint> for OptBigUint {
    fn add_assign(&mut self, other: &OptBigUint) {
        let mut carry = false;
        for i in 0..LIMBS.min(self.limbs.min(other.limbs)) {
            (self.values[i], carry) = self.values[i].carrying_add(other.values[i], carry);
        }
        if self.limbs > other.limbs {
            for i in self.limbs.min(other.limbs)..self.limbs {
                (self.values[i], carry) = self.values[i].carrying_add(0u64, carry);
            }
        } else {
            for i in self.limbs.min(other.limbs)..other.limbs {
                (self.values[i], carry) = other.values[i].carrying_add(0u64, carry);
                self.limbs += 1;
            }
        }

        if carry {
            self.values[self.limbs] = 1;
            self.limbs += 1;
        }
    }
}

impl OptBigUint {
    const fn new(value: u64) -> Self {
        let limbs = if value == 0 { 0 } else { 1 };
        let mut values = [0; LIMBS];
        values[0] = value;
        OptBigUint { values, limbs }
    }

    fn is_zero(&self) -> bool {
        self.values.iter().all(|&v| v == 0)
    }

    fn div_rem(&self, d: u64) -> (OptBigUint, u64) {
        let mut result = OptBigUint::new(0);
        result.limbs = self.limbs;
        let mut remainder = 0u64;
        for i in (0..self.limbs).rev() {
            let div = ((remainder as u128) << 64) + self.values[i] as u128;
            let (a, b) = (div / d as u128, div % d as u128);
            result.values[i] = a as u64;
            remainder = b as u64;
        }
        (result, remainder)
    }

    fn check_if_overflows(&self, other: &OptBigUint) -> bool {
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
}

#[cfg(test)]
mod tests {
    use crate::optbiguint::OptBigUint;

    #[test]
    fn test_display_zero_padding() {
        // 10^19 + 1: the low chunk is 1, but must be printed as "0000000000000000001"
        let a = OptBigUint::new(10_000_000_000_000_000_000u64);
        let b = OptBigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }
}
