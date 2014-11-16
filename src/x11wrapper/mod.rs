#![feature(globs)]

/**
 * Simple stupid wrappers arround X11 stuff
 *
 */


pub use self::display::*;
pub mod display;
pub mod window;