use std::{convert::TryFrom, error::Error, fs};

use nodi::midir::{MidiOutput, MidiOutputConnection};
use nodi::{
    midly::{Format, Smf},
    timers::Ticker,
    Connection, Player, Sheet,
};

fn get_connection(n: usize) -> Result<MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("play_midi")?;

    let out_ports = midi_out.ports();
    if out_ports.is_empty() {
        return Err("no MIDI output device detected".into());
    }
    if n >= out_ports.len() {
        return Err(format!(
            "only {} MIDI devices detected; run with --list  to see them",
            out_ports.len()
        )
        .into());
    }

    let out_port = &out_ports[n];
    let out = midi_out.connect(out_port, "cello-tabs")?;
    Ok(out)
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = fs::read("C:\\Users\\thelu\\Downloads\\Billy_Joel_-_Piano_Man.mid")?;
    let Smf { header, tracks } = Smf::parse(&data)?;
    let timer = Ticker::try_from(header.timing)?;

    let con = get_connection(0)?;

    let sheet = match header.format {
        Format::SingleTrack | Format::Sequential => Sheet::sequential(&tracks),
        Format::Parallel => Sheet::parallel(&tracks),
    };

    let mut player = Player::new(timer, con);

    println!("starting playback");
    player.play(&sheet);
    Ok(())
}
