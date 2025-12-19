use std::num::Wrapping;

const SEED: u64 = 0;
const PRIMES: [Wrapping<u64>; 5]= [
		Wrapping(11400714785074694791),
		Wrapping(14029467366897019727),
		Wrapping(1609587929392839161),
		Wrapping(9650029242287828579),
		Wrapping(2870177450012600261),
	];

pub trait XXhash64 {
	fn hash(&self) -> u64;
}

fn init_state(state: &mut [Wrapping<u64>; 4]) {
	state[0] += PRIMES[0] + PRIMES[1];
	state[1] += PRIMES[1];
	// state[2] already equal to SEED
	state[3] -= PRIMES[0];
}

fn rot_left(input: u64, shift: u64) -> u64 {
	input << shift | input >> (64 - shift)
}
// does a single iteration of processing
fn process(prev_state: Wrapping<u64>, input: u64) -> Wrapping<u64> {
	Wrapping(
		rot_left((prev_state + Wrapping(input) * PRIMES[1]).0, 31)
	) * PRIMES[0]
}

impl XXhash64 for Vec<u8> {
	fn hash(&self) -> u64 {
		let mut state: [Wrapping<u64>; 4] = [Wrapping(SEED); 4];
		init_state(&mut state);

		let mut i = 0;
		1
	}
}
