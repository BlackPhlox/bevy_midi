<div align="left">
<a href="https://github.com/BlackPhlox/bevy_midi"><img src="https://raw.githubusercontent.com/BlackPhlox/BlackPhlox/master/bevy_midi.svg" alt="bevy_midi"></a>
</div>


<div align="left">
<a href="https://crates.io/crates/bevy_midi"><img src="https://img.shields.io/crates/v/bevy_midi" alt="link to crates.io"></a>
<a href="https://docs.rs/bevy_midi"><img src="https://docs.rs/bevy_midi/badge.svg" alt="link to docs.rs"></a>
<a href="https://github.com/BlackPhlox/bevy_midi/blob/master/LICENSE-MIT"><img src="https://img.shields.io/crates/l/bevy_midi" alt="link to license"></a>
<a href="https://crates.io/crates/bevy_midi"><img src="https://img.shields.io/crates/d/bevy_midi" alt="downloads/link to crates.io"></a>   
<a href="https://github.com/BlackPhlox/bevy_midi"><img src="https://img.shields.io/github/stars/BlackPhlox/bevy_midi" alt="stars/github repo"></a>
<a href="https://github.com/BlackPhlox/bevy_midi/actions/workflows/master.yml"><img src="https://github.com/BlackPhlox/bevy_midi/actions/workflows/master.yml/badge.svg" alt="github actions"></a>
<a href="https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking"><img src="https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue" alt="tracking bevy release branch"></a>
</div>
</br>

A bevy plugin using [midir](https://github.com/Boddlnagg/midir) and [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam). This plugin allows you to read or write midi data for a selected midi input.</br>(Currently, write is not implemented yet)
## Showcase

Run the examples using:</br>

Terminal only: 
`cargo run --release --example minimal`</br>
Virtual Piano: `cargo run --release --example piano`

Running the piano example:</br>

https://user-images.githubusercontent.com/25123512/122971334-3bae6100-d38f-11eb-9605-4c314b088ff2.mp4

Notice: Sustain is not handled by the example

Browser support: Still work in progress.
# Setup

See examples

# Support
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

|bevy|bevy_midi|
|---|---|
|0.5|0.1.X|
|0.5|0.2.X|
|0.6|0.3.X|
|0.7|0.4.X|
|0.8|0.5.X|

# Licensing
The project is under dual license MIT and Apache 2.0, so joink to your hearts content, just remember the license agreements.

# Contributing
Yes this project is still very much WIP, so PRs are very welcome
