/// Re-export [`midly::num`] module .
pub mod num {
    pub use midly::num::{u14, u15, u24, u28, u4, u7};
}

pub mod input;
pub mod output;
pub mod types;

pub mod prelude {
    pub use crate::{input::*, output::*, types::*, *};
}

pub const KEY_RANGE: [&str; 12] = [
    "C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B",
];
