mod backend;
mod frontend;
mod ir;
mod optimizer;

mod globals;
mod structs;

pub mod driver;
mod options;
mod types;

use globals::with_session_globals;
