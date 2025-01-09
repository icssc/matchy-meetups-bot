use rand::prelude::SliceRandom;
use rand::SeedableRng;

/// Creates "pairs" from the vector (up to one triple is created if there is not an even number).
/// Each pair is represented as a smaller vector
/// within the larger returned vector.
pub fn pair<T: Clone>(mut vec: Vec<T>, seed: u64) -> Vec<Vec<T>> {
    // TODO: smarter pairing algorithm (easiest way might be to generate N random pairings, score
    //  each one, and then pick the best, or maybe one randomly weighted by score).
    //  Alternative is to do some smarter algorithm but I'm not entirely sure what that looks like.
    if vec.len() <= 1 {
        panic!("Cannot pair with <= 1 elements.")
    }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    vec.shuffle(&mut rng);
    let chunks = vec.chunks_exact(2);
    let remainder = chunks.remainder();
    let mut x: Vec<Vec<T>> = chunks.map(|chunk| chunk.to_vec()).collect();
    x.last_mut().unwrap().extend_from_slice(remainder);
    x
}
