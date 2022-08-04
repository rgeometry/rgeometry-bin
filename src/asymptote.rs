// asymptote
use anes::*;
use pbr::ProgressBar;
use std::time::Duration;
use std::time::Instant;

pub type Step = usize;
pub type Factor = f64;
// step -> (Input, Factor)
//

// in:
//   algo_fn
//   gen_fn
//   min_time
//   n_steps
//
// out:
//   median_unit_time
//   r_sq

// 1. Generate data of size N and factor.
// 2. Measure algorithm time for given data
//   2.1
// Data size
//

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub sample_size: usize,
    pub minimum_step_duration: Duration,
    pub target_rel_stdev: f64,
    pub minimum_batch_duration: Duration,
    pub maximum_batch_duration: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sample_size: 10,
            minimum_step_duration: Duration::from_secs_f64(0.05),
            target_rel_stdev: 0.01, // 1%
            minimum_batch_duration: Duration::from_secs_f64(0.1),
            maximum_batch_duration: Duration::from_secs_f64(5.0),
        }
    }
}

/// Benchmark algorithm and return step cost as x±y.
pub fn asymptote<I: Clone>(
    cfg: Config,
    gen_fn: impl Fn(Step) -> (I, Factor),
    algo: impl Fn(I),
) -> (Duration, Duration) {
    assert!(cfg.sample_size > 0);
    assert!(cfg.target_rel_stdev > 0.0);
    assert!(cfg.target_rel_stdev < 1.0);

    let mut pb = ProgressBar::new(cfg.sample_size as u64);
    pb.show_speed = false;

    let mut samples = Vec::new();
    let mut step = 1;
    while samples.len() < cfg.sample_size {
        eprint!("\r{}Generating data", ClearLine::All,);
        let (data, factor) = gen_fn(step);
        let t = measure_algorithm(cfg, &mut pb, data, &algo);
        if t > cfg.minimum_step_duration {
            // pb.inc();
            samples.push(t.div_f64(factor));
        }
        step += 1;
    }
    pb.finish_print("");
    let avg = samples.iter().sum::<Duration>() / cfg.sample_size as u32;
    let min = *samples.iter().min().unwrap();
    let max = *samples.iter().max().unwrap();
    let span = (max - min) / 2_u32;
    (avg, span)
}

// Keep doubling the sampling size until the stdev between two successive runs is less than `cfg.target_rel_stdev`
fn measure_algorithm<I: Clone>(
    cfg: Config,
    pb: &mut ProgressBar<std::io::Stdout>,
    data: I,
    algo: impl Fn(I),
) -> Duration {
    let sq = |val| val * val;
    let mut n = 1;
    let time_start = Instant::now();
    let mut t_prev = bench_algorithm(n, data.clone(), &algo).as_secs_f64();
    loop {
        let t_now = bench_algorithm(n * 2, data.clone(), &algo).as_secs_f64();
        let t = (t_prev + 2. * t_now) / 5.;
        let stdev = (sq(t_prev - t) + sq(t_now - 2. * t)).sqrt();
        let elapsed = time_start.elapsed();
        eprint!(
            "\r{}Looking for stdev {:.1}% < {:.1}% AND elapsed {:.1?} > {:.1?}",
            ClearLine::All,
            stdev / t * 100.0,
            cfg.target_rel_stdev * 100.0,
            elapsed,
            cfg.minimum_batch_duration
        );
        if (stdev < cfg.target_rel_stdev * t && elapsed > cfg.minimum_batch_duration)
            || elapsed > cfg.maximum_batch_duration
        {
            eprint!("\n{}", ClearLine::All,);
            return Duration::from_secs_f64(t / (n as f64));
        }
        n *= 2;
        t_prev = t_now;
        // pb.tick();
    }
}

fn bench_algorithm<I: Clone>(n: usize, data: I, algo: impl Fn(I)) -> Duration {
    eprint!("\r{}Cloning data", ClearLine::All,);
    let input = vec![data; n];
    eprint!("\r{}Measuring...", ClearLine::All,);
    let now = Instant::now();
    for val in input.into_iter() {
        algo(black_box(val));
    }
    now.elapsed()
}

fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}
