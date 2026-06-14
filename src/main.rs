use biguint::BigUint;
use dynbiguint::DynBigUint;
use optbiguint::OptBigUint;
use plotters::prelude::*;
use std::error::Error;
use std::io::Write;
use std::time::Instant;

mod biguint;
mod dynbiguint;
mod fib;
mod optbiguint;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(run)?
        .join()
        .unwrap()
}

fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    benchmark_all_variants()?;
    benchmark_dyn_vs_matrix()?;
    Ok(())
}

fn print_progress(done_work: f64, total_work: f64, n: u32, run_start: Instant) {
    const BAR_WIDTH: usize = 40;
    let pct = done_work / total_work * 100.0;
    let filled = ((pct / 100.0) * BAR_WIDTH as f64) as usize;
    let elapsed = run_start.elapsed().as_secs_f64();
    let eta_secs = (elapsed / (pct / 100.0)) * (1.0 - pct / 100.0);
    let eta_str = if pct < 0.1 {
        "   --:--".to_string()
    } else if eta_secs >= 3600.0 {
        format!(
            "{:2}h{:02}m{:02}s",
            eta_secs as u64 / 3600,
            (eta_secs as u64 % 3600) / 60,
            eta_secs as u64 % 60
        )
    } else {
        format!("   {:2}m{:02}s", eta_secs as u64 / 60, eta_secs as u64 % 60)
    };
    print!(
        "\r[{}>{}] {:5.1}%  n={:>7}  ETA {}",
        "#".repeat(filled),
        "-".repeat(BAR_WIDTH.saturating_sub(filled)),
        pct,
        n,
        eta_str
    );
    std::io::stdout().flush().unwrap();
}

fn benchmark_all_variants() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut big_timings = Vec::new();
    let mut dyn_timings = Vec::new();
    let mut opt_timings = Vec::new();
    let mut dyn_ip_timings = Vec::new();
    let mut opt_ip_timings = Vec::new();

    let steps: Vec<u32> = (1..=700_000u32).step_by(700).collect();
    let total_work: f64 = steps.iter().map(|&n| (n as f64).powf(1.5)).sum();
    let mut done_work: f64 = 0.0;
    let run_start = Instant::now();

    for &n in &steps {
        let start = Instant::now();
        fib::fib_two_values::<BigUint>(n);
        big_timings.push((n, start.elapsed().as_micros()));

        let start = Instant::now();
        fib::fib_two_values::<DynBigUint>(n);
        dyn_timings.push((n, start.elapsed().as_micros()));

        let start = Instant::now();
        fib::fib_two_values::<OptBigUint>(n);
        opt_timings.push((n, start.elapsed().as_micros()));

        let start = Instant::now();
        fib::fib_inplace_two_values::<DynBigUint>(n);
        dyn_ip_timings.push((n, start.elapsed().as_micros()));

        let start = Instant::now();
        fib::fib_inplace_two_values::<OptBigUint>(n);
        opt_ip_timings.push((n, start.elapsed().as_micros()));

        done_work += (n as f64).powf(1.5);
        print_progress(done_work, total_work, n, run_start);
    }
    println!();

    plot_all_variants(
        &big_timings,
        &dyn_timings,
        &opt_timings,
        &dyn_ip_timings,
        &opt_ip_timings,
    )?;
    println!("Zeitmessung wurde in timings.svg geplottet");
    Ok(())
}

fn benchmark_dyn_vs_matrix() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut two_values_timings = Vec::new();
    let mut matrix_timings = Vec::new();

    let steps: Vec<u32> = (1..=1_400_001u32).step_by(1400).collect();
    let total_work: f64 = steps.iter().map(|&n| (n as f64).powf(2.1)).sum();
    let mut done_work: f64 = 0.0;
    let run_start = Instant::now();

    for &n in &steps {
        let start = Instant::now();
        fib::fib_two_values::<DynBigUint>(n);
        two_values_timings.push((n, start.elapsed().as_micros()));

        let start = Instant::now();
        fib::fib_matrix_mult::<DynBigUint>(n);
        matrix_timings.push((n, start.elapsed().as_micros()));

        done_work += (n as f64).powf(2.1);
        print_progress(done_work, total_work, n, run_start);
    }
    println!();

    plot_dyn_vs_matrix(&two_values_timings, &matrix_timings)?;
    println!("Zeitmessung wurde in timings_dyn_vs_matrix.svg geplottet");
    Ok(())
}

fn plot_all_variants(
    big_timings: &[(u32, u128)],
    dyn_timings: &[(u32, u128)],
    opt_timings: &[(u32, u128)],
    dyn_ip_timings: &[(u32, u128)],
    opt_ip_timings: &[(u32, u128)],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let all = [
        big_timings,
        dyn_timings,
        opt_timings,
        dyn_ip_timings,
        opt_ip_timings,
    ];
    let max_n = all
        .iter()
        .flat_map(|s| s.iter())
        .map(|(n, _)| *n)
        .max()
        .unwrap_or(1);
    let max_ms = all
        .iter()
        .flat_map(|s| s.iter())
        .map(|(_, us)| *us as f64 / 1000.0)
        .fold(0f64, f64::max);

    let to_ms = |data: &[(u32, u128)]| {
        data.iter()
            .map(|(n, us)| (*n, *us as f64 / 1000.0))
            .collect::<Vec<_>>()
    };

    let col_big = RGBColor(34, 139, 34);
    let col_dyn = RGBColor(214, 102, 21);
    let col_opt = RGBColor(70, 130, 180);
    let col_dyn_ip = RGBColor(148, 52, 186);
    let col_opt_ip = RGBColor(0, 160, 136);

    let root = SVGBackend::new("timings.svg", (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Fibonacci-Zeitmessung", ("sans-serif", 36))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(1u32..max_n, 0f64..max_ms * 1.05)?;

    chart
        .configure_mesh()
        .x_desc("n")
        .y_desc("Zeit in ms")
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    macro_rules! draw {
        ($data:expr, $label:expr, $color:expr) => {
            chart
                .draw_series(LineSeries::new(to_ms($data), $color))?
                .label($label)
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], $color));
        };
    }

    draw!(big_timings, "BigUint", col_big);
    draw!(dyn_timings, "DynBigUint", col_dyn);
    draw!(opt_timings, "OptBigUint", col_opt);
    draw!(dyn_ip_timings, "DynBigUint (ip)", col_dyn_ip);
    draw!(opt_ip_timings, "OptBigUint (ip)", col_opt_ip);

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn plot_dyn_vs_matrix(
    two_values_timings: &[(u32, u128)],
    matrix_timings: &[(u32, u128)],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let all = [two_values_timings, matrix_timings];
    let max_n = all
        .iter()
        .flat_map(|s| s.iter())
        .map(|(n, _)| *n)
        .max()
        .unwrap_or(1);
    let max_ms = all
        .iter()
        .flat_map(|s| s.iter())
        .map(|(_, us)| *us as f64 / 1000.0)
        .fold(0f64, f64::max);

    let to_ms = |data: &[(u32, u128)]| {
        data.iter()
            .map(|(n, us)| (*n, *us as f64 / 1000.0))
            .collect::<Vec<_>>()
    };

    let col_two_values = RGBColor(214, 102, 21);
    let col_matrix = RGBColor(70, 130, 180);

    let root = SVGBackend::new("timings_dyn_vs_matrix.svg", (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("DynBigUint: two_values vs matrix_mult", ("sans-serif", 36))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(1u32..max_n, 0f64..max_ms * 1.05)?;

    chart
        .configure_mesh()
        .x_desc("n")
        .y_desc("Zeit in ms")
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    macro_rules! draw {
        ($data:expr, $label:expr, $color:expr) => {
            chart
                .draw_series(LineSeries::new(to_ms($data), $color))?
                .label($label)
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], $color));
        };
    }

    draw!(
        two_values_timings,
        "DynBigUint (two_values)",
        col_two_values
    );
    draw!(matrix_timings, "DynBigUint (matrix)", col_matrix);

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fib;
    use std::time::Instant;

    fn max_fib_in_one_second(fib_fn: impl Fn(u32), max_n: Option<u32>) -> u32 {
        let mut n = 1u32;
        loop {
            let start = Instant::now();
            fib_fn(n);
            if start.elapsed().as_secs_f64() >= 1.0 {
                break;
            }
            match max_n {
                Some(max) if n >= max => break,
                Some(max) => n = n.saturating_mul(2).min(max),
                None => n = n.saturating_mul(2),
            }
        }
        let (mut lo, mut hi) = (n / 2, n);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let start = Instant::now();
            fib_fn(mid);
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
        use crate::optbiguint::OptBigUint;

        let n_big = max_fib_in_one_second(
            |n| {
                fib::fib_two_values::<BigUint>(n);
            },
            Some(700_000),
        );
        println!("BigUint:    max fib in <1s = fib({})", n_big);
        let n_dyn = max_fib_in_one_second(
            |n| {
                fib::fib_two_values::<DynBigUint>(n);
            },
            None,
        );
        println!("DynBigUint: max fib in <1s = fib({})", n_dyn);
        let n_opt = max_fib_in_one_second(
            |n| {
                fib::fib_two_values::<OptBigUint>(n);
            },
            Some(700_000),
        );
        println!("OptBigUint: max fib in <1s = fib({})", n_opt);
        let n_dyn_ip = max_fib_in_one_second(
            |n| {
                fib::fib_inplace_two_values::<DynBigUint>(n);
            },
            None,
        );
        println!("DynBigUint (ip): max fib in <1s = fib({})", n_dyn_ip);
        let n_opt_ip = max_fib_in_one_second(
            |n| {
                fib::fib_inplace_two_values::<OptBigUint>(n);
            },
            Some(700_000),
        );
        println!("OptBigUint (ip): max fib in <1s = fib({})", n_opt_ip);
        let n_mat_dyn = max_fib_in_one_second(
            |n| {
                fib::fib_matrix_mult::<DynBigUint>(n);
            },
            None,
        );
        println!("DynBigUint (matrix): max fib in <1s = fib({})", n_mat_dyn);
    }
}
