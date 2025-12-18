const SEED: u64 = 0;
const PRIMES: [u64; 5]= [
		11400714785074694791,
		14029467366897019727,
		1609587929392839161,
		9650029242287828579,
		2870177450012600261,
	];

pub trait XXhash64 {
	fn hash(&self) -> u64;
}
impl XXhash64 for Vec<u8> {
	fn hash(&self) -> u64 {
		let mut state: [u64; 4] = [SEED; 4];
		1
	}
}
