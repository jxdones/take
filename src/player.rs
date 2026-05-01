use crate::take::{Instruction, Key, KeyCombo, TakeFile};
use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use regex::Regex;
use std::io::{Read, Write};
use tokio::time::{Duration, sleep};

/// A snapshot of the terminal screen at a point in time, with how long it should be displayed.
pub struct Frame {
    pub screen: vt100::Screen,
    pub duration: Duration,
}

/// A live PTY session controlled by the player.
struct Pty {
    writer: Box<dyn Write + Send>,
    reader: Box<dyn Read + Send>,
    child: Box<dyn portable_pty::Child + Send>,
}

/// Opens a PTY, launches the given shell inside it, and returns the live session.
fn spawn_shell(shell: &str) -> Result<Pty> {
    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        cols: 80,
        rows: 24,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    let child = pair.slave.spawn_command(cmd)?;

    let writer = pair.master.take_writer()?;
    let reader = pair.master.try_clone_reader()?;

    Ok(Pty {
        writer,
        reader,
        child,
    })
}

/// Runs a `.take` script against a live shell and returns the recorded screen frames.
pub async fn play(take_file: TakeFile) -> Result<Vec<Frame>> {
    let shell = take_file.shell.as_deref().unwrap_or("zsh");
    let global_pace = take_file.pace;
    let pty = spawn_shell(shell)?;
    let mut writer = pty.writer;
    let mut reader = pty.reader;
    let mut child = pty.child;

    let cols = take_file
        .cols
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(80);
    let rows = take_file
        .rows
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(24);

    let mut vt = vt100::Parser::new(rows, cols, 0);
    let mut frames: Vec<Frame> = vec![];
    let mut hidden = false;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

    tokio::task::spawn_blocking(move || {
        let mut buf = vec![0u8; 1024];
        loop {
            let n = reader.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            let _ = tx.blocking_send(buf[..n].to_vec());
        }
    });

    for instruction in take_file.instructions {
        println!("{}", instruction);
        match instruction {
            Instruction::Sleep(duration) => {
                sleep(duration).await;
                while let Ok(bytes) = rx.try_recv() {
                    vt.process(&bytes);
                }
                if !hidden {
                    frames.push(snapshot(&vt, duration));
                }
            }
            Instruction::Type { text, pace } => {
                let effective_pace = pace.or(global_pace);

                for ch in text.chars() {
                    writer.write_all(ch.to_string().as_bytes())?;
                    writer.flush()?;
                    while let Ok(bytes) = rx.try_recv() {
                        vt.process(&bytes);
                    }
                    if let Some(p) = effective_pace {
                        sleep(p).await;
                    }
                    if !hidden {
                        let frame_duration = effective_pace.unwrap_or(Duration::from_millis(50));
                        frames.push(snapshot(&vt, frame_duration));
                    }
                }
                read_until_idle(&mut rx, &mut vt).await;
            }
            Instruction::Press(key, count) => {
                let bytes = match key {
                    Key::Enter => "\r",
                    Key::Backspace => "\x08",
                    Key::Tab => "\t",
                    Key::Up => "\x1b[A",
                    Key::Down => "\x1b[B",
                    Key::Right => "\x1b[C",
                    Key::Left => "\x1b[D",
                    Key::Space => " ",
                    Key::Delete => "\x7f",
                    Key::Escape => "\x1b",
                };

                for _ in 0..count {
                    writer.write_all(bytes.as_bytes())?;
                }
                writer.flush()?;
                read_until_idle(&mut rx, &mut vt).await;
                if !hidden {
                    frames.push(snapshot(&vt, Duration::from_millis(500)));
                }
            }
            Instruction::Ctrl { key, modifiers } => {
                let bytes = ctrl_bytes_from_keycombo(&key, modifiers.alt);
                writer.write_all(&bytes)?;
                writer.flush()?;
                read_until_idle(&mut rx, &mut vt).await;
                if !hidden {
                    frames.push(snapshot(&vt, Duration::from_millis(500)));
                }
            }
            Instruction::Expect { regex, timeout } => {
                expect_output(&mut rx, &mut vt, &regex, timeout, false).await;
            }
            Instruction::ExpectLine { regex, timeout } => {
                expect_output(&mut rx, &mut vt, &regex, timeout, true).await;
            }
            Instruction::Hide => {
                hidden = true;
            }
            Instruction::Show => {
                hidden = false;
                frames.push(snapshot(&vt, Duration::from_millis(500)));
            }
        }
    }

    let _ = child.kill();
    Ok(frames)
}

/// A point-in-time capture of the terminal screen, ready to be rendered.
fn snapshot(vt: &vt100::Parser, duration: Duration) -> Frame {
    Frame {
        screen: vt.screen().clone(),
        duration,
    }
}

/// Converts a key combo into the raw bytes sent to the PTY for Ctrl chords.
///
/// Chars use the standard ASCII control mapping (low 5 bits, `& 0x1f`).
/// Digits use the Kitty keyboard protocol (`CSI <codepoint> ; 5 u`) since terminals
/// don't produce meaningful single-byte control codes for digit keys.
fn ctrl_bytes_from_keycombo(key: &KeyCombo, alt: bool) -> Vec<u8> {
    match key {
        KeyCombo::Char(c) => {
            let byte = c.to_ascii_lowercase() as u8 & 0x1f;
            if alt { vec![0x1b, byte] } else { vec![byte] }
        }
        KeyCombo::Digit(digit) => {
            let codepoint = b'0' + digit;
            let seq = format!("\x1b[{};5u", codepoint);
            if alt {
                let mut v = vec![0x1b];
                v.extend_from_slice(seq.as_bytes());
                v
            } else {
                seq.into_bytes()
            }
        }
    }
}

/// Waits for the terminal to settle by consuming output until the PTY goes quiet for 50ms.
async fn read_until_idle(rx: &mut tokio::sync::mpsc::Receiver<Vec<u8>>, vt: &mut vt100::Parser) {
    while let Ok(Some(bytes)) = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
        vt.process(&bytes);
    }
}

/// Blocks until the terminal output matches `regex`, or until `timeout` expires.
/// When `last_line_only` is true, only the last output line is tested against the pattern.
async fn expect_output(
    rx: &mut tokio::sync::mpsc::Receiver<Vec<u8>>,
    vt: &mut vt100::Parser,
    regex: &str,
    timeout: Option<Duration>,
    last_line_only: bool,
) {
    let re = Regex::new(regex).unwrap();

    let matches = |vt: &vt100::Parser| {
        let contents = vt.screen().contents();
        if last_line_only {
            let last = contents
                .lines()
                .rfind(|l| !l.trim().is_empty())
                .unwrap_or("");
            re.is_match(last)
        } else {
            re.is_match(&contents)
        }
    };

    if matches(vt) {
        return;
    }

    loop {
        match tokio::time::timeout(timeout.unwrap_or(Duration::from_secs(10)), rx.recv()).await {
            Ok(Some(bytes)) => {
                vt.process(&bytes);
            }
            _ => break,
        }
        if matches(vt) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add more tests for the player module later
    #[test]
    fn ctrl_bytes_for_chars() {
        let cases = vec![
            (KeyCombo::Char('A'), false, vec![0x01]),
            (KeyCombo::Char('c'), false, vec![0x03]),
            (KeyCombo::Char('c'), true, vec![0x1b, 0x03]),
        ];

        for (input, alt, expected) in cases {
            assert_eq!(
                ctrl_bytes_from_keycombo(&input, alt),
                expected,
                "failed for input: {:?}",
                input
            );
        }
    }

    #[test]
    fn ctrl_bytes_for_digits_use_kitty_protocol() {
        let cases = vec![
            (KeyCombo::Digit(1), false, b"\x1b[49;5u".to_vec()),
            (KeyCombo::Digit(5), false, b"\x1b[53;5u".to_vec()),
            (KeyCombo::Digit(1), true, b"\x1b\x1b[49;5u".to_vec()),
        ];

        for (input, alt, expected) in cases {
            assert_eq!(
                ctrl_bytes_from_keycombo(&input, alt),
                expected,
                "failed for input: {:?}",
                input
            );
        }
    }
}
