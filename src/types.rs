//! Module definined owned variants of `[midly]` structures. These owned variants allow for more
//! ergonomic usage.
use midly::live::{LiveEvent, SystemCommon};
use midly::num;
pub use midly::{
    live::{MtcQuarterFrameMessage, SystemRealtime},
    MidiMessage,
};

/// Owned version of a [`midly::live::LiveEvent`].
///
/// Standard [`midly::live::LiveEvent`]s have a lifetime parameter limiting them to the scope in
/// which they are generated to avoid any copying. However, because we are sending these messages
/// through the bevy event system, they need to outlive this original scope.
///
/// Creating [`OwnedLiveEvent`]s only allocates when the message is a an [`OwnedSystemCommon`] that
/// itself contains an allocation.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum OwnedLiveEvent {
    /// A midi message with a channel and music data.
    Midi {
        channel: num::u4,
        message: midly::MidiMessage,
    },

    /// A System Common message with owned data.
    Common(OwnedSystemCommon),

    /// A one-byte System Realtime Message.
    Realtime(SystemRealtime),
}

/// Owned version of [`midly::live::SystemCommon`].
///
/// [`OwnedSystemCommon`] fully owns any underlying value data, including
/// [`OwnedSystemCommon::SysEx`] messages.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum OwnedSystemCommon {
    /// A system-exclusive event.
    ///
    /// Only contains the data bytes; does not inclde the `0xF0` and `0xF6` begin/end marker bytes.
    /// slice does not include either: it only includes data bytes in the `0x00..=0x7F` range.
    SysEx(Vec<num::u7>),
    /// A MIDI Time Code Quarter Frame message, carrying a tag type and a 4-bit tag value.
    MidiTimeCodeQuarterFrame(MtcQuarterFrameMessage, num::u4),
    /// The number of MIDI beats (6 x MIDI clocks) that have elapsed since the start of the
    /// sequence.
    SongPosition(num::u14),
    /// Select a given song index.
    SongSelect(num::u7),
    /// Request the device to tune itself.
    TuneRequest,
    /// An undefined System Common message, with arbitrary data bytes.
    Undefined(u8, Vec<num::u7>),
}

impl OwnedLiveEvent {
    /// Returns a [`MidiMessage::NoteOn`] event.
    pub fn note_on<C: Into<num::u4>, K: Into<num::u7>, V: Into<num::u7>>(
        channel: C,
        key: K,
        vel: V,
    ) -> OwnedLiveEvent {
        OwnedLiveEvent::Midi {
            channel: channel.into(),
            message: midly::MidiMessage::NoteOn {
                key: key.into(),
                vel: vel.into(),
            },
        }
    }

    /// Returns a [`MidiMessage::NoteOff`] event.
    pub fn note_off<C: Into<num::u4>, K: Into<num::u7>, V: Into<num::u7>>(
        channel: C,
        key: K,
        vel: V,
    ) -> OwnedLiveEvent {
        OwnedLiveEvent::Midi {
            channel: channel.into(),
            message: midly::MidiMessage::NoteOff {
                key: key.into(),
                vel: vel.into(),
            },
        }
    }
}

impl<'a> From<LiveEvent<'a>> for OwnedLiveEvent {
    fn from(value: LiveEvent) -> Self {
        match value {
            LiveEvent::Midi { channel, message } => OwnedLiveEvent::Midi { channel, message },
            LiveEvent::Realtime(rt) => OwnedLiveEvent::Realtime(rt),
            LiveEvent::Common(sc) => OwnedLiveEvent::Common(match sc {
                SystemCommon::MidiTimeCodeQuarterFrame(m, v) => {
                    OwnedSystemCommon::MidiTimeCodeQuarterFrame(m, v)
                }
                SystemCommon::SongPosition(pos) => OwnedSystemCommon::SongPosition(pos),
                SystemCommon::SongSelect(ss) => OwnedSystemCommon::SongSelect(ss),
                SystemCommon::TuneRequest => OwnedSystemCommon::TuneRequest,
                SystemCommon::SysEx(b) => OwnedSystemCommon::SysEx(b.to_vec()),
                SystemCommon::Undefined(tag, b) => OwnedSystemCommon::Undefined(tag, b.to_vec()),
            }),
        }
    }
}

impl<'a, 'b: 'a> From<&'b OwnedLiveEvent> for LiveEvent<'a> {
    fn from(value: &'b OwnedLiveEvent) -> Self {
        match value {
            OwnedLiveEvent::Midi { channel, message } => LiveEvent::Midi {
                channel: *channel,
                message: *message,
            },
            OwnedLiveEvent::Realtime(rt) => LiveEvent::Realtime(*rt),
            OwnedLiveEvent::Common(sc) => LiveEvent::Common(match sc {
                OwnedSystemCommon::MidiTimeCodeQuarterFrame(m, v) => {
                    SystemCommon::MidiTimeCodeQuarterFrame(*m, *v)
                }
                OwnedSystemCommon::SongPosition(pos) => SystemCommon::SongPosition(*pos),
                OwnedSystemCommon::SongSelect(ss) => SystemCommon::SongSelect(*ss),
                OwnedSystemCommon::TuneRequest => SystemCommon::TuneRequest,
                OwnedSystemCommon::SysEx(b) => SystemCommon::SysEx(b),
                OwnedSystemCommon::Undefined(tag, b) => SystemCommon::Undefined(*tag, b),
            }),
        }
    }
}
