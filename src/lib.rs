//! `take` is a small crate for describing and (eventually) replaying terminal sessions.
//!
//! - `take`: the in-memory representation of a `.take` script
//! - `parser`: a line-oriented parser that turns `.take` text into `take::TakeFile`
//! - `player`: TBD
pub mod parser;
pub mod player;
pub mod renderer;
pub mod take;
