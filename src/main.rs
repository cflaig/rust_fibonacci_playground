use plotters::prelude::*;
use std::error::Error;
use std::time::Instant;
use biguint::BigUint;
use dynbiguint::DynBigUint;
use optbiguint::OptBigUint;

mod biguint;
mod fib;
mod dynbiguint;
mod optbiguint;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(run)?
        .join()
        .unwrap()
}

fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut two_values_timings = Vec::new();
    let mut dyn_two_values_timings = Vec::new();
    let mut opt_two_values_timings = Vec::new();

    for n in (1..=400_000).step_by(400) {
        let start = Instant::now();
        fib::fib_two_values::<BigUint>(n);
        let elapsed = start.elapsed().as_micros();
        two_values_timings.push((n, elapsed));

        let start = Instant::now();
        fib::fib_two_values::<DynBigUint>(n);
        let elapsed = start.elapsed().as_micros();
        dyn_two_values_timings.push((n, elapsed));

        let start = Instant::now();
        fib::fib_two_values::<OptBigUint>(n);
        let elapsed = start.elapsed().as_micros();
        opt_two_values_timings.push((n, elapsed));
    }

    plot_timings(&two_values_timings, &dyn_two_values_timings, &opt_two_values_timings)?;
    println!("Zeitmessung wurde in timings.svg geplottet");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::fib::{self, FibNum};

    fn max_fib_in_one_second<T: FibNum>() -> u32
    where
        for<'a> &'a T: std::ops::Add<&'a T, Output = T>,
    {
        let mut n = 1u32;
        loop {
            let start = Instant::now();
            fib::fib_two_values::<T>(n);
            if start.elapsed().as_secs_f64() >= 1.0 {
                break;
            }
            n = n.saturating_mul(2);
        }
        let (mut lo, mut hi) = (n / 2, n);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let start = Instant::now();
            fib::fib_two_values::<T>(mid);
            if start.elapsed().as_secs_f64() >= 1.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        lo
    }

    #[test]
    fn test_max_fib_one_second() {
        use crate::biguint::BigUint;
        use crate::dynbiguint::DynBigUint;
        let n_big = max_fib_in_one_second::<BigUint>();
        println!("BigUint:    max fib in <1s = fib({})", n_big);
        let n_dyn = max_fib_in_one_second::<DynBigUint>();
        println!("DynBigUint: max fib in <1s = fib({})", n_dyn);
        let n_opt = max_fib_in_one_second::<crate::optbiguint::OptBigUint>();
        println!("OptBigUint: max fib in <1s = fib({})", n_opt);
    }
}

fn plot_timings(
    two_values_timings: &[(u32, u128)],
    dyn_two_values_timings: &[(u32, u128)],
    opt_two_values_timings: &[(u32, u128)],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let root = SVGBackend::new("timings.svg", (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_n = two_values_timings
        .iter()
        .chain(dyn_two_values_timings)
        .chain(opt_two_values_timings)
        .map(|(n, _)| *n)
        .max()
        .unwrap_or(1);
    let max_time = two_values_timings
        .iter()
        .chain(dyn_two_values_timings)
        .chain(opt_two_values_timings)
        .map(|(_, elapsed)| *elapsed as u64)
        .max()
        .unwrap_or(1);

    let mut chart = ChartBuilder::on(&root)
        .caption("Fibonacci-Zeitmessung", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(1u32..max_n, 0u64..max_time)?;

    chart
        .configure_mesh()
        .x_desc("n")
        .y_desc("Zeit in µs")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            two_values_timings
                .iter()
                .map(|(n, elapsed)| (*n, *elapsed as u64)),
            GREEN,
        ))?
        .label("Two Values (BigUint)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart
        .draw_series(LineSeries::new(
            dyn_two_values_timings
                .iter()
                .map(|(n, elapsed)| (*n, *elapsed as u64)),
            &MAGENTA,
        ))?
        .label("Two Values (DynBigUint)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], MAGENTA));

    chart
        .draw_series(LineSeries::new(
            opt_two_values_timings
                .iter()
                .map(|(n, elapsed)| (*n, *elapsed as u64)),
            BLUE,
        ))?
        .label("Two Values (OptBigUint)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}
