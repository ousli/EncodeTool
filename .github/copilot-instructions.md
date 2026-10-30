# Copilot instructions

EncodeTool is a single interactive zsh script (`EncodeTool.sh`, macOS only) that batch-renames and re-encodes videos with ffmpeg. There is no build system, test suite or linter.

## Run / check

- Run: `./EncodeTool.sh` (interactive: prompts for source folder, then a menu).
- Syntax check: `zsh -n EncodeTool.sh`.
- Requires macOS tools (BSD `stat -f`, `open`), `ffmpeg` with `hevc_videotoolbox`, optional `ffprobe`, and `perl`.

## Architecture

- Configuration lives at the top: `SOURCE_DIR` / `EXPORT_DIR` (empty = prompt) and `LUT_SLOG2` / `LUT_SLOG3` / `LUT_DLOGM`, which are absolute paths specific to the author's machine.
- Menu loop at the bottom dispatches to `rename_files`, `encode_files` (mode `plain` or `rename`) and `apply_lut` (profile chosen by `select_profile`, which sets the globals `PROFILE_NAME` / `PROFILE_LUT`).
- All ffmpeg calls go through `run_ffmpeg_progress input idx total <ffmpeg args...>`. It runs ffmpeg with `-progress pipe:1`, parses the key=value stream, computes percent/ETA with `awk` (duration from `ffprobe`), and redraws a fixed 10-line block using ANSI cursor-up (`\033[10A`). If you change the number of lines printed in that block, update the `10A` in both places (redraw and final cleanup).
- Output files are always `.mov` (HEVC Main10, `-tag:v hvc1`, AAC 192k). Original creation/modification dates are copied onto outputs by `apply_original_dates`.
- LUT output naming: `<name>_<profile>_lutted.mov`; rename prefix format: `YYYY-MM-DD_HHMM_` from the file's mtime (`get_formatted_date`).

## Conventions

- Script is zsh, not bash: uses `print`, `read "var?prompt"`, `$+commands[...]`, `pipestatus`, `setopt local_options null_glob`. Don't introduce bash-only syntax.
- Helpers that return values via stdout must send prompts to stderr (`>&2`), as in `get_source_folder` / `setup_export_folder`.
- Dragged-and-dropped paths go through `clean_path` (strips quotes, escaped spaces, trailing NBSP).
- Video discovery is `list_videos`, which returns matches in `REPLY` for `*.mp4|*.mov` (case variants only).
- Keep `unsetopt xtrace verbose` at the top; it prevents trace output from corrupting the progress display.
- User-facing strings are in French with emoji prefixes; the README is in English.
