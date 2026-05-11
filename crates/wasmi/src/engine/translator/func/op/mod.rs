#[macro_use]
mod utils;
mod binary;
mod load;
mod store;
mod unary;

use self::utils::IntoResult;
pub use self::{binary::*, load::*, store::*, unary::*};
