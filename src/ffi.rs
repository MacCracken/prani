//! C FFI — buffer-callback API for game engine middleware.
//!
//! Provides an `extern "C"` interface for creating creature voices, starting
//! vocalizations, and filling audio buffers on demand. Designed to map cleanly
//! to Wwise source plugins, FMOD DSP callbacks, and Godot `AudioStreamGenerator`.
//!
//! # Lifecycle
//!
//! ```text
//! prani_voice_create(species) → handle
//! prani_voice_set_*(handle, value)     // configure RTPC params
//! prani_stream_start(handle, ...) → stream_handle
//! prani_stream_fill(stream_handle, buf, len) → samples_written
//! prani_stream_destroy(stream_handle)
//! prani_voice_destroy(handle)
//! ```
//!
//! # Safety
//!
//! All functions that take raw pointers require non-null, valid pointers.
//! Handles are opaque pointers — misuse (double-free, use-after-free) is UB.

use alloc::boxed::Box;

use crate::species::Species;
use crate::stream::SynthStream;
use crate::vocalization::{CallIntent, Vocalization};
use crate::voice::CreatureVoice;

/// Opaque handle to a creature voice.
pub type PraniVoice = *mut CreatureVoice;

/// Opaque handle to an active synthesis stream.
pub type PraniStream = *mut SynthStream;

// ---------------------------------------------------------------------------
// Voice lifecycle
// ---------------------------------------------------------------------------

/// Creates a new creature voice for the given species index.
///
/// Species indices: 0=Wolf, 1=Dog, 2=Cat, 3=Lion, 4=Songbird, 5=Crow,
/// 6=Raptor, 7=Snake, 8=Crocodilian, 9=Cricket, 10=Bee, 11=Dragon, 12=Fantasy.
///
/// Returns a null pointer if `species_index` is out of range.
///
/// # Safety
///
/// The returned handle must be freed with [`prani_voice_destroy`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_voice_create(species_index: u32) -> PraniVoice {
    let species = match species_from_index(species_index) {
        Some(s) => s,
        None => return core::ptr::null_mut(),
    };
    Box::into_raw(Box::new(CreatureVoice::new(species)))
}

/// Destroys a creature voice and frees its memory.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by [`prani_voice_create`],
/// and must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_voice_destroy(handle: PraniVoice) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

// ---------------------------------------------------------------------------
// Voice RTPC setters
// ---------------------------------------------------------------------------

/// Sets the vocal effort (0.0 = whisper, 0.5 = normal, 1.0 = shout).
///
/// # Safety
///
/// `handle` must be a valid, non-null `PraniVoice`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_voice_set_effort(handle: PraniVoice, effort: f32) {
    if let Some(voice) = unsafe { handle.as_mut() } {
        voice.set_vocal_effort(effort);
    }
}

/// Sets the individual size scale.
///
/// # Safety
///
/// `handle` must be a valid, non-null `PraniVoice`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_voice_set_size(handle: PraniVoice, size: f32) {
    if let Some(voice) = unsafe { handle.as_mut() } {
        // Re-apply via builder clone since size affects multiple params.
        // For real-time use, we clone and replace.
        let species = voice.species();
        let new = CreatureVoice::new(species).with_size(size);
        *voice = new;
    }
}

/// Applies the Lombard effect (ambient noise level in dB SPL).
///
/// # Safety
///
/// `handle` must be a valid, non-null `PraniVoice`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_voice_apply_lombard(handle: PraniVoice, ambient_spl_db: f32) {
    if let Some(voice) = unsafe { handle.as_mut() } {
        voice.apply_lombard_effect(ambient_spl_db);
    }
}

// ---------------------------------------------------------------------------
// Stream lifecycle
// ---------------------------------------------------------------------------

/// Starts a new synthesis stream for a vocalization.
///
/// Vocalization indices: 0=Howl, 1=Bark, 2=Growl, 3=Roar, 4=Hiss, 5=Chirp,
/// 6=Trill, 7=Whine, 8=Rumble, 9=Purr, 10=Yelp, 11=Screech, 12=Stridulate, 13=Buzz.
///
/// Intent indices: 0=Alarm, 1=Territorial, 2=Mating, 3=Distress, 4=Idle, 5=Threat, 6=Social.
///
/// Returns a null pointer on error (invalid voice, unsupported vocalization, etc.).
///
/// # Safety
///
/// `voice_handle` must be a valid `PraniVoice`. The returned stream must
/// be freed with [`prani_stream_destroy`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_stream_start(
    voice_handle: PraniVoice,
    vocalization_index: u32,
    intent_index: u32,
    sample_rate: f32,
    duration: f32,
) -> PraniStream {
    let voice = match unsafe { voice_handle.as_ref() } {
        Some(v) => v,
        None => return core::ptr::null_mut(),
    };
    let vocalization = match vocalization_from_index(vocalization_index) {
        Some(v) => v,
        None => return core::ptr::null_mut(),
    };
    let intent = match intent_from_index(intent_index) {
        Some(i) => i,
        None => return core::ptr::null_mut(),
    };

    match SynthStream::new(voice.clone(), vocalization, intent, sample_rate, duration) {
        Ok(stream) => Box::into_raw(Box::new(stream)),
        Err(_) => core::ptr::null_mut(),
    }
}

/// Fills a buffer with audio samples from the stream.
///
/// Returns the number of samples written. Returns 0 when the stream is finished.
///
/// # Safety
///
/// `stream_handle` must be a valid `PraniStream`. `buffer` must point to at
/// least `buffer_len` writable `f32` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_stream_fill(
    stream_handle: PraniStream,
    buffer: *mut f32,
    buffer_len: u32,
) -> u32 {
    let stream = match unsafe { stream_handle.as_mut() } {
        Some(s) => s,
        None => return 0,
    };
    if buffer.is_null() || buffer_len == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_len as usize) };
    stream.fill_buffer(slice) as u32
}

/// Returns 1 if the stream is finished, 0 otherwise.
///
/// # Safety
///
/// `stream_handle` must be a valid `PraniStream`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_stream_is_finished(stream_handle: PraniStream) -> u32 {
    match unsafe { stream_handle.as_ref() } {
        Some(s) => u32::from(s.is_finished()),
        None => 1,
    }
}

/// Destroys a synthesis stream and frees its memory.
///
/// # Safety
///
/// `stream_handle` must be a valid pointer returned by [`prani_stream_start`],
/// and must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn prani_stream_destroy(stream_handle: PraniStream) {
    if !stream_handle.is_null() {
        drop(unsafe { Box::from_raw(stream_handle) });
    }
}

// ---------------------------------------------------------------------------
// Index mapping helpers
// ---------------------------------------------------------------------------

fn species_from_index(index: u32) -> Option<Species> {
    match index {
        0 => Some(Species::Wolf),
        1 => Some(Species::Dog),
        2 => Some(Species::Cat),
        3 => Some(Species::Lion),
        4 => Some(Species::Songbird),
        5 => Some(Species::Crow),
        6 => Some(Species::Raptor),
        7 => Some(Species::Snake),
        8 => Some(Species::Crocodilian),
        9 => Some(Species::Cricket),
        10 => Some(Species::Bee),
        11 => Some(Species::Dragon),
        12 => Some(Species::Fantasy),
        _ => None,
    }
}

fn vocalization_from_index(index: u32) -> Option<Vocalization> {
    match index {
        0 => Some(Vocalization::Howl),
        1 => Some(Vocalization::Bark),
        2 => Some(Vocalization::Growl),
        3 => Some(Vocalization::Roar),
        4 => Some(Vocalization::Hiss),
        5 => Some(Vocalization::Chirp),
        6 => Some(Vocalization::Trill),
        7 => Some(Vocalization::Whine),
        8 => Some(Vocalization::Rumble),
        9 => Some(Vocalization::Purr),
        10 => Some(Vocalization::Yelp),
        11 => Some(Vocalization::Screech),
        12 => Some(Vocalization::Stridulate),
        13 => Some(Vocalization::Buzz),
        _ => None,
    }
}

fn intent_from_index(index: u32) -> Option<CallIntent> {
    match index {
        0 => Some(CallIntent::Alarm),
        1 => Some(CallIntent::Territorial),
        2 => Some(CallIntent::Mating),
        3 => Some(CallIntent::Distress),
        4 => Some(CallIntent::Idle),
        5 => Some(CallIntent::Threat),
        6 => Some(CallIntent::Social),
        _ => None,
    }
}
