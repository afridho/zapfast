# Fork notes (afridho/zapfast)

This is a personal fork of [crmne/zapfast](https://github.com/crmne/zapfast)
with Linux-focused enhancements and a few upstream pull requests pulled in
early. These notes are for coding agents and for the maintainer of this fork.
The upstream `AGENTS.md` still applies; read it first. This file only records
what this fork adds on top.

## Working rules for this fork

- Keep the fork syncable with upstream `crmne/zapfast`. Do not touch
  `assets/i18n/` or the `fastframe` crates, and do not edit the upstream
  `AGENTS.md`. Fork-specific guidance goes here.
- Only enhance and add. Do not rewrite existing upstream functions or broaden
  a task into a refactor.
- Linux is the focus. Keep Windows and macOS behaviour as upstream has it; a
  change for Linux must keep the other two compiling.
- One commit per feature on local `main`. When pulling an upstream pull
  request, squash it into one commit and keep the author credit
  (`git commit --author=...` plus a `Co-authored-by:` trailer).
- The user writes in Indonesian; reply in Indonesian.

## Build, run and install on Linux

Dev build and run (foreground process is killed and restarted):

```sh
cargo build
pkill -x zapfast; setsid ./target/debug/zapfast >/dev/null 2>&1 < /dev/null &
```

Install to the system (dock, launcher, binary):

```sh
cargo build --release --locked
packaging/install-user.sh
```

`install-user.sh` installs the binary to `~/.local/bin/zapfast`, the launcher
entry to `~/.local/share/applications/zapfast.desktop` (`Icon=zapfast`,
`StartupWMClass=zapfast`), and the app icon to
`~/.local/share/icons/hicolor/512x512/apps/zapfast.png`. It registers the PNG
size directory in `index.theme` and refreshes the icon cache, so the launcher
icon updates without a relogin. On GNOME Wayland the dock may still cache the
old icon until logout/login.

The dock launches the installed binary, not `./target/debug/zapfast`. To make
the dock use fork changes (window and tray icon included), reinstall.

### Build notes

- Builds are slow; use long timeouts.
- Pass `cargo test` one filter per invocation. Multiple filters match nothing.
- Keep `custom_titlebar: true` in `~/.config/zapfast/settings.json`.

## Full checks (definition of done)

Run before finishing substantial work:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-targets --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
```

The full checks passed on Linux for the fork through `44c5c34`.
For PR #726, formatting and default-feature Clippy passed; the user deferred
the remaining checks (all-feature Clippy, both test configurations, rustdoc).

## What this fork adds (oldest to newest)

Own Linux enhancements:

- **install-user.sh icon fix** (`a488334`): fix the Linux icon install and make
  re-running the script replace files in place.
- **Display name "WhatsApp"** (`1a1d3a9`): `zapfast::DISPLAY_NAME` shows
  "WhatsApp" in the interface. Technical ids stay `zapfast`/`fastsapp`.
- **Light/Dark palette pickers** (`ab61c9a`): under Follow system, choose a
  palette for light and for dark.
- **macOS-style traffic lights on Linux** (`6d24333`): when Custom title bar is
  on, Linux draws traffic lights in place of the GNOME caption buttons. See
  `src/ui/window_controls.rs`.
- **Blur chats setting** (`936203f`): `widgets::redact` blurs chat names and
  previews with peek-on-hover; stored as `settings.blur_chats`.
- **Rounded frameless corners on Linux** (`35ddcba`): a transparent window plus
  a `cut_corners` GL scissor clear at 20pt. See `src/main.rs` and
  `src/ui/window_controls.rs` (`rounds_corners`, `cut_corners`).
- **Blur chats toggle in the rail** (`e67098b`): `Stop::BlurChats` sits between
  Calls and Starred in `src/ui/focus.rs` (which drives tab order); toggle in
  `src/ui/rail.rs` (`blur_toggle`). Order-dependent tests in `src/demo.rs`.
- **Custom PNG app/tray/launcher icon** (`710322b`): `app_icon_rgba` in
  `src/util.rs` renders an embedded PNG (`packaging/icons/app-icon.png`) scaled
  to the requested size, with the SVG mark then an accent disc as fallbacks.
  `install-user.sh` installs it under hicolor `512x512/apps`. To change the
  icon, replace `packaging/icons/app-icon.png` and rebuild. A low-colour source
  PNG is best cleaned up first (crop the transparent margin, fill ~88% of the
  canvas, rebuild the alpha edge so it is not jagged).

Upstream pull requests pulled in early (squashed, author credited):

- **Export chats** (`0f473c4`, PR #664, eternalwaitt): "Export chat" in the
  conversation and chat-list menus, no ellipsis and no OS save picker. Exports
  go to `settings.download_folder` else the system Downloads folder, named
  `Chat export - {title}.txt/.zip` with a ` (n)` suffix to avoid overwriting.
  See `src/backend/worker/chat_export.rs`, `src/ui/chats.rs`, `Action::ExportChat`.
- **Single-frame animated WebP stickers** (`4db2121`, PR #702, Goodly58).
- **Click beside the picture closes the media viewer** (`8152a85`, PR #699,
  Edson Boldrini).
- **Native Wayland clipboard paste** (`2ebf70a`, PR #695, simonbbby,
  co-author afridho): pastes image attachments through winit's
  `Clipboard::load_bytes`. arboard fails on GNOME. Needs the winit pin in
  `Cargo.toml` (`[patch.crates-io] winit`) to point at the fork
  `https://github.com/afridho/winit` rev `8e77206e` (branch
  `apps-0.30-clipboard-bytes`). Once `crmne/winit#7` merges upstream, point the
  pin back at `crmne/winit`.
- **Close the emoji picker when sending text** (`1610d2c`, PR #652, simonbbby):
  `send_text` in `src/app.rs` closes the picker. An emoji click only inserts;
  it does not send, same as the official clients.
- **Animate GIF search results and fix short loops** (`69aa84c`, PR #623,
  ayoubdya): GIF previews animate on hover; GIFs with B-frames play their full
  length (openh264 flushes once at the end, frames timed in presentation
  order). One env-brittle test (`a_variable_duration_mp4_with_b_frames_keeps_its_timing`
  in `src/animation.rs`) was loosened: the last frame's trailing duration comes
  from the track end and varies by ffmpeg build, so only the frames before it
  are checked for the display-order pattern.
- **Reaction popup** (`0ec2cf6`, PR #633, dschodits): clicking a reaction chip
  opens a popup with a per-emoji summary and a who-reacted-with-what list,
  instead of toggling the emoji directly.
- **Voice recording presence** (PR #726, eternalwaitt): chat activity keeps
  recording audio distinct from typing. The chat list and header describe
  recording activity, and the activity bubble shows a microphone for recording
  participants while retaining animated dots for participants who are typing.
  The synthetic `voice-presence` demo covers the recording state.

Emoji fallback maintenance: the bundled Noto font was refreshed from revision
`99701e07d3d1fe1bdbcdb14964547f0fd592966f` to cover newer picker entries such as
the distorted face. Tests cover recent smileys with system/fallback rendering
and verify that every base picker entry has a bundled picture. No fastframe
crate or platform font preference was changed.
Settings > Development, immediately above About, contains
`show_unsupported_emoji` (default false). Off filters picker categories, search,
and recents by cached system-font rendering coverage; On allows bundled fallback
artwork. Coverage is probed on a worker thread, not in a UI frame. This replaces
the earlier single-emoji exclusion. Existing messages are unchanged.

Linux scrolling is handled locally in `src/scroll.rs`: direct touchpad movement
with release momentum (260 ms decay), braking on a new gesture or click,
1.3x touchpad gain, and wheel steps
eased over a short 25 ms time constant (120 points per line notch). This replaces both the original
fastframe-scroll behaviour and the abruptly stopping standard-input experiment.
Pane routing and touchpad detection for image previews remain. Windows and macOS
keep their existing scrolling path. No fastframe crate was modified. This is a
browser-inspired response, not Firefox/Zen's exact scroll physics.
