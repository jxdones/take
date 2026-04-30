//! `take` is a small crate for recording and replaying terminal sessions as animated GIFs.
//!
//! - `take`: the in-memory representation of a `.take` script
//! - `parser`: a line-oriented parser that turns `.take` text into `take::TakeFile`
//! - `player`: executes a `TakeFile` against a real terminal, capturing output frames
//! - `box_drawing`: geometric rendering of Unicode box-drawing characters
//! - `renderer`: converts captured frames into an animated GIF
pub mod box_drawing;
pub mod parser;
pub mod player;
pub mod renderer;
pub mod take;
