<div align="center">
  <p>
    <h2>take</h2>
  </p>
  <p>Script terminal sessions in plain text. Replay them exactly.</p>

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Status](https://img.shields.io/badge/status-early%20development-yellow.svg)

[Quick Start](#quick-start) • [Why take](#why-take) • [Install](#install) • [Script Format](#script-format) • [Project Maturity](#project-maturity) • [License](#license)

</div>

`take` is a Rust CLI for authoring terminal scripts as code.
No screen recording workflow, no manual retakes. Just a `.take` file and deterministic playback.

## Quick Start

```bash
take play demo.take
```

```text
# demo.take

Set Shell zsh
Set Pace 250ms

Sleep 1s
Type "echo hello from take"
Enter
Sleep 700ms
Type "ls -la"
Enter
```

## Why take?

- Plain-text scripts you can read, review, and version in git.
- Deterministic playback for docs, demos, and walkthroughs.
- A compact DSL focused on terminal flow: typing, pauses, enter, and visibility.

## Install

```bash
cargo install --path .
```

## Script Format

`.take` files describe a terminal session with simple directives.

### Directives

| Directive | Description |
|---|---|
| `Set Shell <shell>` | Shell to use (`zsh`, `bash`, `fish`, ...) |
| `Set Output <file>` | Output file path (for render/export workflows) |
| `Set Pace <duration>` | Default typing pace between keystrokes |
| `Sleep <duration>` | Pause for a duration (`1s`, `500ms`, ...) |
| `Type@<pace> "<text>"` | Type text with an optional per-command pace override |
| `Type "<text>"` | Type text using the default pace |
| `Enter` | Press return |
| `Hide` | Hide terminal output |
| `Show` | Show terminal output |

Durations follow Rust style units: `ms` for milliseconds, `s` for seconds.

## Project Maturity

`take` is in early development and the DSL is still evolving.

Right now:
- `take play` is in progress.
- `take rec` is planned.
- Export workflows are in progress.

Near term:
- Complete playback stability and script compatibility.
- Add first-pass recording support.
- Improve export options (GIF / video).

## License

MIT
