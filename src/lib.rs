pub mod input;
pub mod output;

pub const KEY_RANGE: [&str; 12] = [
    "C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B",
];

const NOTE_ON_STATUS: u8 = 0b1001_0000;
const NOTE_OFF_STATUS: u8 = 0b1000_0000;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MidiMessage {
    pub msg: [u8; 3],
}

impl From<[u8; 3]> for MidiMessage {
    fn from(msg: [u8; 3]) -> Self {
        MidiMessage { msg }
    }
}

impl MidiMessage {
    #[must_use]
    pub fn is_note_on(&self) -> bool {
        (self.msg[0] & 0b1111_0000) == NOTE_ON_STATUS
    }

    #[must_use]
    pub fn is_note_off(&self) -> bool {
        (self.msg[0] & 0b1111_0000) == NOTE_OFF_STATUS
    }

    /// Get the channel of a message, assuming the message is not a system message.
    #[must_use]
    pub fn channel(&self) -> u8 {
        self.msg[0] & 0b0000_1111
    }
}
