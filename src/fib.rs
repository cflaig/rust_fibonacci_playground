use std::mem::swap;
use std::ops::{Add, AddAssign, Mul};

pub trait FibNum: Add<Output = Self> + Clone {
    fn zero() -> Self;
    fn one() -> Self;
}
pub trait FibNumInplace: FibNum + for<'a> AddAssign<&'a Self> {}

#[allow(dead_code)]
fn fib_recursive(n: u32) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        n => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}

#[allow(dead_code)]
pub fn fib_recursive_cache<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    if n < 3 {
        return T::one();
    }
    let mut cache: Vec<Option<T>> = vec![None; (n + 1) as usize];
    cache[1] = Some(T::one());
    cache[2] = Some(T::one());

    fn compute<T: FibNum>(n: u32, cache: &mut Vec<Option<T>>) -> T
    where
        for<'a> &'a T: Add<&'a T, Output = T>,
    {
        if let Some(val) = &cache[n as usize] {
            return val.clone();
        }
        let value = &compute(n - 2, cache) + &compute(n - 1, cache);
        cache[n as usize] = Some(value.clone());
        value
    }
    compute(n, &mut cache)
}

#[allow(dead_code)]
pub fn fibu_dynamic_programming<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    if n < 3 {
        return T::one();
    }
    let mut fibs = vec![T::zero(); (n + 1) as usize];
    fibs[1] = T::one();
    fibs[2] = T::one();
    for i in 3..=n as usize {
        fibs[i] = &fibs[i - 1] + &fibs[i - 2];
    }
    fibs[n as usize].clone()
}

pub fn fib_two_values<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    let mut a = T::one();
    let mut b = T::one();
    for _ in 3..=n {
        let tmp = &a + &b;
        a = b;
        b = tmp;
    }
    b
}

pub fn fib_inplace_two_values<T: FibNumInplace>(n: u32) -> T {
    let mut a = Box::new(T::one());
    let mut b = Box::new(T::one());

    for _ in 3..=n {
        (&mut *a).add_assign(&b);
        swap(&mut b, &mut a);
    }
    *b
}

pub fn matmult<T>(a: &(T, T, T), b: &(T, T, T)) -> (T, T, T)
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
{
    (
        &a.0 * &b.0 + &a.1 * &b.1,
        &a.0 * &b.1 + &a.1 * &b.2,
        &a.1 * &b.1 + &a.2 * &b.2,
    )
}

pub fn fib_matrix_mult<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
{
    fib_advance_by_matrix_mult(n - 1, T::zero(), T::one())
}
pub fn fib_advance_by_matrix_mult<T: FibNum>(mut n: u32, fx_minus_one: T, fx: T) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
{
    let mut a = (T::zero(), T::one(), T::one());
    let mut r = (T::one(), T::zero(), T::one());

    while n > 0 {
        if n % 2 == 1 {
            r = matmult(&r, &a);
        }
        n /= 2;
        if n == 0 {
            break;
        }
        a = matmult(&a, &a);
    }

    &fx_minus_one * &r.1 + &fx * &r.2
}

impl FibNum for u64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }
}

#[cfg(test)]
mod tests {
    use crate::biguint::BigUint;
    use crate::dynbiguint::DynBigUint;
    use crate::fib::FibNum;
    use crate::fib::{
        fib_advance_by_matrix_mult, fib_inplace_two_values, fib_matrix_mult, fib_two_values,
    };
    use crate::optbiguint::OptBigUint;

    #[test]
    fn test_matmult() {
        for i in 1..20 {
            println!("fib({}) = {}", i, fib_matrix_mult::<DynBigUint>(i));
            //assert_eq!(fib_matrix_mult::<BigUint>(i).to_string, fib_recursive(i));
        }
        assert_eq!(
            fib_advance_by_matrix_mult::<DynBigUint>(5, DynBigUint::zero(), DynBigUint::one())
                .to_string(),
            fib_two_values::<DynBigUint>(6).to_string()
        );
        assert_eq!(
            fib_advance_by_matrix_mult(
                5,
                fib_two_values::<DynBigUint>(4),
                fib_two_values::<DynBigUint>(5)
            )
            .to_string(),
            fib_two_values::<DynBigUint>(10).to_string()
        );
    }

    #[test]
    fn test_matmult2() {
        for n in 1..1015 {
            let now = std::time::Instant::now();
            let x = fib_two_values::<DynBigUint>(n);
            println!(
                "fib_two_values({})  took\t {:15} ns",
                n,
                now.elapsed().as_nanos()
            );

            let now = std::time::Instant::now();
            let y = fib_matrix_mult::<DynBigUint>(n);
            println!(
                "fib_matrix_mult({}) took\t {:15} ns",
                n,
                now.elapsed().as_nanos()
            );

            assert_eq!(x.to_string(), y.to_string());
        }

        let n = 1_000_000;
        let n = 10_000;
        let now = std::time::Instant::now();
        let x = fib_two_values::<DynBigUint>(n);
        println!(
            "fib_two_values({})  took\t {:15} ms",
            n,
            now.elapsed().as_millis()
        );

        let now = std::time::Instant::now();
        let y = fib_matrix_mult::<DynBigUint>(n);
        println!(
            "fib_matrix_mult({}) took\t {:15} ms",
            n,
            now.elapsed().as_millis()
        );

        assert_eq!(x.to_string(), y.to_string());
    }

    #[test]
    fn test_big_uint_dynuint() {
        for i in 1..1000 {
            let x = fib_two_values::<BigUint>(i);
            let y = fib_two_values::<DynBigUint>(i);
            assert_eq!(x.to_string(), y.to_string());
        }
    }

    #[test]
    fn test_big_uint_optbiguint() {
        for i in 1..1000 {
            let x = fib_two_values::<BigUint>(i);
            let y = fib_two_values::<OptBigUint>(i);
            assert_eq!(x.to_string(), y.to_string());
        }
    }

    #[test]
    fn test_fib_inplace() {
        for i in 1..1000 {
            let x = fib_two_values::<OptBigUint>(i);
            let y = fib_inplace_two_values::<DynBigUint>(i);
            assert_eq!(x.to_string(), y.to_string());
        }
    }

    #[test]
    fn test_fib_inplace_optbigunit() {
        for i in 1..1000 {
            let x = fib_inplace_two_values::<OptBigUint>(i);
            let y = fib_inplace_two_values::<DynBigUint>(i);
            assert_eq!(x.to_string(), y.to_string());
        }
    }

    #[test]
    fn test_biguint_fib_700000() {
        let x = fib_two_values::<BigUint>(700_000);
        let y = fib_two_values::<DynBigUint>(700_000);
        assert_eq!(x.to_string(), y.to_string());
    }

    #[test]
    fn test_optbiguint_fib_700000() {
        let x = fib_two_values::<OptBigUint>(700_000);
        let y = fib_two_values::<DynBigUint>(700_000);
        assert_eq!(x.to_string(), y.to_string());
    }
}
