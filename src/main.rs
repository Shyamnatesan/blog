#![warn(clippy::pedantic)]

use crate::static_site_gen::AppError;

mod static_site_gen;

fn main() -> Result<(), AppError> {
    static_site_gen::generate()
}
