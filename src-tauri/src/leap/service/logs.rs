//! The history of recent log entries.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::models::LogEntry;

const MAX_LOG_ENTRIES: usize = 2000;

/// Recent log entries, readable from any thread.
#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<VecDeque<LogEntry>>>);

impl LogBuffer {
    pub(super) fn push(&self, entry: LogEntry) {
        if let Ok(mut entries) = self.0.lock() {
            if entries.len() == MAX_LOG_ENTRIES {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.0
            .lock()
            .map(|entries| entries.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LogSeverity, LogSource};

    #[test]
    fn log_buffer_is_bounded() {
        let buffer = LogBuffer::default();
        for i in 0..(MAX_LOG_ENTRIES + 5) {
            buffer.push(LogEntry {
                timestamp_ms: i as u64,
                severity: LogSeverity::Info,
                source: LogSource::App,
                message: String::new(),
            });
        }
        let entries = buffer.snapshot();
        assert_eq!(entries.len(), MAX_LOG_ENTRIES);
        assert_eq!(entries[0].timestamp_ms, 5);
    }
}
