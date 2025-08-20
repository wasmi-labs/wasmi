use crate::{BranchOffset, Sign, Stack};
use core::num::NonZero;

include!(concat!(env!("OUT_DIR"), "/op.rs"));

impl Copy for Op {}
impl Clone for Op {
    fn clone(&self) -> Self {
        *self
    }
}
