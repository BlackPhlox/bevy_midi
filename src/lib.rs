pub mod input;
pub mod output;

pub const KEY_RANGE: [&str; 12] = [
    "C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B",
];

const NOTE_ON_STATUS: u8 = 0b10010000;
const NOTE_OFF_STATUS: u8 = 0b10000000;

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
    pub fn is_note_on(&self) -> bool {
        (self.msg[0] & 0b11110000) == NOTE_ON_STATUS
    }

    pub fn is_note_off(&self) -> bool {
        (self.msg[0] & 0b11110000) == NOTE_OFF_STATUS
    }

    /// Get the channel of a message, assuming the message is not a system message.
    pub fn channel(&self) -> u8 {
        self.msg[0] & 0b00001111
    }
}
