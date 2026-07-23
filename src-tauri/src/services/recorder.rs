// services/recorder.rs
//
// Records a live sequence of user intent strings and packages them into a
// `Vibemacro` ready to save. The recorder is intentionally lightweight:
// it simply captures the raw text the user typed into the CommandBar, in
// the order they typed it, so that the macro round-trip is lossless.

use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::error::CntrlError;
use crate::services::macro_format::{MacroStep, Vibemacro};

/// Shared, thread-safe recorder state.
#[derive(Debug, Default, Clone)]
pub struct Recorder {
    inner: Arc<Mutex<RecorderInner>>,
}

#[derive(Debug, Default)]
struct RecorderInner {
    recording: bool,
    session_id: Option<String>,
    steps: Vec<MacroStep>,
    started_at: Option<chrono::DateTime<Utc>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin a new recording session. Returns `Err` if already recording.
    pub fn start(&self) -> Result<String, CntrlError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;

        if inner.recording {
            return Err(CntrlError::Macro(
                "A recording is already in progress".into(),
            ));
        }

        let session_id = Uuid::new_v4().to_string();
        inner.recording = true;
        inner.session_id = Some(session_id.clone());
        inner.steps.clear();
        inner.started_at = Some(Utc::now());

        Ok(session_id)
    }

    /// Append a captured intent string to the current session.
    /// Silently no-ops if not currently recording.
    pub fn capture(&self, intent: &str, delay_ms: u64) -> Result<(), CntrlError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;

        if !inner.recording {
            return Ok(());
        }

        inner.steps.push(MacroStep {
            intent: intent.to_string(),
            delay_ms,
        });
        Ok(())
    }

    /// Stop recording and return the assembled `Vibemacro`.
    /// Returns `Err` if not currently recording.
    pub fn stop(&self, name: &str) -> Result<Vibemacro, CntrlError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;

        if !inner.recording {
            return Err(CntrlError::Macro("No recording in progress".into()));
        }

        let steps = std::mem::take(&mut inner.steps);
        inner.recording = false;
        inner.session_id = None;
        inner.started_at = None;

        if steps.is_empty() {
            return Err(CntrlError::Macro(
                "Cannot save a macro with zero steps".into(),
            ));
        }

        Ok(Vibemacro::new(name, steps))
    }

    /// Discard the current recording without producing a macro.
    pub fn cancel(&self) -> Result<(), CntrlError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;

        inner.recording = false;
        inner.session_id = None;
        inner.steps.clear();
        inner.started_at = None;
        Ok(())
    }

    /// Returns `true` if a recording is currently in progress.
    pub fn is_recording(&self) -> Result<bool, CntrlError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;
        Ok(inner.recording)
    }

    /// Returns the number of steps captured so far in the current session.
    pub fn step_count(&self) -> Result<usize, CntrlError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| CntrlError::Macro("Recorder lock poisoned".into()))?;
        Ok(inner.steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_capture_stop_round_trip() {
        let rec = Recorder::new();
        rec.start().expect("start should succeed");
        rec.capture("go to gmail.com", 0).unwrap();
        rec.capture("search for rust async", 200).unwrap();
        let mac = rec.stop("My Macro").expect("stop should succeed");
        assert_eq!(mac.name, "My Macro");
        assert_eq!(mac.steps.len(), 2);
        assert_eq!(mac.steps[1].delay_ms, 200);
    }

    #[test]
    fn double_start_is_error() {
        let rec = Recorder::new();
        rec.start().unwrap();
        assert!(rec.start().is_err());
    }

    #[test]
    fn stop_without_start_is_error() {
        let rec = Recorder::new();
        assert!(rec.stop("X").is_err());
    }

    #[test]
    fn empty_recording_is_rejected() {
        let rec = Recorder::new();
        rec.start().unwrap();
        assert!(rec.stop("Empty").is_err());
    }

    #[test]
    fn cancel_clears_state() {
        let rec = Recorder::new();
        rec.start().unwrap();
        rec.capture("go to github.com", 0).unwrap();
        rec.cancel().unwrap();
        assert!(!rec.is_recording().unwrap());
        // After cancel we can start again cleanly
        rec.start().expect("restart after cancel should work");
    }
}
