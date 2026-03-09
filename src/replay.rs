use crate::constants::{FIXED_DT, REPLAY_PATH};
use crate::entities::Direction;
use crate::input::PlayerCommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayFile {
    pub fixed_dt: f32,
    pub frames: Vec<PlayerCommand>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptFile {
    pub duration: f32,
    #[serde(default)]
    pub captures: Vec<ScriptCapture>,
    pub events: Vec<ScriptEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptCapture {
    pub at: f32,
    pub file: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ScriptEvent {
    pub at: f32,
    #[serde(default)]
    pub move_dir: Option<ScriptMove>,
    #[serde(default)]
    pub fire: Option<bool>,
    #[serde(default)]
    pub restart: Option<bool>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptMove {
    Up,
    Down,
    Left,
    Right,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ReplaySourceFile {
    Replay(ReplayFile),
    Script(ScriptFile),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayMode {
    Idle,
    Recording,
    Playback,
}

#[derive(Clone, Debug)]
pub struct ReplaySummary {
    pub label: String,
    pub frames_played: usize,
}

pub struct ReplayController {
    mode: ReplayMode,
    frames: Vec<PlayerCommand>,
    frame_index: usize,
    capture_points: Vec<ScriptCapture>,
    capture_index: usize,
    playback_label: String,
    autorun_exit: bool,
    pending_summary: Option<ReplaySummary>,
}

impl ReplayController {
    pub fn new() -> Self {
        Self {
            mode: ReplayMode::Idle,
            frames: Vec::new(),
            frame_index: 0,
            capture_points: Vec::new(),
            capture_index: 0,
            playback_label: String::new(),
            autorun_exit: false,
            pending_summary: None,
        }
    }

    pub fn configure_from_env(&mut self) -> Result<Option<String>, String> {
        if let Ok(path) = std::env::var("TIGER_AUTOPLAY_SCRIPT") {
            self.start_playback_from_path(&path, true)?;
            self.autorun_exit = true;
            return Ok(Some(format!("Autoplaying script {path}")));
        }

        if let Ok(path) = std::env::var("TIGER_AUTOPLAY_REPLAY") {
            self.start_playback_from_path(&path, true)?;
            self.autorun_exit = true;
            return Ok(Some(format!("Autoplaying replay {path}")));
        }

        if std::env::var("TIGER_AUTODEMO").is_ok() {
            self.start_demo(true);
            self.autorun_exit = true;
            return Ok(Some("Autoplaying built-in demo".to_owned()));
        }

        Ok(None)
    }

    pub fn active_mode(&self) -> ReplayMode {
        self.mode
    }

    pub fn status_label(&self) -> &'static str {
        match self.mode {
            ReplayMode::Idle => "LIVE",
            ReplayMode::Recording => "REC",
            ReplayMode::Playback => "PLAYBACK",
        }
    }

    pub fn status_detail(&self) -> String {
        match self.mode {
            ReplayMode::Idle => {
                "F6 record  F7 replay latest  F8 stop  F10 demo  env:TIGER_AUTOPLAY_SCRIPT"
                    .to_owned()
            }
            ReplayMode::Recording => format!("Recording {} frames", self.frames.len()),
            ReplayMode::Playback => {
                format!(
                    "{}  frame {}/{}  captures {}/{}",
                    self.playback_label,
                    self.frame_index.min(self.frames.len()),
                    self.frames.len(),
                    self.capture_index.min(self.capture_points.len()),
                    self.capture_points.len()
                )
            }
        }
    }

    pub fn start_recording(&mut self) {
        self.mode = ReplayMode::Recording;
        self.frames.clear();
        self.frame_index = 0;
        self.playback_label = "recording".to_owned();
        self.pending_summary = None;
    }

    pub fn stop_and_save(&mut self, path: &str) -> Result<usize, String> {
        let frame_count = self.frames.len();
        let replay = ReplayFile {
            fixed_dt: FIXED_DT,
            frames: std::mem::take(&mut self.frames),
        };
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let raw = serde_json::to_string_pretty(&replay).map_err(|err| err.to_string())?;
        fs::write(path, raw).map_err(|err| err.to_string())?;
        self.stop();
        Ok(frame_count)
    }

    pub fn start_playback_from_path(
        &mut self,
        path: &str,
        autorun_exit: bool,
    ) -> Result<(), String> {
        let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let source: ReplaySourceFile = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
        let (frames, captures) = match source {
            ReplaySourceFile::Replay(replay) => (replay.frames, Vec::new()),
            ReplaySourceFile::Script(script) => compile_script(script),
        };
        self.start_playback(frames, captures, path.to_owned(), autorun_exit);
        Ok(())
    }

    pub fn start_demo(&mut self, autorun_exit: bool) {
        self.start_playback(
            build_demo_frames(),
            Vec::new(),
            "built-in demo".to_owned(),
            autorun_exit,
        );
    }

    pub fn stop(&mut self) {
        self.mode = ReplayMode::Idle;
        self.frames.clear();
        self.frame_index = 0;
        self.capture_points.clear();
        self.capture_index = 0;
        self.playback_label.clear();
        self.autorun_exit = false;
    }

    pub fn step_command(&mut self, live_command: PlayerCommand) -> PlayerCommand {
        match self.mode {
            ReplayMode::Idle => live_command,
            ReplayMode::Recording => {
                self.frames.push(live_command);
                live_command
            }
            ReplayMode::Playback => {
                if let Some(command) = self.frames.get(self.frame_index).copied() {
                    self.frame_index += 1;
                    if self.frame_index >= self.frames.len() {
                        self.pending_summary = Some(ReplaySummary {
                            label: self.playback_label.clone(),
                            frames_played: self.frame_index,
                        });
                    }
                    command
                } else {
                    PlayerCommand::default()
                }
            }
        }
    }

    pub fn playback_finished(&self) -> bool {
        self.mode == ReplayMode::Playback && self.frame_index >= self.frames.len()
    }

    pub fn should_autorun_exit(&self) -> bool {
        self.autorun_exit && self.playback_finished()
    }

    pub fn take_summary(&mut self) -> Option<ReplaySummary> {
        self.pending_summary.take()
    }

    pub fn drain_due_captures(&mut self) -> Vec<String> {
        if self.mode != ReplayMode::Playback {
            return Vec::new();
        }

        let mut due = Vec::new();
        let playback_time = self.frame_index as f32 * FIXED_DT;
        while let Some(capture) = self.capture_points.get(self.capture_index) {
            if capture.at > playback_time {
                break;
            }
            due.push(capture.file.clone());
            self.capture_index += 1;
        }
        due
    }

    fn start_playback(
        &mut self,
        frames: Vec<PlayerCommand>,
        mut captures: Vec<ScriptCapture>,
        label: String,
        autorun_exit: bool,
    ) {
        self.mode = ReplayMode::Playback;
        self.frames = frames;
        self.frame_index = 0;
        captures.sort_by(|a, b| a.at.total_cmp(&b.at));
        self.capture_points = captures;
        self.capture_index = 0;
        self.playback_label = label;
        self.autorun_exit = autorun_exit;
        self.pending_summary = None;
    }
}

fn build_demo_frames() -> Vec<PlayerCommand> {
    let mut frames = Vec::new();
    push_frames(&mut frames, Some(Direction::Up), false, 120);
    push_frames(&mut frames, Some(Direction::Up), true, 40);
    push_frames(&mut frames, Some(Direction::Right), false, 70);
    push_frames(&mut frames, Some(Direction::Right), true, 30);
    push_frames(&mut frames, Some(Direction::Up), false, 80);
    push_frames(&mut frames, Some(Direction::Left), false, 55);
    push_frames(&mut frames, Some(Direction::Up), true, 36);
    push_frames(&mut frames, None, false, 20);
    frames
}

fn push_frames(
    frames: &mut Vec<PlayerCommand>,
    move_dir: Option<Direction>,
    fire: bool,
    count: usize,
) {
    for _ in 0..count {
        frames.push(PlayerCommand {
            move_dir,
            fire,
            restart: false,
        });
    }
}

pub fn default_replay_path() -> &'static str {
    REPLAY_PATH
}

fn compile_script(script: ScriptFile) -> (Vec<PlayerCommand>, Vec<ScriptCapture>) {
    let mut events = script.events;
    events.sort_by(|a, b| a.at.total_cmp(&b.at));

    let frame_count = (script.duration / FIXED_DT).ceil().max(0.0) as usize;
    let mut frames = Vec::with_capacity(frame_count);
    let mut command = PlayerCommand::default();
    let mut event_index = 0usize;

    for frame_idx in 0..frame_count {
        let time = frame_idx as f32 * FIXED_DT;
        while let Some(event) = events.get(event_index) {
            if event.at > time {
                break;
            }
            apply_script_event(&mut command, *event);
            event_index += 1;
        }
        frames.push(command);
    }

    (frames, script.captures)
}

fn apply_script_event(command: &mut PlayerCommand, event: ScriptEvent) {
    if let Some(move_dir) = event.move_dir {
        command.move_dir = match move_dir {
            ScriptMove::Up => Some(Direction::Up),
            ScriptMove::Down => Some(Direction::Down),
            ScriptMove::Left => Some(Direction::Left),
            ScriptMove::Right => Some(Direction::Right),
            ScriptMove::None => None,
        };
    }
    if let Some(fire) = event.fire {
        command.fire = fire;
    }
    if let Some(restart) = event.restart {
        command.restart = restart;
    }
}
