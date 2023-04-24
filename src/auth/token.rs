use rand::RngCore;
use rand_core::OsRng;

pub fn generate_token() -> u64 {
    let mut rng = OsRng::default();
    rng.next_u64()
}
