#[cfg(target_arch = "wasm32")]
use send_wrapper::SendWrapper;
use std::ops::{Deref, DerefMut};

/// A thread-safe wrapper around [`midir::MidiInputPort`] that works on all platforms.
///
/// On native platforms, this is a simple newtype wrapper around the native port type.
/// On WASM, this uses [`SendWrapper`] to provide thread safety for Bevy's ECS.
///
/// Note: On WASM, the underlying midir types are not actually Send/Sync, but we
/// safely provide these traits because WASM is single-threaded and Bevy requires
/// all resources to be Send + Sync. SendWrapper provides runtime checks to ensure
/// the value is only accessed from the original thread.
#[derive(Clone)]
pub struct MidiInputPort(
    #[cfg(not(target_arch = "wasm32"))] midir::MidiInputPort,
    #[cfg(target_arch = "wasm32")] SendWrapper<midir::MidiInputPort>,
);

// SAFETY: MidiInputPort is a wrapper around SendWrapper<midir::MidiInputPort>.
// SendWrapper ensures the wrapped value is only accessed from the thread it was created on.
// On WASM, there is only one thread, so this is always safe.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for MidiInputPort {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for MidiInputPort {}

impl MidiInputPort {
    /// Create a new thread-safe wrapper around a [`midir::MidiInputPort`].
    pub fn new(port: midir::MidiInputPort) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        return Self(port);

        #[cfg(target_arch = "wasm32")]
        return Self(SendWrapper::new(port));
    }
}

impl Deref for MidiInputPort {
    type Target = midir::MidiInputPort;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        return &self.0;

        #[cfg(target_arch = "wasm32")]
        return &self.0;
    }
}

impl DerefMut for MidiInputPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        return &mut self.0;

        #[cfg(target_arch = "wasm32")]
        return &mut self.0;
    }
}

/// A thread-safe wrapper around [`midir::MidiOutputPort`] that works on all platforms.
///
/// On native platforms, this is a simple newtype wrapper around the native port type.
/// On WASM, this uses [`SendWrapper`] to provide thread safety for Bevy's ECS.
///
/// Note: On WASM, the underlying midir types are not actually Send/Sync, but we
/// safely provide these traits because WASM is single-threaded and Bevy requires
/// all resources to be Send + Sync. SendWrapper provides runtime checks to ensure
/// the value is only accessed from the original thread.
#[derive(Clone)]
pub struct MidiOutputPort(
    #[cfg(not(target_arch = "wasm32"))] midir::MidiOutputPort,
    #[cfg(target_arch = "wasm32")] SendWrapper<midir::MidiOutputPort>,
);

// SAFETY: MidiOutputPort is a wrapper around SendWrapper<midir::MidiOutputPort>.
// SendWrapper ensures the wrapped value is only accessed from the thread it was created on.
// On WASM, there is only one thread, so this is always safe.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for MidiOutputPort {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for MidiOutputPort {}

impl MidiOutputPort {
    /// Create a new thread-safe wrapper around a [`midir::MidiOutputPort`].
    pub fn new(port: midir::MidiOutputPort) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        return Self(port);

        #[cfg(target_arch = "wasm32")]
        return Self(SendWrapper::new(port));
    }
}

impl Deref for MidiOutputPort {
    type Target = midir::MidiOutputPort;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        return &self.0;

        #[cfg(target_arch = "wasm32")]
        return &self.0;
    }
}

impl DerefMut for MidiOutputPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        return &mut self.0;

        #[cfg(target_arch = "wasm32")]
        return &mut self.0;
    }
}
