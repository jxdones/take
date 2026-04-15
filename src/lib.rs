//! `take` is a small crate for describing and (eventually) replaying terminal sessions.
//!
//! - `take`: the in-memory representation of a `.take` script
//! - `parser`: a line-oriented parser that turns `.take` text into `take::TakeFile`
pub mod parser;
pub mod take;
