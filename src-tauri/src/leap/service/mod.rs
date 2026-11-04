//! The background service that owns the tracking source: its state machine, the thread that runs
//! it and the log history they share.

mod logs;
mod machine;
mod thread;

pub use logs::LogBuffer;
pub use machine::{Command, Core, EventSink};
pub use thread::{spawn, ServiceHandle, SourceFactory};
