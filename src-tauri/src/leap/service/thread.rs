//! The thread that owns the tracking source and drives the state machine.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::logs::LogBuffer;
use super::machine::{Command, Core, EventSink};
use crate::leap::{SourceError, TrackingSource};
use crate::models::{LogEntry, Settings};

const POLL_TIMEOUT: Duration = Duration::from_millis(20);
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const SOURCE_RETRY_INTERVAL: Duration = Duration::from_secs(5);
const POLL_ERROR_BACKOFF: Duration = Duration::from_millis(100);

/// Creates the tracking source, and is called again after a failure until one exists.
pub type SourceFactory = Box<dyn FnMut() -> Result<Box<dyn TrackingSource>, SourceError> + Send>;

/// Handle to the running service thread.
pub struct ServiceHandle {
    commands: Sender<Command>,
    logs: LogBuffer,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl ServiceHandle {
    /// Queues a command. Returns false if the service thread has stopped.
    pub fn send(&self, command: Command) -> bool {
        self.commands.send(command).is_ok()
    }

    pub fn logs(&self) -> Vec<LogEntry> {
        self.logs.snapshot()
    }

    /// Stops the thread, which closes the connection to the service.
    pub fn shutdown(&self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Ok(mut thread) = self.thread.lock() {
            if let Some(thread) = thread.take() {
                let _ = thread.join();
            }
        }
    }
}

/// Starts the thread that owns the tracking source.
pub fn spawn<K: EventSink>(
    factory: SourceFactory,
    sink: K,
    settings: Settings,
    mock: bool,
) -> std::io::Result<ServiceHandle> {
    let (commands, receiver) = mpsc::channel();
    let logs = LogBuffer::default();
    let core = Core::new(sink, logs.clone(), settings, mock);
    let thread = thread::Builder::new()
        .name("marey-tracking".to_owned())
        .spawn(move || run(core, factory, receiver))?;
    Ok(ServiceHandle {
        commands,
        logs,
        thread: Mutex::new(Some(thread)),
    })
}

/// Retries until a source exists, then serves it until shutdown.
fn run<K: EventSink>(mut core: Core<K>, mut factory: SourceFactory, commands: Receiver<Command>) {
    loop {
        match factory() {
            Ok(mut source) => return serve_source(&mut core, source.as_mut(), &commands),
            Err(error) => {
                core.set_unavailable(error.to_string());
                core.flush();
                if !wait_for_retry(&mut core, &commands) {
                    return;
                }
            }
        }
    }
}

/// Serves commands while no source exists. Returns false when the service must stop.
fn wait_for_retry<K: EventSink>(core: &mut Core<K>, commands: &Receiver<Command>) -> bool {
    let deadline = Instant::now() + SOURCE_RETRY_INTERVAL;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match commands.recv_timeout(remaining) {
            Ok(command) => {
                if !core.command(command, None) {
                    return false;
                }
                core.flush();
            }
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

/// Runs until shutdown. A source lives as long as the process, because LeapC reconnects by itself.
fn serve_source<K: EventSink>(
    core: &mut Core<K>,
    source: &mut (dyn TrackingSource + 'static),
    commands: &Receiver<Command>,
) {
    let mut last_refresh = Instant::now();
    loop {
        while let Ok(command) = commands.try_recv() {
            if !core.command(command, Some(&mut *source)) {
                return;
            }
        }

        let polled = source.poll(POLL_TIMEOUT, &mut |event| core.handle(event));
        core.note_poll_error(polled.as_ref().err());
        if polled.is_err() {
            thread::sleep(POLL_ERROR_BACKOFF);
        }
        core.run_pending(source);

        let now = Instant::now();
        if now.duration_since(last_refresh) >= REFRESH_INTERVAL {
            last_refresh = now;
            source.refresh(&mut |event| core.handle(event));
            core.run_pending(source);
            core.tick(now);
        }
        core.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use super::*;
    use crate::leap::{EventCallback, SourceEvent, StereoImage};
    use crate::models::{DeviceFlags, DeviceSummary, LinkState, PanelState, TrackingFrame};

    struct SharedSink(Arc<Mutex<Vec<PanelState>>>);

    impl EventSink for SharedSink {
        fn state(&mut self, state: &PanelState) {
            self.0.lock().unwrap().push(state.clone());
        }
        fn frame(&mut self, _: &TrackingFrame) -> bool {
            true
        }
        fn images(&mut self, _: &StereoImage<'_>) -> bool {
            true
        }
        fn log(&mut self, _: &LogEntry) {}
        fn settings(&mut self, _: &Settings) {}
    }

    struct ScriptedSource(VecDeque<SourceEvent<'static>>);

    impl TrackingSource for ScriptedSource {
        fn poll(&mut self, _: Duration, emit: EventCallback<'_>) -> Result<(), SourceError> {
            if let Some(event) = self.0.pop_front() {
                emit(event);
            }
            Ok(())
        }
        fn refresh(&mut self, _: EventCallback<'_>) {}
        fn activate(&mut self, _: u32, _: &Settings) -> Result<(), SourceError> {
            Ok(())
        }
        fn apply(&mut self, _: u32, _: &Settings) -> Result<(), SourceError> {
            Ok(())
        }
    }

    #[test]
    fn service_thread_runs_a_scripted_source_and_stops() {
        let states = Arc::new(Mutex::new(Vec::new()));
        let factory: SourceFactory = Box::new(|| {
            Ok(Box::new(ScriptedSource(VecDeque::from([
                SourceEvent::Link {
                    state: LinkState::Connected,
                    client_version: None,
                    service_version: None,
                },
                SourceEvent::DeviceAdded(DeviceSummary {
                    id: 7,
                    product: "Device 7".to_owned(),
                    pid: 3,
                    serial: "S7".to_owned(),
                    firmware: None,
                    raw_status: 0,
                    flags: DeviceFlags::default(),
                    baseline_mm: 40.0,
                    h_fov_deg: 140.0,
                    v_fov_deg: 120.0,
                    range_mm: 800.0,
                }),
            ]))))
        });
        let handle = spawn(
            factory,
            SharedSink(states.clone()),
            Settings::default(),
            false,
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut active = None;
        while active.is_none() && Instant::now() < deadline {
            let (reply, answer) = mpsc::channel();
            assert!(handle.send(Command::GetState(reply)));
            active = answer.recv().unwrap().active_device_id;
        }
        assert_eq!(active, Some(7));
        handle.shutdown();
        assert!(!states.lock().unwrap().is_empty());
    }
}
