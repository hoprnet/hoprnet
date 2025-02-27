pub mod messages;
pub mod options;
pub mod parsing;

pub use self::options::SearchOptions;

use rand::{self, Rng};

pub fn random_port() -> u16 {
    rand::thread_rng().gen_range(32_768_u16..65_535_u16)
}
