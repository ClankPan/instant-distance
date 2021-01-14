use std::collections::HashSet;

use ordered_float::OrderedFloat;
use rand::rngs::{StdRng, ThreadRng};
use rand::{Rng, SeedableRng};

use instant_distance::{Hnsw, Point as _, PointId, Search};

#[test]
fn basic() {
    let (hnsw, pids) = Hnsw::<Point>::builder().build(&[
        Point(0.1, 0.4),
        Point(-0.324, 0.543),
        Point(0.87, -0.33),
        Point(0.452, 0.932),
    ]);

    let mut search = Search::default();
    let mut results = vec![PointId::default()];
    let p = Point(0.1, 0.35);
    let found = hnsw.search(&p, &mut results, &mut search);
    assert_eq!(found, 1);
    assert_eq!(&results, &[pids[0]]);
}

#[test]
fn randomized() {
    let seed = ThreadRng::default().gen::<u64>();
    println!("seed {}", seed);

    let mut rng = StdRng::seed_from_u64(seed);
    let points = (0..1024)
        .into_iter()
        .map(|_| Point(rng.gen(), rng.gen()))
        .collect::<Vec<_>>();

    let query = Point(rng.gen(), rng.gen());
    println!("query: {:?}", query);

    let mut nearest = Vec::with_capacity(256);
    for (i, p) in points.iter().enumerate() {
        nearest.push((OrderedFloat::from(query.distance(p)), i));
        if nearest.len() >= 200 {
            nearest.sort_unstable();
            nearest.truncate(100);
        }
    }

    let (hnsw, pids) = Hnsw::<Point>::builder().seed(seed).build(&points);
    let mut search = Search::default();
    let mut results = vec![PointId::default(); 100];
    let found = hnsw.search(&query, &mut results, &mut search);
    assert_eq!(found, 100);

    nearest.sort_unstable();
    nearest.truncate(100);
    let forced = nearest
        .iter()
        .map(|(_, i)| pids[*i])
        .collect::<HashSet<_>>();
    let found = results.into_iter().take(found).collect::<HashSet<_>>();

    let recall = forced.intersection(&found).count();
    println!("{} matched", recall);
    assert!(recall > 90);
}

#[derive(Clone, Copy, Debug)]
struct Point(f32, f32);

impl instant_distance::Point for Point {
    fn distance(&self, other: &Self) -> f32 {
        // Euclidean distance metric
        ((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)).sqrt()
    }
}