use std::{
    fs,
    iter::{Cycle, Peekable},
    thread::Thread,
};

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
};
use bevy_egui::{
    egui::{self, Color32, ColorImage, ImageButton, Key, TextureHandle, TextureOptions, Ui},
    EguiContext, EguiContexts, EguiPlugin,
};
use bevy_midi::prelude::*;
use midly::{Format, Smf};
use nodi::{timers::Ticker, Player, Sheet};
use strum::{EnumCount, EnumIter, IntoEnumIterator};

//Adapted to bevy_egui from https://github.com/gamercade-io/gamercade_console/blob/audio_editor/gamercade_editor/src/ui/audio/instrument_editor/piano_roll.rs
//All credits goes to @gamercade-io | Apache-2.0, MIT license

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::WARN,
                    filter: "bevy_midi=debug".to_string(),
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (1350., 300.).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
        // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_system(ui_example)
        .init_resource::<PianoRoll>()
        .init_resource::<SelectedInputPort>()
        .init_resource::<SelectedOutputPort>()
        .init_resource::<DragFile>()
        .init_resource::<MidiPlayer>()
        .add_plugin(MidiOutputPlugin)
        .add_plugin(MidiInputPlugin)
        .add_system(select_midi_io_ui)
        .add_system(midi_playback_ui)
        .add_system(debug)
        //.add_system(play_notes)
        .run();
}

#[derive(Resource, Default)]
struct SelectedInputPort(Option<(usize, String)>);

#[derive(Resource, Default)]
struct SelectedOutputPort(Option<(usize, String)>);

#[derive(Resource, Default)]
struct MidiPlayer {
    playing: bool,
    path: Option<String>,
}

const BOTTOM_NOTE_INDEX_START: usize = 36;
const KEYBOARD_KEY_COUNT: usize = 24;
const TOTAL_NOTES_COUNT: usize = 96;
const FIRST_NOTE_OFFSET: usize = 2;
const NOTE_SPACING: f32 = 1.0;
const TOP_KEY_SIZE: bevy_egui::egui::Vec2 = bevy_egui::egui::Vec2::new(12.0, 32.0);
const BOTTOM_KEY_SIZE: bevy_egui::egui::Vec2 = bevy_egui::egui::Vec2::new(
    (((TOP_KEY_SIZE.x + NOTE_SPACING) * TOTAL_NOTES_COUNT as f32) - (NOTE_SPACING * 56.0)) / 56.0,
    24.0,
);

const KEYS: &[Key; KEYBOARD_KEY_COUNT] = &[
    Key::Z,
    Key::S,
    Key::X,
    Key::D,
    Key::C,
    Key::V,
    Key::G,
    Key::B,
    Key::H,
    Key::N,
    Key::J,
    Key::M,
    Key::Q,
    Key::Num2,
    Key::W,
    Key::Num3,
    Key::E,
    Key::R,
    Key::Num5,
    Key::T,
    Key::Num6,
    Key::Y,
    Key::Num7,
    Key::U,
];

#[derive(Debug, Clone, Copy, EnumIter, EnumCount, PartialEq, Eq)]
pub enum NoteName {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
}

impl NoteName {
    pub fn get_key_color(self) -> NoteColor {
        use NoteColor::*;
        match self {
            NoteName::A => White,
            NoteName::ASharp => Black,
            NoteName::B => White,
            NoteName::C => White,
            NoteName::CSharp => Black,
            NoteName::D => White,
            NoteName::DSharp => Black,
            NoteName::E => White,
            NoteName::F => White,
            NoteName::FSharp => Black,
            NoteName::G => White,
            NoteName::GSharp => Black,
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter, EnumCount, PartialEq, Eq)]
pub enum Octave {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NoteColor {
    White,
    Black,
}

pub struct NotesIter {
    count: usize,
    name_iter: Cycle<NoteNameIter>,
    octave_iter: Peekable<OctaveIter>,
}

impl Default for NotesIter {
    fn default() -> Self {
        let octave_iter = Octave::iter().peekable(); //Start at 1
        let mut name_iter = NoteName::iter().cycle(); // Start at A

        name_iter.nth(FIRST_NOTE_OFFSET); // Advance to C1

        Self {
            count: 0,
            name_iter,
            octave_iter,
        }
    }
}

impl Iterator for NotesIter {
    type Item = (NoteName, Octave);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= TOTAL_NOTES_COUNT {
            None
        } else {
            self.count += 1;
            let name = self.name_iter.next().unwrap();

            if name == NoteName::A {
                self.octave_iter.next();
            };

            let octave = self.octave_iter.peek().unwrap();

            Some((name, *octave))
        }
    }
}

#[derive(Resource, Clone)]
pub struct PianoRoll {
    default_piano_texture: Option<TextureHandle>,

    bottom_note_index: usize,
    key_states: [bool; KEYBOARD_KEY_COUNT],
    //key_channels: [Option<usize>; KEYBOARD_KEY_COUNT],
}

impl Default for PianoRoll {
    fn default() -> Self {
        Self {
            default_piano_texture: Default::default(),
            bottom_note_index: BOTTOM_NOTE_INDEX_START,
            key_states: Default::default(),
            //key_channels: Default::default(),
        }
    }
}

impl PianoRoll {
    fn key_in_keyboard_range(&self, index: usize) -> bool {
        index >= self.bottom_note_index && index < self.bottom_note_index + KEYBOARD_KEY_COUNT
    }

    fn update_key_states(&mut self, ui: &mut Ui, midi_output: &MidiOutput) {
        let input = ui.input(|i| i.key_pressed(egui::Key::A));
        let next_keys = std::array::from_fn(|index| ui.input(|i| i.key_down(KEYS[index])));

        self.key_states
            .iter()
            .zip(next_keys.iter())
            .enumerate()
            .for_each(|(index, (prev, next))| {
                if prev != next {
                    println!(
                        "Pressed {}{}",
                        KEY_RANGE[index % 12],
                        (self.bottom_note_index + index) / 12
                    );

                    // Note on, channel 1, max velocity
                    if *next {
                        midi_output.send(
                            [0b1001_0000, (self.bottom_note_index + index) as u8, 127].into(),
                        );
                    } else {
                        midi_output.send(
                            [0b1000_0000, (self.bottom_note_index + index) as u8, 127].into(),
                        );
                    }
                }
            });

        self.key_states = next_keys;
        drop(input);
    }

    fn get_key_texture_tint(&self, note: NoteName, index: usize) -> Color32 {
        const OUT_OF_RANGE: &[Color32; 2] = &[Color32::LIGHT_GRAY, Color32::BLACK];
        const IN_RANGE: &[Color32; 2] = &[Color32::WHITE, Color32::DARK_GRAY];
        const ACTIVE: &[Color32; 2] = &[Color32::GREEN, Color32::GREEN];

        let color = match note.get_key_color() {
            NoteColor::White => 0,
            NoteColor::Black => 1,
        };

        let position = if self.key_in_keyboard_range(index) {
            let inner_index = index - self.bottom_note_index;

            if self.key_states[inner_index] {
                ACTIVE
            } else {
                IN_RANGE
            }
        } else {
            OUT_OF_RANGE
        };

        position[color]
    }

    fn draw_piano_keys(
        &mut self,
        ui: &mut Ui,
        midi_output: &MidiOutput, //selected_instrument: usize,
    ) {
        let texture_id = self
            .default_piano_texture
            .get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "default piano texture",
                    ColorImage::from_rgba_unmultiplied([1, 1], &[255, 255, 255, 255]),
                    TextureOptions::NEAREST,
                )
            })
            .id();

        // Draw the actual piano keys for clicking
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = bevy_egui::egui::Vec2 {
                x: NOTE_SPACING,
                y: 0.0,
            };
            ui.spacing_mut().button_padding = bevy_egui::egui::Vec2 { x: 0.0, y: 0.0 };

            ui.horizontal(|ui| {
                let all_notes_iter = NotesIter::default().enumerate();

                all_notes_iter.for_each(|(index, (note, _octave))| {
                    let color = self.get_key_texture_tint(note, index);

                    let button_top = ImageButton::new(texture_id, TOP_KEY_SIZE).tint(color);
                    if ui.add(button_top).clicked() {
                        //sync.trigger_note(index, selected_instrument);
                        println!("Pressed {}{}", KEY_RANGE[index % 12], index / 12);
                        midi_output.send([0b1001_0000, index as u8, 127].into());
                    };
                });
            });

            ui.spacing_mut().item_spacing = bevy_egui::egui::Vec2 {
                x: NOTE_SPACING,
                y: 0.0,
            };
            ui.horizontal(|ui| {
                let mut white_notes_iter = NotesIter::default().enumerate();

                for (index, (note, _octave)) in white_notes_iter.by_ref() {
                    if note.get_key_color() == NoteColor::White {
                        let tint = self.get_key_texture_tint(note, index);

                        let button_bottom =
                            ImageButton::new(texture_id, BOTTOM_KEY_SIZE).tint(tint);

                        let bs = ui.add(button_bottom);

                        if bs.clicked() {
                            //sync.trigger_note(index, selected_instrument);
                            println!("Pressed {}{}", KEY_RANGE[index % 12], index / 12);
                            midi_output.send([0b1001_0000, index as u8, 127].into());

                            //println!("Released {}{}", KEY_RANGE[index % 12], index / 12);
                        };
                    }
                }
            })
        });
    }
}

fn ui_example(
    egui_context: Query<&EguiContext>,
    mut piano: ResMut<PianoRoll>,
    output: Res<MidiOutput>,
) {
    if let Ok(ctx) = egui_context.get_single() {
        egui::Window::new("Virtual Keyboard Piano").show(ctx.get(), |ui| {
            ui.label(format!(
                "Octave {}-{}",
                piano.bottom_note_index / 12 + 1,
                piano.bottom_note_index / 12 + 2
            ));

            piano.update_key_states(ui, &output);
            // Draws the left/right buttons, and handles
            // Arrow keys going left or right
            ui.horizontal(|ui| {
                let go_left =
                    ui.button("<--").clicked() || (ui.input(|i| i.key_pressed(Key::ArrowLeft)));
                let go_right =
                    ui.button("-->").clicked() || (ui.input(|i| i.key_pressed(Key::ArrowRight)));

                if go_left && piano.bottom_note_index > 0 {
                    piano.bottom_note_index -= 12
                } else if go_right
                    && piano.bottom_note_index < TOTAL_NOTES_COUNT - KEYBOARD_KEY_COUNT
                {
                    piano.bottom_note_index += 12
                }

                piano.draw_piano_keys(ui /*, sync, selected_instrument*/, &output);
            });
        });
    }
}

fn debug(mut midi: EventReader<MidiData>, output: Res<MidiOutput>) {
    for data in midi.iter() {
        let pitch = data.message.msg[1];

        if data.message.is_note_on() {
            output.send([0b1001_0000, pitch, 127].into());
        } else if data.message.is_note_off() {
            output.send([0b1000_0000, pitch, 127].into());
        } else {
        }
    }
}

#[derive(Resource, Default)]
struct DragFile {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
}

fn midi_playback_ui(
    mut contexts: EguiContexts,
    mut df: ResMut<DragFile>,
    mut mp: ResMut<MidiPlayer>,
    output: Res<MidiOutput>,
    mut selected_o_port: ResMut<SelectedOutputPort>,
) {
    let context = contexts.ctx_mut();

    egui::Window::new("Midi Playback").show(context, |ui| {
        ui.label("Load a midi file (.mid) to be played");

        if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                df.picked_path = Some(path.display().to_string());
            }
        }

        if let Some(picked_path) = &df.picked_path {
            ui.horizontal(|ui| {
                ui.label("Playing file:");
                ui.monospace(picked_path);
            });
            if !mp.playing {
                mp.playing = true;

                let data = fs::read(picked_path).unwrap();
                let Smf { header, tracks } = Smf::parse(&data).unwrap();
                let timer = Ticker::try_from(header.timing).unwrap();

                if let Some((id, _)) = selected_o_port.0 {
                    let midi_out = nodi::midir::MidiOutput::new("play_midi").unwrap();
                    let out_ports = midi_out.ports();

                    //Cant have two connection to the same output
                    //output.connect(out_ports[id]);

                    let out_port = &out_ports[id];
                    let con = midi_out.connect(out_port, "cello-tabs").unwrap();

                    let sheet = match header.format {
                        Format::SingleTrack | Format::Sequential => Sheet::sequential(&tracks),
                        Format::Parallel => Sheet::parallel(&tracks),
                    };
                    let mut player = Player::new(timer, con);
                    // thread code
                    player.play(&sheet);
                }
            }
        }

        // Show dropped files (if any):
        if !df.dropped_files.is_empty() {
            ui.group(|ui| {
                ui.label("Dropped files:");

                for file in &df.dropped_files {
                    let mut info = if let Some(path) = &file.path {
                        path.display().to_string()
                    } else if !file.name.is_empty() {
                        file.name.clone()
                    } else {
                        "???".to_owned()
                    };
                    if let Some(bytes) = &file.bytes {
                        use std::fmt::Write as _;
                        write!(info, " ({} bytes)", bytes.len()).ok();
                    }
                    ui.label(info);
                }
            });
        }

        preview_files_being_dropped(context);

        // Collect dropped files:
        context.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                df.dropped_files = i.raw.dropped_files.clone();
            }
        });
    });
}

fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

fn select_midi_io_ui(
    mut contexts: EguiContexts,
    output: Res<MidiOutput>,
    input: Res<MidiInput>,
    mut selected_i_port: ResMut<SelectedInputPort>,
    mut selected_o_port: ResMut<SelectedOutputPort>,
) {
    let context = contexts.ctx_mut();

    egui::Window::new("Select Input & Output MIDI device").show(context, |ui| {
        let i_ports = input.ports().iter().enumerate().collect::<Vec<_>>();
        let o_ports = output.ports().iter().enumerate().collect::<Vec<_>>();

        egui::ComboBox::from_label("Midi Input")
            .width(200.)
            .selected_text(format!(
                "{:?}",
                selected_i_port
                    .0
                    .clone()
                    .unwrap_or_else(|| (0, "None".to_string()))
                    .1
            ))
            .show_ui(ui, |ui| {
                for (index, (port, input_port)) in &i_ports {
                    let value = ui.selectable_value(
                        &mut selected_i_port.0,
                        Some((*index, port.to_string())),
                        port,
                    );
                    if value.clicked() {
                        input.disconnect();
                        input.connect(input_port.clone());
                    }
                }
            });

        egui::ComboBox::from_label("Midi Output")
            .width(200.)
            .selected_text(format!(
                "{:?}",
                selected_o_port
                    .0
                    .clone()
                    .unwrap_or_else(|| (0, "None".to_string()))
                    .1
            ))
            .show_ui(ui, |ui| {
                for (index, (port, output_port)) in &o_ports {
                    let value = ui.selectable_value(
                        &mut selected_o_port.0,
                        Some((*index, port.to_string())),
                        port,
                    );
                    if value.clicked() {
                        output.disconnect();
                        output.connect(output_port.clone());
                    }
                }
            });
    });
}
