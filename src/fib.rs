use std::mem::swap;
use std::ops::{Add, AddAssign, Mul};

pub trait FibNum: Add<Output = Self> + Clone {
    fn zero() -> Self;
    fn one() -> Self;
}
pub trait FibNumInplace: FibNum + for<'a> AddAssign<&'a Self> {}

#[allow(dead_code)]
pub fn fib_recursive(n: u32) -> u128 {
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

pub fn matmult_2<T>(a: &(T, T)) -> (T, T)
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
    T: Add<Output = T>,
{
    let a_1_sr = &a.1 * &a.1;
    let a_0_sr = &a.0 * &a.0;
    let a_0_a_1 = &a.0 * &a.1;
    (&a_0_sr + &a_1_sr, &a_0_a_1 + &a_0_a_1 + a_1_sr)
}

pub fn matmult_advance_one<T: FibNum>(b: (T, T, T)) -> (T, T, T)
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    let r_2 = &b.1 + &b.2;
    (b.1, b.2, r_2)
}

pub fn matmult_2_advance_one<T: FibNum>(b: (T, T)) -> (T, T)
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    let t = &b.0 + &b.1;
    (b.1, t)
}

pub fn fib_matrix_mult<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
    T: Add<Output = T>,
{
    fib_advance_by_matrix_mult(n - 1, T::zero(), T::one())
}

#[allow(dead_code)]
pub fn fib_matrix_mult_2<T: FibNum>(n: u32) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
    T: Add<Output = T>,
{
    fib_advance_by_matrix_mult_fast_2(n - 1, T::zero(), T::one())
}

pub fn fib_advance_by_matrix_mult<T: FibNum>(mut n: u32, fx_minus_one: T, fx: T) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
{
    let mut a = (T::zero(), T::one(), T::one());
    let mut r = (fx_minus_one, fx);

    while n > 0 {
        if n % 2 == 1 {
            //r = A^n*r
            r = (&r.0 * &a.0 + &r.1 * &a.1, &r.0 * &a.1 + &r.1 * &a.2);
        }
        n /= 2;
        if n == 0 {
            break;
        }
        a = matmult(&a, &a);
    }

    r.1
}

#[allow(dead_code)]
pub fn fib_advance_by_matrix_mult_fast<T: FibNum>(n: u32, fx_minus_one: T, fx: T) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    if n == 0 {
        return fx;
    }
    let mut a = (T::one(), T::zero(), T::one());
    let r = (fx_minus_one, fx);

    let highest_bit = 31 - n.leading_zeros();
    let mut mask = 1u32 << highest_bit;

    loop {
        if n & mask != 0 {
            a = matmult_advance_one(a);
        }
        mask = mask >> 1;
        if mask == 0 {
            break;
        }
        a = matmult(&a, &a);
    }

    &r.0 * &a.1 + &r.1 * &a.2
}

#[allow(dead_code)]
pub fn fib_advance_by_matrix_mult_fast_2<T: FibNum>(mut n: u32, fx_minus_one: T, fx: T) -> T
where
    for<'a> &'a T: Mul<&'a T, Output = T>,
    T: Add<Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    n += 1;
    if n == 0 {
        return fx;
    }
    let mut a = (T::one(), T::zero());
    let r = (fx_minus_one, fx);

    let highest_bit = 31 - n.leading_zeros();
    let mut mask = 1u32 << highest_bit;

    loop {
        if n & mask != 0 {
            a = matmult_2_advance_one(a);
        }
        mask = mask >> 1;
        if mask == 0 {
            break;
        }
        a = matmult_2(&a);
    }
    //a.1
    &r.0 * &a.0 + &r.1 * &(a.1)
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
    use crate::dynbiguint::{DynBigUint, FFT, Karatsuba, UnrolledMul};
    use crate::fib::{FibNum, fib_advance_by_matrix_mult, fib_advance_by_matrix_mult_fast_2, fib_inplace_two_values, fib_matrix_mult, fib_two_values, fib_matrix_mult_2};
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
            fib_advance_by_matrix_mult_fast_2(
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
        let n = 510_000;
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
    fn test_matmult_intense() {
        let now = std::time::Instant::now();
        let mut x = fib_matrix_mult::<DynBigUint>(10_000_000);
        for n in (100_000..1_000_000).step_by(11) {

            let y = fib_matrix_mult::<DynBigUint>(n);
            x = x + y;
        }
        println!(
            "fib_matrix_mult({}) took\t {:15} ns",
            x.len(),
            now.elapsed().as_nanos()
        );
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
    fn test_kat_fib() {
        let mut n = 1_000_00;
        while n < 2_000_000 {
            let now = std::time::Instant::now();
            let x = fib_matrix_mult_2::<DynBigUint<UnrolledMul>>(n);
            println!(
                "standard: fib_matrix_mult({}) took\t {:15} ms",
                n,
                now.elapsed().as_millis()
            );

            let now = std::time::Instant::now();
            let y = fib_matrix_mult_2::<DynBigUint<Karatsuba>>(n);
            println!(
                "karatuba: fib_matrix_mult({}) took\t {:15} ms",
                n,
                now.elapsed().as_millis()
            );
            n = n << 1;// - n >> 1;
            assert_eq!(x.to_string(), y.to_string());
        }
    }

    #[test]
    fn test_optbiguint_fib_700000() {
        let now = std::time::Instant::now();
        let x = fib_two_values::<OptBigUint>(700_000);
        let y = fib_two_values::<DynBigUint>(700_000);
        assert_eq!(x.to_string(), y.to_string());
    }

    #[test]
    #[ignore = "finds FFT f64 precision boundary with real Fibonacci numbers (hours)"]
    fn test_fib_fft_precision_boundary() {
        use std::time::Instant;

        fn compute_both(n: u32) -> (bool, usize, std::time::Duration, std::time::Duration) {
            let t0 = Instant::now();
            let kara = fib_matrix_mult_2::<DynBigUint<Karatsuba>>(n);
            let t_k = t0.elapsed();
            let t1 = Instant::now();
            let fft = fib_matrix_mult_2::<DynBigUint<FFT>>(n);
            let t_f = t1.elapsed();
            (kara.limbs() == fft.limbs(), kara.limbs().len(), t_k, t_f)
        }

        fn print_row(n: u32, limbs: usize, t_k: std::time::Duration, t_f: std::time::Duration, ok: bool) {
            println!(
                "{:>12} | {:>10} | {:>12.1} | {:>12.1} | {}",
                n, limbs,
                t_k.as_secs_f64() * 1000.0,
                t_f.as_secs_f64() * 1000.0,
                if ok { "OK  " } else { "FAIL" }
            );
        }

        fn print_header() {
            println!(
                "{:>12} | {:>10} | {:>12} | {:>12} | {}",
                "fib(n)", "limbs", "kara (ms)", "fft  (ms)", "fft ok?"
            );
            println!("{}", "-".repeat(65));
        }

        println!();
        print_header();

        // Phase 1: exponential search; lo is always a verified-OK index
        let mut lo = 0u32;
        let mut hi = 0u32;
        let mut n = 1_000_000u32;
        let mut failure_found = false;

        loop {
            let (ok, limbs, t_k, t_f) = compute_both(n);
            print_row(n, limbs, t_k, t_f, ok);

            if !ok {
                hi = n;
                failure_found = true;
                break;
            }
            lo = n;
            if n >= 200_000_000 {
                break;
            }
            n *= 2;
        }

        if !failure_found {
            println!("\nNo precision failure found up to fib({n}).");
            return;
        }

        // Phase 2: bisection inside (lo, hi)
        println!("\nBisection [fib({lo}), fib({hi})]:");
        print_header();

        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let (ok, limbs, t_k, t_f) = compute_both(mid);
            print_row(mid, limbs, t_k, t_f, ok);
            if ok { lo = mid; } else { hi = mid; }
        }

        println!("\nPrecision boundary:  fib({lo}) OK,  fib({hi}) FAIL");
    }

    #[test]
    #[ignore = "long-running benchmark (~minutes)"]
    fn test_fib_10m_karatsuba_vs_fft() {
        //const N: u32 = 6_041_564;
        const N: u32 = 20_000_000;

        let now = std::time::Instant::now();
        let kara = fib_matrix_mult_2::<DynBigUint<Karatsuba>>(N);
        let kara_ms = now.elapsed().as_millis();
        println!("Karatsuba: fib({N}) took {kara_ms:>8} ms  ({} limbs)", kara.limbs().len());

        let now = std::time::Instant::now();
        let fft = fib_matrix_mult_2::<DynBigUint<FFT>>(N);
        let fft_ms = now.elapsed().as_millis();
        println!("FFT:       fib({N}) took {fft_ms:>8} ms  ({} limbs)", fft.limbs().len());

        assert_eq!(
            kara.limbs(),
            fft.limbs(),
            "Karatsuba and FFT produced different results for fib({N})"
        );
    }
}
