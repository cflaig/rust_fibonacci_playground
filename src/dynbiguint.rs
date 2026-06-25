use std::cmp::max;
use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::fs::ReadDir;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, Sub};

pub trait MulStrategy<T, O> {
    fn mul(lhs: T, rhs: T) -> O;
}

pub struct StandardMul;
pub struct UnrolledMul;

pub struct FFT;

#[derive(Clone)]
struct Complex<T> {
    re: T,
    im: T,
}

impl<T: num_traits::float::Float> Complex<T> {
    fn from_polar(r: T, phi: T) -> Complex<T> {
        Complex { re: r * phi.cos(), im: r * phi.sin() }
    }
}

impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;

    fn add(self, other: Complex<T>) -> Complex<T> {

        Complex { re: self.re + other.re, im: self.im + other.im}
    }
}

impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;

    fn sub(self, other: Complex<T>) -> Complex<T> {

        Complex { re: self.re - other.re, im: self.im - other.im}
    }
}

impl<T: Copy + Mul<Output = T> + Sub<Output = T> + Add<Output = T>> Mul for Complex<T> {
    type Output = Self;

    fn mul(self, other: Complex<T>) -> Complex<T> {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.im * other.re + self.re * other.im
        }
    }
}

impl<T: Copy + Mul<Output = T> + Sub<Output = T> + Add<Output = T>> Mul for &Complex<T> {
    type Output = Complex<T>;

    fn mul(self, other: &Complex<T>) -> Complex<T> {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.im * other.re + self.re * other.im
        }
    }
}

impl<M> MulStrategy<&DynBigUint<M>, DynBigUint<M>> for FFT {
    fn mul(lhs: &DynBigUint<M>, other: &DynBigUint<M>) -> DynBigUint<M> {
        let number_terms = (8 * (lhs.values.len() + other.values.len()) -1).next_power_of_two();

        let polynom_x = Self::generate_polynom(lhs, number_terms);
        let polynom_y = Self::generate_polynom(other, number_terms);

        let uni = Complex::from_polar(1.0, 2.0 * std::f64::consts::PI / number_terms as f64);
        let polynom_x_frequency = Self::fft(polynom_x, uni.clone());
        let polynom_y_frequency = Self::fft(polynom_y, uni.clone());

        let mut result_polynom_f = vec![Complex { re: 0.0f64, im: 0.0 }; number_terms];
        for i in 0..number_terms {
            result_polynom_f[i] = &polynom_x_frequency[i] * &polynom_y_frequency[i];
        }
        let result_polynom = Self::ifft(result_polynom_f, uni);

        let mut result = DynBigUint::new_with_limbs(0, lhs.values.len() + other.values.len() + 1);

        let mut carry = 0u128;
        for i in 0..(lhs.values.len() + other.values.len()) {
            result.values[i] = carry as u64;
            carry = 0;
            for j in 0..8 {
                let coeff = result_polynom[i * 8 + j].re.round() as u128;
                let coeff = coeff << 8 * j;
                let local_carry;
                let low = coeff as u64;
                let high = coeff >> 64;
                (result.values[i], local_carry) = result.values[i].overflowing_add(low);
                carry += high + local_carry as u128;
            }
        }

        while result.values.len() > 1 && result.values[result.values.len() - 1] == 0 {
            result.values.pop();
        }

        result
    }
}

impl FFT {
    fn generate_polynom<M>(lhs: &DynBigUint<M>, number_terms: usize) -> Vec<Complex<f64>> {
        let mut polynom_x = vec![Complex { re: 0.0f64, im: 0.0 }; number_terms];
        for (i, v) in lhs.values.iter().enumerate() {
            let mask = 255u64;
            let mut value = *v;
            for j in 0..8 {
                polynom_x[i * 8 + j].re = (value & mask) as f64;
                value >>= 8;
            }
        }
        polynom_x
    }
    fn fft(mut input: Vec<Complex<f64>>, w: Complex<f64>) ->Vec<Complex<f64>> {
        if input.len() == 1 {
            return input;
        }
        let mut f_odd = Vec::new();
        let mut f_even = Vec::new();
        for i in 0..input.len() {
            if ( i % 2 == 0 ) {
                f_even.push(input[i].clone());
            } else {
                f_odd.push(input[i].clone());
            }
        }
        let new_w = &w * &w;
        let f_odd = FFT::fft(f_odd, new_w.clone());
        let f_even = FFT::fft(f_even, new_w);
        let n = f_even.len();
        let mut w_i = Complex { re:1.0f64, im:0.0f64};
        for i in 0..f_even.len() {
            input[i] = f_even[i].clone() + &w_i * &f_odd[i];
            input[i+n] = f_even[i].clone() - &w_i * &f_odd[i];
            w_i = &w_i * &w;
        }
        input
    }
    fn ifft(input: Vec<Complex<f64>>, w: Complex<f64>) ->Vec<Complex<f64>> {
        let w_conj = Complex {re: w.re, im:-w.im};
        let mut result = Self::fft(input, w_conj);
        let n = result.len();
        let factor = Complex::from_polar(1.0f64 / n as f64, 0.0);
        for i in 0..n {
            result[i] = &result[i] * &factor;;
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

    #[test]
    fn test_fft_mult() {
        use crate::dynbiguint::{FFT, StandardMul};

        fn check(a: DynBigUint<FFT>, b: DynBigUint<FFT>, label: &str) {
            let fft_result = (&a * &b).to_string();
            // rebuild same values under StandardMul to get ground truth
            let a_std: DynBigUint<StandardMul> = DynBigUint {
                values: a.values.clone(),
                _m: std::marker::PhantomData,
            };
            let b_std: DynBigUint<StandardMul> = DynBigUint {
                values: b.values.clone(),
                _m: std::marker::PhantomData,
            };
            let expected = (&a_std * &b_std).to_string();
            assert_eq!(fft_result, expected, "FFT mismatch: {label}");
        }

        // single-limb cases
        check(DynBigUint::new(0), DynBigUint::new(42), "0 * 42");
        check(DynBigUint::new(1), DynBigUint::new(1), "1 * 1");
        check(DynBigUint::new(6), DynBigUint::new(7), "6 * 7");
        check(DynBigUint::new(256), DynBigUint::new(2), "256 * 2");

        check(DynBigUint::new(u32::MAX as u64), DynBigUint::new(u32::MAX as u64), "u32::MAX^2");
        check(DynBigUint::new(u64::MAX), DynBigUint::new(u64::MAX), "u64::MAX^2");

        // 2-limb inputs: build 2^64 via u64::MAX + 1
        let two64: DynBigUint<FFT> = DynBigUint::new(u64::MAX) + DynBigUint::new(1);
        check(two64.clone(), two64.clone(), "2^64 * 2^64");
        check(two64.clone(), DynBigUint::new(u64::MAX), "2^64 * u64::MAX");

        // 2-limb × 2-limb with non-trivial low limb: (2^64 + u64::MAX) * (2^64 + 1)
        let a2: DynBigUint<FFT> = two64.clone() + DynBigUint::new(u64::MAX);
        let b2: DynBigUint<FFT> = two64.clone() + DynBigUint::new(1);
        check(a2, b2, "(2^64 + u64::MAX) * (2^64 + 1)");

        // 4-limb inputs: square the 2-limb result (u64::MAX^2 has 2 limbs, square that)
        let big: DynBigUint<FFT> = DynBigUint::new(u64::MAX);
        let big2 = &big * &big; // 2-limb
        check(big2.clone(), big2.clone(), "u64::MAX^4 (4-limb inputs)");
    }
}
