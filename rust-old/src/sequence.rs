//! Call sequencing: bouts, phrases, and chorus synthesis.
//!
//! Provides higher-level patterns above individual vocalizations:
//! - **Call bouts**: Repeated calls with species-specific inter-call intervals
//! - **Phrases**: Ordered sequences of different vocalizations (bird song patterns)
//! - **Chorus**: Multiple simultaneous voices with timing variation

use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::vocalization::{CallIntent, Vocalization};
use crate::voice::CreatureVoice;

/// A single element in a call sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallElement {
    /// The vocalization to produce.
    pub vocalization: Vocalization,
    /// Duration of this call in seconds.
    pub duration: f32,
    /// Silence after this call in seconds.
    pub gap: f32,
}

/// A call bout: repeated instances of the same vocalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallBout {
    /// The vocalization to repeat.
    pub vocalization: Vocalization,
    /// Number of repetitions.
    pub count: u32,
    /// Duration of each call in seconds.
    pub call_duration: f32,
    /// Inter-call interval in seconds.
    pub interval: f32,
    /// Behavioral intent.
    pub intent: CallIntent,
}

impl CallBout {
    /// Synthesizes the entire bout as a single buffer.
    pub fn synthesize(&self, voice: &CreatureVoice, sample_rate: f32) -> Result<Vec<f32>> {
        let gap_samples = (self.interval * sample_rate) as usize;
        let mut output = Vec::new();

        for i in 0..self.count {
            let call = voice.vocalize_with_intent(
                &self.vocalization,
                self.intent,
                sample_rate,
                self.call_duration,
            )?;
            output.extend_from_slice(&call);
            if i < self.count - 1 {
                output.resize(output.len() + gap_samples, 0.0);
            }
        }
        Ok(output)
    }
}

/// A call phrase: ordered sequence of different vocalizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallPhrase {
    /// The elements in this phrase.
    pub elements: Vec<CallElement>,
    /// Behavioral intent applied to all elements.
    pub intent: CallIntent,
}

impl CallPhrase {
    /// Synthesizes the entire phrase as a single buffer.
    pub fn synthesize(&self, voice: &CreatureVoice, sample_rate: f32) -> Result<Vec<f32>> {
        let mut output = Vec::new();

        for element in &self.elements {
            let call = voice.vocalize_with_intent(
                &element.vocalization,
                self.intent,
                sample_rate,
                element.duration,
            )?;
            output.extend_from_slice(&call);
            let gap_samples = (element.gap * sample_rate) as usize;
            output.resize(output.len() + gap_samples, 0.0);
        }
        Ok(output)
    }
}

/// Synthesizes a chorus of multiple voices with timing variation.
///
/// Each voice starts at a slightly different time (jittered by `timing_spread` seconds).
/// All voices are summed and normalized.
pub fn synthesize_chorus(
    voices: &[CreatureVoice],
    vocalization: &Vocalization,
    intent: CallIntent,
    sample_rate: f32,
    duration: f32,
    timing_spread: f32,
) -> Result<Vec<f32>> {
    if voices.is_empty() {
        return Ok(Vec::new());
    }

    let max_offset_samples = (timing_spread * sample_rate) as usize;
    let base_samples = (duration * sample_rate) as usize;
    // Total length includes the spread for late-starting voices
    let total_len = base_samples + max_offset_samples * 2;
    let mut mix = vec![0.0f32; total_len];

    let mut rng = crate::rng::Rng::new(voices.len() as u64 * 7919);
    let voice_count = voices.len() as f32;

    for voice in voices {
        let call = voice.vocalize_with_intent(vocalization, intent, sample_rate, duration)?;
        // Random offset within timing_spread
        let offset = (rng.next_u32() as usize) % (max_offset_samples.max(1) * 2);

        for (i, &s) in call.iter().enumerate() {
            let idx = offset + i;
            if idx < total_len {
                mix[idx] += s;
            }
        }
    }

    // Normalize by voice count to prevent clipping
    let scale = 1.0 / voice_count.sqrt();
    for s in &mut mix {
        *s *= scale;
    }

    Ok(mix)
}
