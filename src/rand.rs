use std::rand::TaskRng;
use std::rand::distributions::{IndependentSample,Range};
use std::rand;

static CHARACTERS: &'static str =
    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
static SEQUENCE_LENGTH: uint = 6;

pub struct RandomSequences {
    rng: TaskRng,
}

impl RandomSequences {
    pub fn new() -> RandomSequences {
        RandomSequences {
            rng: rand::task_rng(),
        }
    }
}

impl Iterator<String> for RandomSequences {
    fn next(&mut self) -> Option<String> {
        let between = Range::new(0, CHARACTERS.char_len());

        Some(range(0, SEQUENCE_LENGTH).map(|_| {
            CHARACTERS.char_at(between.ind_sample(&mut self.rng))
        }).collect())
    }
}
