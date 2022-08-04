use crate::asymptote::{asymptote, Config, Factor};
use std::time::Duration;

pub struct Algo {
    pub name: &'static str,
    pub run: fn(Config) -> (Duration, Duration),
}

pub const ALGORITHMS: [Algo; 1] = [EARCLIP];

// const SLEEP_LINEAR: Algo = Algo {
//     name: "sleep (linear)",
//     run: |config| {
//         asymptote(
//             config,
//             |step| (Duration::from_secs_f64(step as f64 * 0.1), step as Factor),
//             |i| std::thread::sleep(i),
//         )
//     },
// };

// const SLEEP_LOG: Algo = Algo {
//     name: "sleep (nlogn)",
//     run: |config| {
//         asymptote(
//             config,
//             |step| {
//                 (
//                     Duration::from_secs_f64(step as f64 * (step as f64).log2() * 0.1),
//                     step as Factor * (step as f64).log2(),
//                 )
//             },
//             |i| std::thread::sleep(i),
//         )
//     },
// };

const EARCLIP: Algo = Algo {
    name: "earclip",
    run: |config| {
        use rand::distributions::Standard;
        use rand::{Rng, SeedableRng};
        use rgeometry::algorithms::polygonization;
        use rgeometry::algorithms::triangulation::earclip;
        use rgeometry::data::{Point, Polygon};
        asymptote(
            config,
            |step| {
                let n = 3 + (1.25_f64).powf(step as f64) as usize;
                let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
                let mut pts = vec![];
                while pts.len() < n {
                    let pt: Point<f64> = rng.sample(Standard);
                    let pt: Point<ordered_float::OrderedFloat<f64>> = pt.cast();
                    if pts.contains(&pt) {
                        continue;
                    }
                    pts.push(pt);
                }
                let ret = polygonization::two_opt_moves(pts, &mut rng).unwrap();
                (
                    ret as Polygon<ordered_float::OrderedFloat<f64>>,
                    (n * n / 10) as Factor,
                )
            },
            |polygon| {
                earclip::earclip(&polygon).count();
            },
        )
    },
};
