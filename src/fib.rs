use std::ops::Add;

pub trait FibNum: Add<Output = Self> + Clone {
    fn zero() -> Self;
    fn one() -> Self;
}

fn fib_recursive(n: u32) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        n => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}

pub fn fib_recursive_cache<T: FibNum>(n: u32) -> T
where
        for<'a> &'a T: Add<&'a T, Output = T> {
    if n < 3 {
        return T::one();
    }
    let mut cache: Vec<Option<T>> = vec![None; (n + 1) as usize];
    cache[1] = Some(T::one());
    cache[2] = Some(T::one());

    fn compute<T: FibNum>(n: u32, cache: &mut Vec<Option<T>>) -> T
    where
            for<'a> &'a T: Add<&'a T, Output = T> {
        if let Some(val) = &cache[n as usize] {
            return val.clone();
        }
        let value = &compute(n - 2, cache) + &compute(n - 1, cache);
            cache[n as usize] = Some(value.clone());
            value
    }
    compute(n, &mut cache)
}

pub fn fibu_dynamic_programming<T: FibNum>(n: u32) -> T
where
        for<'a> &'a T: Add<&'a T, Output = T> {
    if n < 3 {
        return T::one();
    }
    let mut fibs = vec![T::zero(); (n + 1) as usize];
    fibs[1] = T::one();
    fibs[2] = T::one();
    for i in 3..=n as usize {
        fibs[i] = &fibs[i-1] + &fibs[i-2];
    }
    fibs[n as usize].clone()
}

pub fn fib_two_values<T: FibNum>(n: u32) -> T
where
        for<'a> &'a T: Add<&'a T, Output = T> {
    let mut a = T::one();
    let mut b = T::one();
    for _ in 3..=n {
        let tmp = &a + &b;
        a = b;
        b = tmp;
    }
    b
}

#[cfg(test)]
mod tests {
    use crate::biguint::BigUint;
    use crate::dynbiguint::DynBigUint;
    use crate::fib::fib_two_values;
    use crate::optbiguint::OptBigUint;

    #[test]
    fn test_big_uint_dynuint() {
        for i in 1..9000 {
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
}