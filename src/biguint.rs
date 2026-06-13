use std::fmt::Display;
use std::ops::Add;
use crate::fib::FibNum;

const LIMBS: usize = 4400;

#[derive(Clone)]
pub struct BigUint {
    values: [u64;LIMBS]
}

impl FibNum for BigUint {
    fn zero() -> Self {
        BigUint::new(0)
    }
    fn one() -> Self {
        BigUint::new(1)
    }
}

impl Display for BigUint {
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

impl Add for BigUint {
    type Output = Self;

    fn add(self, other: BigUint) -> BigUint {
        &self + &other
    }
}

impl Add for &BigUint {
    type Output = BigUint;

    fn add(self, other: &BigUint) -> BigUint {
        let mut result = BigUint::new(0);
        let mut carry = false;
        for i in 0..LIMBS {
            if carry {
                result.values[i] += 1;
            }
            let (sum, c1) = result.values[i].overflowing_add(self.values[i]);
            let (sum, c2) = sum.overflowing_add(other.values[i]);
            result.values[i] = sum;
            carry = c1 || c2;
        }
        result
    }
}

impl BigUint {
    const fn new(value: u64) -> Self {
        let mut values = [0; LIMBS];
        values[0] = value;
        BigUint { values }
    }

    fn is_zero(&self) -> bool {
        self.values.iter().all(|&v| v == 0)
    }

    fn div_rem(&self, d: u64) -> (BigUint, u64) {
        let mut result = BigUint::new(0);
        let mut remainder = 0u64;
        for i in (0..LIMBS).rev() {
            let div = ((remainder as u128) << 64) + self.values[i] as u128;
            let (a,b) = (div / d as u128, div % d as u128);
            result.values[i] = a as u64;
            remainder = b as u64;
        }
        (result, remainder)
    }

}

#[cfg(test)]
mod tests {
    use crate::biguint::BigUint;

    #[test]
    fn test_display_zero_padding() {
        // 10^19 + 1: der niedrige Chunk ist 1, muss aber als "0000000000000000001" gedruckt werden
        let a = BigUint::new(10_000_000_000_000_000_000u64);
        let b = BigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }
}