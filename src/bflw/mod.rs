pub mod file_iter;
mod float_str;
pub mod market_book;
pub mod market_definition;
pub mod market_definition_runner;
pub mod runner_book;
mod runner_book_sp;

pub trait RoundToCents {
    fn round_cent(self) -> Self;
}

impl RoundToCents for f64 {
    fn round_cent(self) -> Self {
        (self * 100f64).round() / 100f64
    }
}
