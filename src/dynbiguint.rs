use crate::fib::{FibNum, FibNumInplace};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};


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

pub trait MulStrategy<T, O> {
    fn mul(lhs: T, rhs: T) -> O;
}

pub struct StandardMul;
pub struct UnrolledMul;
pub struct Karatsuba;
pub struct FFT;

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

impl MulAssign<f64> for Complex<f64> {
    fn mul_assign(&mut self, rhs: f64) {
        self.re *= rhs;
        self.im *= rhs;
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
            let f_odd_evaluated = &w_i * &f_odd[i];
            input[i] = f_even[i].clone() + f_odd_evaluated.clone();
            input[i+n] = f_even[i].clone() - f_odd_evaluated;
            w_i = &w_i * &w;
            let correction = 0.5 * (3.0 - w_i.re*w_i.re - w_i.im*w_i.im);
            w_i *= correction;
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

    pub fn limbs(&self) -> &[u64] {
        &self.values
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
    use crate::dynbiguint::{DynBigUint, FFT, Karatsuba, StandardMul, UnrolledMul};
    use std::marker::PhantomData;

    #[test]
    fn test_display_zero_padding() {
        // 10^19 + 1: the low chunk is 1, but must be printed as "0000000000000000001"
        let a: DynBigUint = DynBigUint::new(10_000_000_000_000_000_000u64);
        let b: DynBigUint = DynBigUint::new(1);
        let result = (a + b).to_string();
        assert_eq!(result, "10000000000000000001");
    }

    #[test]
    fn test_subtraction() {
        // 10^19 + 1: the low chunk is 1, but must be printed as "0000000000000000001"
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
    #[test]
    fn test_random_mul_consistency() {
        use rand::Rng;

        let mut rng = rand::rng();

        for i in 0..10000u32 {
            let len_a = rng.random_range(64usize..=512);
            let len_b = rng.random_range(64usize..=512);
            let a_vals: Vec<u64> = (0..len_a).map(|_| rng.random::<u64>()).collect();
            let b_vals: Vec<u64> = (0..len_b).map(|_| rng.random::<u64>()).collect();

            let mk_unrolled = |v: &Vec<u64>| -> DynBigUint<UnrolledMul> {
                DynBigUint { values: v.clone(), _m: PhantomData }
            };
            let mk_kara = |v: &Vec<u64>| -> DynBigUint<Karatsuba> {
                DynBigUint { values: v.clone(), _m: PhantomData }
            };
            let mk_fft = |v: &Vec<u64>| -> DynBigUint<FFT> {
                DynBigUint { values: v.clone(), _m: PhantomData }
            };

            let expected  = &mk_unrolled(&a_vals) * &mk_unrolled(&b_vals);
            let got_kara  = &mk_kara(&a_vals)     * &mk_kara(&b_vals);
            let got_fft   = &mk_fft(&a_vals)      * &mk_fft(&b_vals);

            assert_eq!(
                expected.values, got_kara.values,
                "Karatsuba mismatch at iteration {i} (len_a={len_a}, len_b={len_b})"
            );
            assert_eq!(
                expected.values, got_fft.values,
                "FFT mismatch at iteration {i} (len_a={len_a}, len_b={len_b})"
            );
        }
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

    #[test]
    #[ignore = "finds FFT f64 precision boundary via exponential search + bisection (minutes)"]
    fn test_fft_precision_boundary() {
        use std::marker::PhantomData;
        use std::time::Instant;

        // Worst-case input: all bytes 0xFF maximises every convolution coefficient,
        // so this is the earliest possible precision failure point.
        fn mul_both(n: usize) -> (bool, std::time::Duration, std::time::Duration) {
            let vals: Vec<u64> = vec![u64::MAX; n];
            let a_k: DynBigUint<Karatsuba> = DynBigUint { values: vals.clone(), _m: PhantomData };
            let b_k: DynBigUint<Karatsuba> = DynBigUint { values: vals.clone(), _m: PhantomData };
            let a_f: DynBigUint<FFT>       = DynBigUint { values: vals.clone(), _m: PhantomData };
            let b_f: DynBigUint<FFT>       = DynBigUint { values: vals.clone(), _m: PhantomData };

            let t0 = Instant::now();
            let r_k = &a_k * &b_k;
            let t_k = t0.elapsed();

            let t1 = Instant::now();
            let r_f = &a_f * &b_f;
            let t_f = t1.elapsed();

            (r_k.values == r_f.values, t_k, t_f)
        }

        let log2_phi = ((1.0 + 5.0f64.sqrt()) / 2.0).log2();
        let fib_n = |limbs: usize| -> u64 { (limbs as f64 * 64.0 / log2_phi) as u64 };

        fn print_row(limbs: usize, fib_n: u64, t_k: std::time::Duration, t_f: std::time::Duration, ok: bool) {
            println!(
                "{:>10} | {:>14} | {:>12.3} | {:>12.3} | {}",
                limbs,
                fib_n,
                t_k.as_secs_f64() * 1000.0,
                t_f.as_secs_f64() * 1000.0,
                if ok { "OK  " } else { "FAIL" }
            );
        }

        println!(
            "\n{:>10} | {:>14} | {:>12} | {:>12} | {}",
            "limbs", "≈ fib(n)", "kara (ms)", "fft  (ms)", "fft ok?"
        );
        println!("{}", "-".repeat(65));

        // Phase 1: exponential search for the failure bracket [lo, hi]
        let (mut lo, mut hi) = (1usize, 1usize);
        let mut failure_found = false;
        let mut size = 1usize;

        loop {
            let (ok, t_k, t_f) = mul_both(size);
            print_row(size, fib_n(size), t_k, t_f, ok);

            if !ok {
                hi = size;
                lo = size / 2;
                failure_found = true;
                break;
            }
            if size >= 1 << 20 {
                break; // 2^20 limbs = 64 Mbit safety cap
            }
            size *= 2;
        }

        if !failure_found {
            println!("\nNo precision failure found up to {} limbs ({} bits).", size, size * 64);
            return;
        }

        // Phase 2: bisection inside [lo, hi]
        println!("\nBisection [{lo}, {hi}]:");
        println!(
            "{:>10} | {:>14} | {:>12} | {:>12} | {}",
            "limbs", "≈ fib(n)", "kara (ms)", "fft  (ms)", "fft ok?"
        );
        println!("{}", "-".repeat(65));

        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            let (ok, t_k, t_f) = mul_both(mid);
            print_row(mid, fib_n(mid), t_k, t_f, ok);
            if ok { lo = mid; } else { hi = mid; }
        }

        println!(
            "\nPrecision boundary:\n  last OK  : {lo} limbs ≈ fib({}) ({} bits ≈ {:.0} decimal digits)",
            fib_n(lo),
            lo * 64,
            lo as f64 * 64.0 * std::f64::consts::LOG10_2,
        );
        println!(
            "  first FAIL: {hi} limbs ≈ fib({}) ({} bits ≈ {:.0} decimal digits)",
            fib_n(hi),
            hi * 64,
            hi as f64 * 64.0 * std::f64::consts::LOG10_2,
        );
    }
}
