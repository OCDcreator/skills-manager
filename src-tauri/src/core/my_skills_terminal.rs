use super::{
    error::AppError,
    my_skills_repo::{
        self, LinkImportPreparation, LinkImportProcessOutput, MySkillsTerminalLaunch,
        MySkillsWorkspaceLinkImportResult,
    },
    skill_store::SkillStore,
};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

const OUTPUT_EVENT: &str = "my-skills-terminal-output";
const STATE_EVENT: &str = "my-skills-terminal-state";
const EXIT_EVENT: &str = "my-skills-terminal-exit";
const DEFAULT_ROWS: u16 = 28;
const DEFAULT_COLS: u16 = 110;

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsTerminalSession {
    pub session_id: String,
    pub source_url: String,
    pub path: String,
    pub command: String,
    pub command_preview: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsTerminalState {
    pub session: MySkillsTerminalSession,
    pub running: bool,
    pub exit_code: Option<u32>,
    pub exit_signal: Option<String>,
    pub last_activity_at: i64,
    pub error: Option<String>,
    pub result: Option<MySkillsWorkspaceLinkImportResult>,
    pub transcript: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsTerminalOutputEvent {
    pub session_id: String,
    pub chunk: String,
    pub received_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsTerminalStateEvent {
    pub state: MySkillsTerminalState,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsTerminalExitEvent {
    pub state: MySkillsTerminalState,
}

#[derive(Default)]
pub struct MySkillsTerminalManager {
    current: Mutex<Option<Arc<TerminalSession>>>,
}

impl MySkillsTerminalManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_link_import(
        &self,
        app_handle: AppHandle,
        store: Arc<SkillStore>,
        source_url: &str,
    ) -> Result<MySkillsTerminalState> {
        if let Some(existing) = self.current_session() {
            let status = existing.snapshot();
            if status.running {
                return Ok(status);
            }
        }

        let prepared = my_skills_repo::prepare_link_import(&store, source_url)?;
        let launch = my_skills_repo::build_link_import_terminal_launch(&prepared)?;
        let session = spawn_terminal_session(app_handle, store, prepared, launch)?;
        let snapshot = session.snapshot();
        self.replace_session(session);
        Ok(snapshot)
    }

    pub fn get_status(&self) -> Option<MySkillsTerminalState> {
        self.current_session().map(|session| session.snapshot())
    }

    pub fn write_input(&self, session_id: &str, input: &str) -> Result<()> {
        let session = self.require_session(session_id)?;
        session.write_input(input.as_bytes())
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let session = self.require_session(session_id)?;
        session.resize(cols.max(1), rows.max(1))
    }

    pub fn interrupt(&self, session_id: &str) -> Result<()> {
        let session = self.require_session(session_id)?;
        session.interrupt()
    }

    pub fn close(&self, session_id: &str) -> Result<()> {
        let session = self.require_session(session_id)?;
        session.close()?;
        let mut guard = self.current.lock().map_err(|_| anyhow!("Terminal session lock poisoned"))?;
        if guard
            .as_ref()
            .is_some_and(|current| current.session_id() == session_id)
        {
            *guard = None;
        }
        Ok(())
    }

    fn replace_session(&self, session: Arc<TerminalSession>) {
        if let Ok(mut guard) = self.current.lock() {
            *guard = Some(session);
        }
    }

    fn current_session(&self) -> Option<Arc<TerminalSession>> {
        self.current.lock().ok()?.clone()
    }

    fn require_session(&self, session_id: &str) -> Result<Arc<TerminalSession>> {
        let Some(session) = self.current_session() else {
            return Err(anyhow!("OpenCode terminal session not found"));
        };
        if session.session_id() != session_id {
            return Err(anyhow!("OpenCode terminal session changed; refresh and try again"));
        }
        Ok(session)
    }
}

struct TerminalSession {
    state: Mutex<MySkillsTerminalState>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    killer: Mutex<Option<Box<dyn portable_pty::ChildKiller + Send + Sync>>>,
}

impl TerminalSession {
    fn new(
        session_id: String,
        launch: &MySkillsTerminalLaunch,
        writer: Box<dyn Write + Send>,
        master: Box<dyn portable_pty::MasterPty + Send>,
        killer: Box<dyn portable_pty::ChildKiller + Send + Sync>,
    ) -> Arc<Self> {
        let now = Utc::now().timestamp_millis();
        Arc::new(Self {
            state: Mutex::new(MySkillsTerminalState {
                session: MySkillsTerminalSession {
                    session_id,
                    source_url: launch.source_url.clone(),
                    path: launch.path.clone(),
                    command: launch.command.clone(),
                    command_preview: launch.command_preview.clone(),
                    started_at: now,
                },
                running: true,
                exit_code: None,
                exit_signal: None,
                last_activity_at: now,
                error: None,
                result: None,
                transcript: String::new(),
            }),
            writer: Mutex::new(Some(writer)),
            master: Mutex::new(master),
            killer: Mutex::new(Some(killer)),
        })
    }

    fn session_id(&self) -> String {
        self.state
            .lock()
            .map(|state| state.session.session_id.clone())
            .unwrap_or_default()
    }

    fn snapshot(&self) -> MySkillsTerminalState {
        self.state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| MySkillsTerminalState {
                session: MySkillsTerminalSession {
                    session_id: String::new(),
                    source_url: String::new(),
                    path: String::new(),
                    command: String::new(),
                    command_preview: String::new(),
                    started_at: 0,
                },
                running: false,
                exit_code: None,
                exit_signal: None,
                last_activity_at: 0,
                error: Some("Terminal session lock poisoned".to_string()),
                result: None,
                transcript: String::new(),
            })
    }

    fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let mut writer_guard = self
            .writer
            .lock()
            .map_err(|_| anyhow!("Terminal writer lock poisoned"))?;
        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| anyhow!("OpenCode terminal is no longer writable"))?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let master = self
            .master
            .lock()
            .map_err(|_| anyhow!("Terminal resize lock poisoned"))?;
        master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    fn interrupt(&self) -> Result<()> {
        if self.write_input(&[3]).is_ok() {
            return Ok(());
        }

        let mut killer_guard = self
            .killer
            .lock()
            .map_err(|_| anyhow!("Terminal interrupt lock poisoned"))?;
        let killer = killer_guard
            .as_mut()
            .ok_or_else(|| anyhow!("OpenCode terminal is not running"))?;
        killer.kill()?;
        Ok(())
    }

    fn close(&self) -> Result<()> {
        if self.snapshot().running {
            let mut killer_guard = self
                .killer
                .lock()
                .map_err(|_| anyhow!("Terminal close lock poisoned"))?;
            if let Some(killer) = killer_guard.as_mut() {
                killer.kill()?;
            }
        }
        Ok(())
    }

    fn append_output(&self, chunk: &str) -> i64 {
        let mut state = self.state.lock().expect("terminal state lock poisoned");
        state.transcript.push_str(chunk);
        state.last_activity_at = Utc::now().timestamp_millis();
        state.last_activity_at
    }

    fn finish(
        &self,
        exit_code: u32,
        exit_signal: Option<String>,
        result: Option<MySkillsWorkspaceLinkImportResult>,
        error: Option<String>,
    ) -> MySkillsTerminalState {
        let mut state = self.state.lock().expect("terminal state lock poisoned");
        state.running = false;
        state.exit_code = Some(exit_code);
        state.exit_signal = exit_signal;
        state.error = error;
        state.result = result;
        state.last_activity_at = Utc::now().timestamp_millis();
        state.clone()
    }

    fn transcript(&self) -> String {
        self.state
            .lock()
            .map(|state| state.transcript.clone())
            .unwrap_or_default()
    }
}

fn spawn_terminal_session(
    app_handle: AppHandle,
    store: Arc<SkillStore>,
    prepared: LinkImportPreparation,
    launch: MySkillsTerminalLaunch,
) -> Result<Arc<TerminalSession>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: DEFAULT_ROWS,
        cols: DEFAULT_COLS,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut builder = CommandBuilder::new(&launch.program);
    for arg in &launch.args {
        builder.arg(arg);
    }
    builder.cwd(&launch.path);
    if let Some(path_env) = launch.path_env.clone() {
        builder.env("PATH", path_env);
    }
    builder.env("NO_COLOR", "1");

    let mut child = pair
        .slave
        .spawn_command(builder)
        .with_context(|| format!("Failed to start OpenCode terminal in {}", launch.path))?;

    drop(pair.slave);

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let killer = child.clone_killer();
    let session = TerminalSession::new(uuid::Uuid::new_v4().to_string(), &launch, writer, pair.master, killer);
    if !launch.startup_banner.is_empty() {
        session.append_output(&launch.startup_banner);
    }
    emit_state(&app_handle, &session.snapshot());

    let reader_session = session.clone();
    let reader_app = app_handle.clone();
    let reader_handle = thread::spawn(move || read_terminal_output(reader, reader_session, reader_app));

    let wait_session = session.clone();
    thread::spawn(move || {
        let exit_status = child.wait();
        let _ = reader_handle.join();

        let (exit_code, exit_signal, success, exit_label) = match exit_status {
            Ok(status) => (
                status.exit_code(),
                status.signal().map(str::to_string),
                status.success(),
                format!("{}", status),
            ),
            Err(error) => (1, None, false, error.to_string()),
        };

        let process_output = LinkImportProcessOutput {
            success,
            text: wait_session.transcript(),
            exit_label,
        };

        let (result, error) = match my_skills_repo::finalize_link_import(&store, &prepared, &process_output) {
            Ok(result) => (Some(result), None),
            Err(error) => (None, Some(error.to_string())),
        };

        let final_state = wait_session.finish(exit_code, exit_signal, result, error);
        emit_state(&app_handle, &final_state);
        emit_exit(&app_handle, &final_state);
    });

    Ok(session)
}

fn read_terminal_output(
    mut reader: Box<dyn Read + Send>,
    session: Arc<TerminalSession>,
    app_handle: AppHandle,
) {
    let mut decoder = Utf8ChunkDecoder::default();
    let mut buffer = [0u8; 4096];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                let remaining = decoder.finish();
                if !remaining.is_empty() {
                    let received_at = session.append_output(&remaining);
                    emit_output(&app_handle, &session.session_id(), &remaining, received_at);
                }
                break;
            }
            Ok(size) => {
                let chunk = decoder.push(&buffer[..size]);
                if !chunk.is_empty() {
                    let received_at = session.append_output(&chunk);
                    emit_output(&app_handle, &session.session_id(), &chunk, received_at);
                }
            }
            Err(_) => {
                let remaining = decoder.finish();
                if !remaining.is_empty() {
                    let received_at = session.append_output(&remaining);
                    emit_output(&app_handle, &session.session_id(), &remaining, received_at);
                }
                break;
            }
        }
    }
}

fn emit_output(app_handle: &AppHandle, session_id: &str, chunk: &str, received_at: i64) {
    let _ = app_handle.emit(
        OUTPUT_EVENT,
        MySkillsTerminalOutputEvent {
            session_id: session_id.to_string(),
            chunk: chunk.to_string(),
            received_at,
        },
    );
}

fn emit_state(app_handle: &AppHandle, state: &MySkillsTerminalState) {
    let _ = app_handle.emit(
        STATE_EVENT,
        MySkillsTerminalStateEvent {
            state: state.clone(),
        },
    );
}

fn emit_exit(app_handle: &AppHandle, state: &MySkillsTerminalState) {
    let _ = app_handle.emit(
        EXIT_EVENT,
        MySkillsTerminalExitEvent {
            state: state.clone(),
        },
    );
}

#[derive(Default)]
struct Utf8ChunkDecoder {
    pending: Vec<u8>,
}

impl Utf8ChunkDecoder {
    fn push(&mut self, chunk: &[u8]) -> String {
        self.pending.extend_from_slice(chunk);
        self.decode_available()
    }

    fn finish(&mut self) -> String {
        if self.pending.is_empty() {
            return String::new();
        }
        let remaining = String::from_utf8_lossy(&self.pending).to_string();
        self.pending.clear();
        remaining
    }

    fn decode_available(&mut self) -> String {
        let mut out = String::new();

        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(valid) => {
                    out.push_str(valid);
                    self.pending.clear();
                    break;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    if valid_up_to > 0 {
                        let valid = std::str::from_utf8(&self.pending[..valid_up_to])
                            .expect("UTF-8 valid_up_to must be valid");
                        out.push_str(valid);
                        self.pending.drain(..valid_up_to);
                    }

                    match error.error_len() {
                        Some(error_len) => {
                            out.push_str(&String::from_utf8_lossy(&self.pending[..error_len]));
                            self.pending.drain(..error_len);
                        }
                        None => break,
                    }
                }
            }
        }

        out
    }
}

pub fn classify_terminal_error(error: impl std::fmt::Display) -> AppError {
    AppError::internal(error)
}
