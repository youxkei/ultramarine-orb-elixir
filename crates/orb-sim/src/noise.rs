//! The part of the host that is not a function of anything orb can see.
//!
//! A simulated Windows that answers the same thing every time is not a Windows: the real one wakes a
//! thread when it gets round to it, and the pacing's behaviour turns on that. So the simulator is
//! deliberately non-deterministic, and a scenario that only passes for some of it is a scenario whose
//! failure would happen on a real machine too.
//!
//! Seeded, because a failure has to be reproducible: the seed goes in the assertion message and
//! re-running with it replays exactly the run that failed.
//!
//! xorshift64* rather than a crate. What is wanted is a stream of numbers with no structure a test
//! could accidentally rely on, over a range of a millisecond or two; the statistical properties a
//! generator is chosen for do not come into it, and a dependency that has to be explained does.

pub struct Noise {
    state: u64,
}

impl Noise {
    pub fn seeded(seed: u64) -> Self {
        // Zero is a fixed point of the shift, so it is moved off it. Any odd constant does.
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A number in `0..=most`, which is how every delay here is drawn.
    pub fn up_to(&mut self, most: i64) -> i64 {
        if most <= 0 {
            return 0;
        }
        // Modulo, whose bias is one part in 2^64 / most and so is nothing beside the millisecond
        // being modelled.
        (self.next() % (most as u64 + 1)) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::Noise;

    /// The same seed is the same run, which is what makes a failure reproducible.
    #[test]
    fn a_seed_replays_exactly() {
        let draw = |seed| {
            let mut noise = Noise::seeded(seed);
            (0..16).map(|_| noise.up_to(1_000)).collect::<Vec<_>>()
        };
        assert_eq!(draw(7), draw(7));
        assert_ne!(draw(7), draw(8));
    }

    /// And it stays inside the range, which every caller relies on to keep a blank from moving
    /// backwards.
    #[test]
    fn nothing_is_drawn_outside_the_range() {
        let mut noise = Noise::seeded(1);
        for _ in 0..10_000 {
            let drawn = noise.up_to(1_600);
            assert!((0..=1_600).contains(&drawn), "{drawn}");
        }
        assert_eq!(noise.up_to(0), 0, "a range of nothing is not a range");
    }

    /// Spread over the range rather than clustered, because a scenario that only fails for a delay
    /// near one end has to be able to reach that end.
    #[test]
    fn the_whole_range_is_reached() {
        let mut noise = Noise::seeded(3);
        let mut buckets = [0usize; 8];
        for _ in 0..8_000 {
            buckets[(noise.up_to(799) / 100) as usize] += 1;
        }
        assert!(
            buckets.iter().all(|count| *count > 700),
            "{buckets:?} of 8000 over eight buckets"
        );
    }
}
