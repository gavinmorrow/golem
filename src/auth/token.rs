use rand::RngCore;
use rand_core::OsRng;

use crate::model::session::Token;

pub fn generate_token() -> Token {
    let mut rng = OsRng::default();
    rng.next_u64() as Token
}
