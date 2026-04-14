use crate::roku::{Instruction, Key, RokuFile};
use std::time::Duration;

/// Parse a `.roku` script into a [`RokuFile`].
///
/// The format is intentionally line-oriented:
/// - Blank lines are ignored.
/// - Lines starting with `#` are treated as comments.
/// - Instructions are `Keyword [args...]`
/// - Some instructions accept an `@modifier` suffix (currently used for per-instruction pacing),
///   e.g. `Type@250ms "hello"`.
///
/// This parser aims to be forgiving about whitespace, but strict about unknown keywords/settings
/// so mistakes are surfaced early.
pub fn parse(input: &str) -> Result<RokuFile, String> {
    let mut file = RokuFile::default();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split into `keyword[@modifier]` and the remainder (if any) without losing spaces inside
        // arguments (e.g. quoted strings).
        let mut parts = line.splitn(2, ' ');

        // `Type@250ms` style modifier. This is parsed here so each instruction can interpret it
        // independently (e.g. Type uses it as a per-instruction pace).
        let keyword = parts.next();
        let mut keyword_parts = keyword.unwrap_or("").splitn(2, '@');
        let keyword = keyword_parts.next();
        let modifier = keyword_parts.next();

        let rest = parts.next();

        match keyword {
            Some("Set") => {
                let Some(rest) = rest else {
                    return Err("Set requires a setting name and value".to_string());
                };
                let mut set_args = rest.splitn(2, ' ');
                let setting = set_args.next();
                let value = set_args.next();

                match setting {
                    Some("Shell") => {
                        let Some(value) = value else {
                            return Err("Set Shell requires a value".to_string());
                        };
                        file.shell = Some(value.to_string());
                    }
                    Some("Output") => {
                        let Some(value) = value else {
                            return Err("Set Output requires a path".to_string());
                        };
                        file.output = Some(value.to_string());
                    }
                    // TODO: File-level pacing that acts as the default for instructions that
                    // support pacing (e.g. `Type`) unless overridden with `@...`.
                    Some("Pace") => todo!(),
                    Some(unkwown) => return Err(format!("unkwown setting: {}", unkwown)),
                    None => return Err("Set requires a setting name".to_string()),
                }
            }
            Some("Type") => {
                let Some(rest) = rest else {
                    return Err("Type requires a value".to_string());
                };
                let typed = rest.trim_matches('"').to_string();
                let pace = modifier.map(parse_duration).transpose()?;

                file.instructions
                    .push(Instruction::Type { text: typed, pace })
            }
            Some("Enter") => {
                file.instructions.push(Instruction::Press(
                    Key::Enter,
                    parse_key_count("Enter", rest)?,
                ));
            }
            Some("Backspace") => {
                file.instructions.push(Instruction::Press(
                    Key::Backspace,
                    parse_key_count("Backspace", rest)?,
                ));
            }
            Some("Tab") => {
                file.instructions
                    .push(Instruction::Press(Key::Tab, parse_key_count("Tab", rest)?));
            }
            Some("Space") => {
                file.instructions.push(Instruction::Press(
                    Key::Space,
                    parse_key_count("Space", rest)?,
                ));
            }
            Some("Up") => file
                .instructions
                .push(Instruction::Press(Key::Up, parse_key_count("Up", rest)?)),
            Some("Down") => file.instructions.push(Instruction::Press(
                Key::Down,
                parse_key_count("Down", rest)?,
            )),
            Some("Left") => file.instructions.push(Instruction::Press(
                Key::Left,
                parse_key_count("Left", rest)?,
            )),
            Some("Right") => file.instructions.push(Instruction::Press(
                Key::Right,
                parse_key_count("Right", rest)?,
            )),
            Some("Sleep") => {
                let Some(rest) = rest else {
                    return Err("Sleep requires a duration".to_string());
                };
                let duration = parse_duration(rest)?;
                file.instructions.push(Instruction::Sleep(duration));
            }
            Some("Hide") => {
                file.instructions.push(Instruction::Hide);
            }
            Some("Show") => {
                file.instructions.push(Instruction::Show);
            }
            Some("Expect") => {
                // TODO: Implement output expectations (regex + optional timeout).
                todo!();
            }
            Some("ExpectLine") => {
                // TODO: Implement line-scoped output expectations (regex + optional timeout).
                todo!();
            }
            Some("Ctrl") => {
                // TODO: Implement modifier + key parsing (e.g. Ctrl+C, Ctrl+Shift+X).
                todo!();
            }
            None => unreachable!(),
            _ => return Err(format!("unkwown instruction: {:?}", keyword)),
        }
    }
    Ok(file)
}

/// Parse a duration token used in `.roku` scripts.
///
/// Accepted suffixes:
/// - `ms` for milliseconds (e.g. `250ms`)
/// - `s` for seconds (e.g. `2s`)
///
/// Returns a string error for unknown suffixes or invalid numbers.
fn parse_duration(input: &str) -> Result<Duration, String> {
    if input.ends_with("ms") {
        let value = input.trim_end_matches("ms");
        let ms = value.parse::<u64>().map_err(|e| e.to_string())?;
        Ok(Duration::from_millis(ms))
    } else if input.ends_with('s') {
        let value = input.trim_end_matches('s');
        let secs = value.parse::<u64>().map_err(|e| e.to_string())?;
        Ok(Duration::from_secs(secs))
    } else {
        Err(format!("unkown duration: {:?}", input.to_string()))
    }
}

/// Parse an optional `count` argument for key press instructions.
///
/// Examples:
/// - `Enter` -> 1
/// - `Enter 3` -> 3
fn parse_key_count(keyword: &str, rest: Option<&str>) -> Result<u32, String> {
    let count = if let Some(rest) = rest {
        rest.parse::<u32>().map_err(|_| {
            format!(
                "invalid count for {}: \"{}\" - expected a positive integer",
                keyword, rest
            )
        })?
    } else {
        1
    };
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = r##"
# example.roku
Set Shell zsh
Set Output demo.gif
Sleep 1s
Enter 3
Type@250ms "# this is a comment"
Hide
Show
"##;
        let result = parse(input).unwrap();
        assert_eq!(result.shell, Some("zsh".to_string()));
        assert_eq!(result.output, Some("demo.gif".to_string()));
        assert_eq!(
            result.instructions,
            vec![
                Instruction::Sleep(Duration::from_secs(1)),
                Instruction::Press(Key::Enter, 3),
                Instruction::Type {
                    text: "# this is a comment".to_string(),
                    pace: Some(Duration::from_millis(250))
                },
                Instruction::Hide,
                Instruction::Show,
            ]
        );
    }

    #[test]
    fn test_parse_duration() {
        let cases = vec![
            ("1s", Ok(Duration::from_secs(1))),
            ("2s", Ok(Duration::from_secs(2))),
            ("100ms", Ok(Duration::from_millis(100))),
            ("500ms", Ok(Duration::from_millis(500))),
            (
                "ten seconds",
                Err("invalid digit found in string".to_string()),
            ),
            ("", Err("unkown duration: \"\"".to_string())),
        ];

        for (input, expected) in cases {
            let duration = parse_duration(input);
            assert_eq!(duration, expected, "failed for input: {}", input);
        }
    }

    #[test]
    fn test_parse_keys() {
        let cases = vec![
            ("Enter", Ok(vec![Instruction::Press(Key::Enter, 1)])),
            ("Enter 3", Ok(vec![Instruction::Press(Key::Enter, 3)])),
            ("Backspace", Ok(vec![Instruction::Press(Key::Backspace, 1)])),
            (
                "Backspace 5",
                Ok(vec![Instruction::Press(Key::Backspace, 5)]),
            ),
            ("Up 2", Ok(vec![Instruction::Press(Key::Up, 2)])),
            ("Down", Ok(vec![Instruction::Press(Key::Down, 1)])),
            ("Left 4", Ok(vec![Instruction::Press(Key::Left, 4)])),
            ("Right", Ok(vec![Instruction::Press(Key::Right, 1)])),
            ("Tab 2", Ok(vec![Instruction::Press(Key::Tab, 2)])),
            ("Space", Ok(vec![Instruction::Press(Key::Space, 1)])),
        ];

        for (input, expected) in cases {
            assert_eq!(
                parse(input).map(|f| f.instructions),
                expected,
                "failed for input: {}",
                input
            );
        }
    }
}
