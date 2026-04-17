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

    let cmd = CommandBuilder::new(shell);
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
    let pty = spawn_shell(shell)?;
    let mut writer = pty.writer;
    let mut reader = pty.reader;
    let _child = pty.child;

    let mut vt = vt100::Parser::new(24, 80, 0);
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
        match instruction {
            Instruction::Sleep(duration) => {
                sleep(duration).await;
            }
            Instruction::Type { text, .. } => {
                writer.write_all(text.as_bytes())?;
                writer.flush()?;
                read_until_idle(&mut rx, &mut vt).await;
                if !hidden {
                    frames.push(snapshot(&vt, Duration::from_millis(500)));
                }
            }
            Instruction::Press(key, count) => {
                let bytes = match key {
                    Key::Enter => "\n",
                    Key::Backspace => "\x08",
                    Key::Tab => "\t",
                    Key::Up => "\x1b[A",
                    Key::Down => "\x1b[B",
                    Key::Right => "\x1b[C",
                    Key::Left => "\x1b[D",
                    Key::Space => " ",
                    Key::Delete => "\x7f",
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
                let byte = ctrl_byte_from_keycombo(&key);
                if modifiers.alt {
                    writer.write_all(&[0x1b, byte])?;
                } else {
                    writer.write_all(&[byte])?;
                }
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

    Ok(frames)
}

/// A point-in-time capture of the terminal screen, ready to be rendered.
fn snapshot(vt: &vt100::Parser, duration: Duration) -> Frame {
    Frame {
        screen: vt.screen().clone(),
        duration,
    }
}

/// Converts a key combo into the raw control byte sent to the PTY for Ctrl chords.
///
/// ASCII control mappings are derived by masking the low 5 bits (`& 0x1f`).
/// Digits are first converted back to ASCII (`'0'..'9'`) to keep one consistent rule.
fn ctrl_byte_from_keycombo(key: &KeyCombo) -> u8 {
    match key {
        KeyCombo::Char(c) => c.to_ascii_lowercase() as u8 & 0x1f,
        KeyCombo::Digit(digit) => (b'0' + digit) & 0x1f,
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
    let mut output = String::new();
    let re = Regex::new(regex).unwrap();
    loop {
        match tokio::time::timeout(timeout.unwrap_or(Duration::from_secs(10)), rx.recv()).await {
            Ok(Some(bytes)) => {
                vt.process(&bytes);
                output.push_str(&String::from_utf8_lossy(&bytes));
            }
            _ => break,
        }
        let target = if last_line_only {
            output.lines().last().unwrap_or("")
        } else {
            &output
        };
        if re.is_match(target) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add more tests for the player module later
    #[test]
    fn ctrl_byte_for_chars_and_digits() {
        let cases = vec![
            (KeyCombo::Char('A'), 0x01),
            (KeyCombo::Char('c'), 0x03),
            (KeyCombo::Digit(1), 0x11),
        ];

        for (input, expected) in cases {
            assert_eq!(
                ctrl_byte_from_keycombo(&input),
                expected,
                "failed for input: {:?}",
                input
            );
        }
    }
}
