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

// NOTE:
// We need to do some experimentation here. raw pointers might be faster than
// stack allocating arrays here
// We also don't know the complexity of try_into(), which returns a
// result to unwrap as well
#[inline]
fn make_block(data: &[u8]) -> [u64; 4] {
	assert!(data.len() == 32);
	[
		u64::from_le_bytes(data[0..8].try_into().unwrap()),
		u64::from_le_bytes(data[8..16].try_into().unwrap()),
		u64::from_le_bytes(data[16..24].try_into().unwrap()),
		u64::from_le_bytes(data[24..32].try_into().unwrap()),
	]
}

impl XXhash64 for Vec<u8> {
	fn hash(&self) -> u64 {
		let mut state: [Wrapping<u64>; 4] = [Wrapping(SEED); 4];
		init_state(&mut state);

		let mut i = 0;
		while i + 32 <= self.len() {
			let block = make_block(&self[i..i+32]);
			state[0] = process(state[0], block[0]);
			state[1] = process(state[1], block[1]);
			state[2] = process(state[2], block[2]);
			state[3] = process(state[3], block[3]);
			i += 32;
		}

		// input length less than 32 bytes means that the previous 4 lane split
		// has not occured
		let mut result: Wrapping<u64> = if self.len() < 32 {
			state[2] + PRIMES[4]
		} else {
			let mut tmp =
				Wrapping(rot_left(state[0].0, 1)) +
				Wrapping(rot_left(state[1].0, 7)) +
				Wrapping(rot_left(state[2].0, 12)) +
				Wrapping(rot_left(state[3].0, 18));
			tmp = (tmp ^ process(Wrapping(0), state[0].0)) * PRIMES[0] + PRIMES[3];
			tmp = (tmp ^ process(Wrapping(0), state[1].0)) * PRIMES[0] + PRIMES[3];
			tmp = (tmp ^ process(Wrapping(0), state[2].0)) * PRIMES[0] + PRIMES[3];
			tmp = (tmp ^ process(Wrapping(0), state[3].0)) * PRIMES[0] + PRIMES[3];
			tmp
		};
		result += Wrapping(self.len() as u64);

		// handle 8 bytes now
		while i + 8 < self.len() {
			let data = u64::from_le_bytes(self[i..i+8].try_into().unwrap());

			result = Wrapping(rot_left(
				result.0 ^ process(
						Wrapping(0),
						data,
					).0,
				27
			)) * PRIMES[1] + PRIMES[3];
			i += 8;
		}

		// handle remainder 4 byte chunks
		while i + 4 < self.len() {
			let data = u32::from_le_bytes(self[i..i+4].try_into().unwrap())
				as u64;
			result = Wrapping(rot_left(
				(result ^ Wrapping(data) * PRIMES[0]).0,
				23
			)) * PRIMES[1] + PRIMES[2];
			i += 4;
		}

		// single byte
		while i < self.len() {
			result = Wrapping(rot_left(
				(result ^ Wrapping(self[i] as u64) * PRIMES[4]).0,
				11
			)) * PRIMES[0];
			i += 1;
		}
		result.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
