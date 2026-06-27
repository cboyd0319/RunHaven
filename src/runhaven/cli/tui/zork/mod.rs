//! Hidden Zork I easter egg for the TUI.
//!
//! Boundary: the bundled story is treated as untrusted bytes, but the engine is
//! kept in process and can only exchange text through `ZorkIo`. Disk saves are
//! constrained to one RunHaven-owned cache file and validated before parsing.

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use sha2::{Digest, Sha256};

use super::theme::{Palette, TuiSettings};
use super::widgets::{
    layout, push_wrapped_line, render_footer, render_screen_body, render_screen_title,
    truncate_to_width,
};
use crate::runhaven::support::paths::{write_private_atomic, zork_save_path};

mod zmachine;

use zmachine::instructions::{InputStreamEnum, WindowLayout};
use zmachine::interfaces::{QuetzalData, TerpIO};
use zmachine::quetzal::{QuetzalRestoreHandler, queztal_data_to_bytes};
use zmachine::vm::{VM, VMLoadError, VMState};

const STORY_BYTES: &[u8] = include_bytes!("../../../../../third_party/zork1/COMPILED/zork1.z3");
const STORY_LEN: usize = 86_838;
const STORY_SHA256: [u8; 32] = [
    0x37, 0x08, 0x49, 0x66, 0x47, 0x7d, 0xff, 0x67, 0x92, 0x82, 0xde, 0x42, 0x97, 0x4b, 0x20, 0x77,
    0x15, 0x6b, 0x1b, 0xd6, 0x8f, 0xad, 0x92, 0xa6, 0x5d, 0x4e, 0xa9, 0x4d, 0x8e, 0xb6, 0x4d, 0x79,
];
const MAX_STEPS_PER_DRIVE: usize = 20_000;
const MAX_SAVE_BYTES: usize = 512 * 1024;
const MIN_QUETZAL_BYTES: usize = 32;

pub(super) struct ZorkState {
    vm: Option<VM>,
    io: ZorkIo,
    input: String,
    save_path: PathBuf,
    message: Option<String>,
}

impl ZorkState {
    pub(super) fn new() -> Self {
        let mut state = Self {
            vm: None,
            io: ZorkIo::default(),
            input: String::new(),
            save_path: zork_save_path(),
            message: None,
        };
        state.reset();
        state
    }

    #[cfg(test)]
    pub(super) fn with_save_path(save_path: PathBuf) -> Self {
        let mut state = Self {
            vm: None,
            io: ZorkIo::default(),
            input: String::new(),
            save_path,
            message: None,
        };
        state.reset();
        state
    }

    pub(super) fn reset(&mut self) {
        self.io = ZorkIo::default();
        self.input.clear();
        self.message = None;
        if let Err(error) = validate_bundled_story() {
            self.vm = None;
            self.message = Some(error.to_string());
            return;
        }

        match VM::create_from_story_bytes(STORY_BYTES.to_vec(), false, true) {
            Ok(mut vm) => {
                drive_vm(&mut vm, &mut self.io, &self.save_path, &mut self.message);
                self.vm = Some(vm);
            }
            Err(error) => {
                self.vm = None;
                self.message = Some(format!("Zork story load failed: {}", load_error(&error)));
            }
        }
    }

    pub(super) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => return true,
            KeyCode::Enter => self.submit_input(),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char('r') if self.is_done() => self.reset(),
            KeyCode::Char(ch) if !ch.is_control() => self.input.push(ch),
            _ => {}
        }
        false
    }

    fn submit_input(&mut self) {
        if self.vm.is_none() || self.is_done() || !self.io.waiting_for_input() {
            return;
        }
        let input = self.input.trim_end().to_string();
        self.io.echo_input(&input);
        self.io.queue_input(input);
        self.input.clear();
        if let Some(vm) = &mut self.vm {
            drive_vm(vm, &mut self.io, &self.save_path, &mut self.message);
        }
    }

    fn is_done(&self) -> bool {
        self.vm
            .as_ref()
            .is_some_and(|vm| matches!(vm.get_state(), VMState::Quit | VMState::Error))
    }

    fn status_line(&self) -> String {
        if self.io.status_left.is_empty() {
            "Zork I".to_string()
        } else if self.io.status_right.is_empty() {
            self.io.status_left.clone()
        } else {
            format!("{}  {}", self.io.status_left, self.io.status_right)
        }
    }

    fn prompt(&self) -> String {
        format!("> {}", self.input)
    }

    fn footer_tip(&self) -> &'static str {
        "Runs in-process from bundled MIT story bytes; save/restore uses one private RunHaven cache slot."
    }

    #[cfg(test)]
    pub(super) fn transcript(&self) -> &str {
        &self.io.text
    }

    #[cfg(test)]
    pub(super) fn save_path(&self) -> &Path {
        &self.save_path
    }

    #[cfg(test)]
    pub(super) fn set_input_for_test(&mut self, input: &str) {
        self.input = input.to_string();
    }

    #[cfg(test)]
    pub(super) fn submit_for_test(&mut self) {
        self.submit_input();
    }
}

pub(super) fn render_zork(
    frame: &mut Frame,
    zork: &ZorkState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(
        frame,
        header,
        "Zork I: The Great Underground Empire",
        settings,
        palette,
    );

    let [status_area, transcript_area, prompt_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(body);

    let status = Line::from(vec![Span::styled(
        truncate_to_width(&zork.status_line(), status_area.width as usize),
        palette.accent(),
    )]);
    frame.render_widget(Paragraph::new(status), status_area);

    render_screen_body(
        frame,
        transcript_area,
        " Story ",
        story_lines(
            zork,
            transcript_area.width as usize,
            transcript_area.height as usize,
            palette,
        ),
        settings,
        palette,
    );

    let mut prompt_lines = vec![Line::styled(
        truncate_to_width(&zork.prompt(), prompt_area.width.saturating_sub(2) as usize),
        palette.text(),
    )];
    if let Some(message) = &zork.message {
        prompt_lines.push(Line::styled(
            truncate_to_width(message, prompt_area.width.saturating_sub(2) as usize),
            palette.muted(),
        ));
    } else if zork.is_done() {
        prompt_lines.push(Line::styled(
            "Game ended. Press r to restart or esc home.",
            palette.muted(),
        ));
    }
    let mut prompt = Paragraph::new(Text::from(prompt_lines)).style(palette.text());
    if !settings.line_mode {
        prompt = prompt.block(
            Block::bordered()
                .title(" Command ")
                .border_style(palette.border()),
        );
    }
    frame.render_widget(prompt, prompt_area);

    render_footer(
        frame,
        footer,
        "type command · enter send · esc home · save/restore use RunHaven slot",
        zork.footer_tip(),
        palette,
    );
}

fn story_lines(
    zork: &ZorkState,
    width: usize,
    height: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    if let Some(message) = &zork.message
        && zork.vm.is_none()
    {
        let mut lines = Vec::new();
        push_wrapped_line(&mut lines, message, palette.accent(), width);
        return lines;
    }

    let mut lines = Vec::new();
    for raw in zork.io.text.lines() {
        if raw.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            push_wrapped_line(&mut lines, raw, palette.text(), width);
        }
    }
    let available = height.saturating_sub(2).max(1);
    if lines.len() > available {
        lines[lines.len() - available..].to_vec()
    } else {
        lines
    }
}

fn drive_vm(vm: &mut VM, io: &mut ZorkIo, save_path: &Path, message: &mut Option<String>) {
    for _ in 0..MAX_STEPS_PER_DRIVE {
        match vm.get_state() {
            VMState::Running | VMState::WaitingForInput(_, _, _) => vm.tick(io),
            VMState::SavePrompt(success_pc, failure_pc) => {
                match save_game(vm, save_path) {
                    Ok(()) => {
                        *message = Some(format!("Saved Zork session to {}.", save_path.display()));
                        vm.set_pc(success_pc);
                    }
                    Err(error) => {
                        *message = Some(format!("Save failed: {error}"));
                        vm.set_pc(failure_pc);
                    }
                }
                vm.set_state(VMState::Running);
            }
            VMState::RestorePrompt => {
                match restore_game(vm, save_path) {
                    Ok(()) => {
                        *message = Some(format!(
                            "Restored Zork session from {}.",
                            save_path.display()
                        ));
                    }
                    Err(error) => {
                        *message = Some(format!("Restore failed: {error}"));
                    }
                }
                vm.set_state(VMState::Running);
            }
            VMState::TranscriptPrompt
            | VMState::CommandOutputPrompt
            | VMState::CommandInputPrompt => {
                *message = Some(
                    "Transcript and command-file streams are disabled in RunHaven.".to_string(),
                );
                vm.set_state(VMState::Running);
            }
            VMState::Quit | VMState::Error => return,
            VMState::Initializing => {
                vm.set_state(VMState::Running);
            }
        }
        if matches!(
            vm.get_state(),
            VMState::WaitingForInput(_, _, _) | VMState::Quit | VMState::Error
        ) {
            return;
        }
    }
    *message = Some("Z-machine step limit reached; command paused.".to_string());
}

fn save_game(vm: &VM, save_path: &Path) -> Result<()> {
    let bytes = queztal_data_to_bytes(vm.get_quetzal_data(true));
    validate_save_bytes(&bytes).context("generated invalid Quetzal save")?;
    write_private_atomic(save_path, &bytes)
}

fn restore_game(vm: &mut VM, save_path: &Path) -> Result<()> {
    let bytes = read_validated_save(save_path)?;
    let data = catch_unwind(AssertUnwindSafe(|| {
        QuetzalRestoreHandler::from_bytes(bytes)
    }))
    .map_err(|_| anyhow!("save parser rejected malformed data"))?
    .map_err(|error| anyhow!("{error}"))?;
    validate_quetzal_data(&data)?;
    let mut restored_vm = VM::create_from_story_bytes(STORY_BYTES.to_vec(), false, true)
        .map_err(|error| anyhow!("could not initialize restore VM: {}", load_error(&error)))?;
    catch_unwind(AssertUnwindSafe(|| restored_vm.restore_game(data)))
        .map_err(|_| anyhow!("save restore rejected malformed data"))?
        .map_err(|error| anyhow!("{error:?}"))?;
    *vm = restored_vm;
    Ok(())
}

fn read_validated_save(save_path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(save_path)
        .with_context(|| format!("no save file at {}", save_path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("save slot is a symlink");
    }
    if !metadata.is_file() {
        bail!("save slot is not a regular file");
    }
    if metadata.len() > MAX_SAVE_BYTES as u64 {
        bail!("save file is too large");
    }
    let bytes =
        fs::read(save_path).with_context(|| format!("could not read {}", save_path.display()))?;
    validate_save_bytes(&bytes)?;
    Ok(bytes)
}

fn validate_save_bytes(bytes: &[u8]) -> Result<()> {
    if bytes.len() < MIN_QUETZAL_BYTES {
        bail!("save file is too small");
    }
    if bytes.len() > MAX_SAVE_BYTES {
        bail!("save file is too large");
    }
    if &bytes[0..4] != b"FORM" {
        bail!("save file is not an IFF FORM");
    }
    let form_len = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let expected_len = form_len
        .checked_add(8)
        .and_then(|len| len.checked_add(form_len % 2))
        .ok_or_else(|| anyhow!("save file length header overflows"))?;
    if expected_len != bytes.len() {
        bail!("save file length header is invalid");
    }
    if &bytes[8..12] != b"IFZS" {
        bail!("save file is not a Quetzal IFZS save");
    }
    let mut offset = 12;
    let mut saw_header = false;
    let mut saw_memory = false;
    let mut saw_stack = false;
    let form_end = 8 + form_len;
    while offset + 8 <= form_end {
        let id = &bytes[offset..offset + 4];
        let len = u32::from_be_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let data_start = offset + 8;
        let data_end = data_start
            .checked_add(len)
            .ok_or_else(|| anyhow!("save chunk length overflow"))?;
        if data_end > form_end {
            bail!("save chunk extends past end of file");
        }
        let chunk_data = &bytes[data_start..data_end];
        match id {
            b"IFhd" => {
                if saw_header || len != 13 {
                    bail!("save header chunk is invalid");
                }
                saw_header = true;
            }
            b"CMem" | b"UMem" => {
                if saw_memory || chunk_data.is_empty() || chunk_data.len() > MAX_SAVE_BYTES {
                    bail!("save memory chunk is invalid");
                }
                saw_memory = true;
            }
            b"Stks" => {
                if saw_stack {
                    bail!("save stack chunk is duplicated");
                }
                validate_stks_chunk(chunk_data)?;
                saw_stack = true;
            }
            _ => bail!("save file contains unsupported chunk"),
        }
        offset = data_end + (len % 2);
    }
    if offset != form_end {
        bail!("save chunks do not align with the FORM length");
    }
    if !saw_header || !saw_memory || !saw_stack {
        bail!("save file is missing required Quetzal chunks");
    }
    Ok(())
}

fn validate_stks_chunk(data: &[u8]) -> Result<()> {
    let mut offset = 0;
    while offset < data.len() {
        if offset + 8 > data.len() {
            bail!("save stack frame is truncated");
        }
        let flags = data[offset + 3];
        let locals = (flags & 0x0f) as usize;
        let stack_words = u16::from_be_bytes([data[offset + 6], data[offset + 7]]) as usize;
        let locals_len = locals
            .checked_mul(2)
            .ok_or_else(|| anyhow!("locals overflow"))?;
        let stack_len = stack_words
            .checked_mul(2)
            .ok_or_else(|| anyhow!("stack overflow"))?;
        let frame_len = 8usize
            .checked_add(locals_len)
            .and_then(|len| len.checked_add(stack_len))
            .ok_or_else(|| anyhow!("stack frame length overflow"))?;
        if offset + frame_len > data.len() {
            bail!("save stack frame extends past chunk");
        }
        offset += frame_len;
    }
    Ok(())
}

fn validate_quetzal_data(data: &QuetzalData) -> Result<()> {
    if data.release_number == 0 || data.serial == [0; 6] || data.checksum == 0 {
        bail!("save header is incomplete");
    }
    if data.data.is_empty() || data.data.len() > MAX_SAVE_BYTES {
        bail!("save memory payload is invalid");
    }
    Ok(())
}

fn validate_bundled_story() -> Result<()> {
    if STORY_BYTES.len() != STORY_LEN {
        bail!("bundled Zork story length changed");
    }
    let digest = Sha256::digest(STORY_BYTES);
    if digest[..] != STORY_SHA256[..] {
        bail!("bundled Zork story SHA-256 changed");
    }
    Ok(())
}

fn load_error(error: &VMLoadError) -> String {
    match error {
        VMLoadError::StoryFileTooSmall(size) => format!("story too small ({size} bytes)"),
        VMLoadError::StoryFileTooLarge(size) => format!("story too large ({size} bytes)"),
        VMLoadError::UnsupportedVersion() => "unsupported Z-code version".to_string(),
        VMLoadError::ChecksumMismatch() => "checksum mismatch".to_string(),
        VMLoadError::LengthMismatch() => "length mismatch".to_string(),
        VMLoadError::InterpreterError(error) => error.clone(),
    }
}

struct ZorkIo {
    text: String,
    pending_input: Option<String>,
    waiting_for_line: bool,
    status_left: String,
    status_right: String,
    input_stream: InputStreamEnum,
    screen_output_active: bool,
    transcript_active: bool,
    command_output_active: bool,
    window: WindowLayout,
    upper_window_lines: usize,
}

impl Default for ZorkIo {
    fn default() -> Self {
        Self {
            text: String::new(),
            pending_input: None,
            waiting_for_line: false,
            status_left: String::new(),
            status_right: String::new(),
            input_stream: InputStreamEnum::Keyboard,
            screen_output_active: true,
            transcript_active: false,
            command_output_active: false,
            window: WindowLayout::Lower,
            upper_window_lines: 0,
        }
    }
}

impl ZorkIo {
    fn echo_input(&mut self, input: &str) {
        self.text.push_str(input);
        self.text.push('\n');
    }

    fn queue_input(&mut self, input: String) {
        self.pending_input = Some(input);
        self.waiting_for_line = false;
    }
}

impl TerpIO for ZorkIo {
    fn print_char(&mut self, c: char) {
        self.text.push(c);
    }

    fn draw_status(&mut self, left: &str, right: &str) {
        self.status_left.clear();
        self.status_left.push_str(left);
        self.status_right.clear();
        self.status_right.push_str(right);
    }

    fn split_window(&mut self, lines: usize) {
        self.upper_window_lines = lines;
    }

    fn set_window(&mut self, window: WindowLayout) {
        self.window = window;
    }

    fn print_to_screen(&mut self, s: &str) {
        if self.screen_output_active {
            self.text.push_str(s);
        }
    }

    fn waiting_for_input(&self) -> bool {
        self.waiting_for_line
    }

    fn last_input(&mut self) -> String {
        self.pending_input.take().unwrap_or_default()
    }

    fn wait_for_line(&mut self, _max_input_length: usize) {
        self.waiting_for_line = true;
    }

    fn recalculate_and_redraw(&mut self, _force: bool) {}

    fn is_screen_output_active(&self) -> bool {
        self.screen_output_active
    }

    fn set_screen_output(&mut self, v: bool) {
        self.screen_output_active = v;
    }

    fn supports_transcript(&self) -> bool {
        false
    }

    fn is_transcript_active(&self) -> bool {
        self.transcript_active
    }

    fn set_transcript(&mut self, v: bool) {
        self.transcript_active = v;
    }

    fn print_to_transcript(&mut self, _s: &str) {}

    fn supports_commands_output(&self) -> bool {
        false
    }

    fn is_command_output_active(&self) -> bool {
        self.command_output_active
    }

    fn set_command_output(&mut self, v: bool) {
        self.command_output_active = v;
    }

    fn print_to_commands(&mut self, _s: &str) {}

    fn supports_commands_input(&self) -> bool {
        false
    }

    fn set_command_input(&mut self, v: bool) {
        self.input_stream = if v {
            InputStreamEnum::File
        } else {
            InputStreamEnum::Keyboard
        };
    }

    fn is_reading_from_commands(&self) -> bool {
        self.input_stream == InputStreamEnum::File
    }

    fn play_sound_effect(&mut self, _sound: u16, _effect: u16, _volume: u16) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runhaven::support::paths::TEST_ENV_LOCK;

    struct EnvGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let old = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(old) = &self.old {
                    std::env::set_var(self.key, old);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn boots_story_and_accepts_commands() {
        let state = ZorkState::new();

        assert!(state.transcript().contains("ZORK I"));
        assert!(state.transcript().contains("West of House"));
        assert!(state.status_line().contains("West of House"));
    }

    #[test]
    fn q_is_game_input_not_screen_exit() {
        let mut state = ZorkState::new();

        assert!(!state.handle_key(KeyCode::Char('q')));
        assert_eq!(state.prompt(), "> q");
    }

    #[test]
    fn save_file_round_trips_with_expected_quetzal_shape() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let dir = tempfile::tempdir().expect("tempdir");
        let _cache = EnvGuard::set("RUNHAVEN_CACHE_HOME", dir.path());
        let mut state = ZorkState::with_save_path(zork_save_path());

        state.set_input_for_test("save");
        state.submit_for_test();

        let save = fs::read(state.save_path()).expect("save bytes");
        assert!(save.starts_with(b"FORM"));
        assert_eq!(&save[8..12], b"IFZS");
        assert!(save.windows(4).any(|window| window == b"IFhd"));
        assert!(save.windows(4).any(|window| window == b"Stks"));
        assert!(
            save.windows(4)
                .any(|window| window == b"CMem" || window == b"UMem")
        );
        validate_save_bytes(&save).expect("valid save");

        state.set_input_for_test("restore");
        state.submit_for_test();
        assert!(
            state
                .message
                .as_deref()
                .is_some_and(|message| message.contains("Restored Zork session"))
        );
    }

    #[test]
    fn malformed_and_oversized_saves_are_rejected_before_parse() {
        let malformed = b"not a quetzal save";
        assert!(validate_save_bytes(malformed).is_err());

        let mut oversized = b"FORM".to_vec();
        oversized.resize(MAX_SAVE_BYTES + 1, 0);
        assert!(validate_save_bytes(&oversized).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn restore_rejects_symlink_save_slot() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target.ifzs");
        let link = dir.path().join("save.ifzs");
        fs::write(&target, b"not a quetzal save").expect("target write");
        symlink(&target, &link).expect("symlink");

        let error = read_validated_save(&link).expect_err("symlink must be rejected");
        assert!(error.to_string().contains("symlink"));
    }
}
