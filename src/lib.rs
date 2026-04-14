//! `roku-rec` is a small crate for describing and (eventually) replaying terminal sessions.
//!
//! - `roku`: the in-memory representation of a `.roku` script
//! - `parser`: a line-oriented parser that turns `.roku` text into `roku::RokuFile`
pub mod parser;
pub mod roku;
