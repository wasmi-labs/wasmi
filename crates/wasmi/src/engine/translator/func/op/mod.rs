#[macro_use]
mod utils;
mod load;
mod store;
mod unary;

pub use self::{binary::*, load::*, store::*, unary::*};
use self::utils::IntoResult;
