use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::dump::DumpState;

lazy_static! {
    pub static ref DUMP_STATE: Mutex<DumpState> = Mutex::new(DumpState::new());
}
