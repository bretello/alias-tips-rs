# alias-tips-rs

A zsh plugin that suggests an alias when you type a command you have an alias
for. Rust rewrite of [djui/alias-tips](https://github.com/djui/alias-tips),
with the same features and configuration.

```console
$ git status -sb
Alias tip: gst
```

## Install

### As a zsh plugin

Add the plugin to your plugin manager of choice, e.g. with
[antidote](https://github.com/mattmc3/antidote):

```zsh
antidote install bretello/alias-tips-rs
```

The first time a tip would be shown, the plugin looks for the
`alias-tips-rs` binary:

1. `$ALIAS_TIPS_BIN`, if set
2. `alias-tips-rs` in `$PATH`
3. `$ALIAS_TIPS_DEST/alias-tips-rs` (default: `~/.local/bin`)

If it can't be found, you are prompted to download the right static binary
from [GitHub releases](https://github.com/bretello/alias-tips-rs/releases)
(x86_64, aarch64 and armv7; they run on any Linux). Declining, or a failed
download, disables the plugin for the session.

### Manual install

Download the tarball for your architecture from the
[releases page](https://github.com/bretello/alias-tips-rs/releases), extract
the binary into your `$PATH`, and use the plugin file from this repository
as a zsh plugin.

Building from source requires the rust toolchain, with `cargo`:

```bash
cargo build --release
cargo install --path .
```

## Configuration

Plugin variables (set before the plugin is sourced):

| Variable                   | Default                  | Description                                       |
| -------------------------- | ------------------------ | ------------------------------------------------- |
| `ALIAS_TIPS_BIN`           |                          | Explicit path to the binary (skips autodetection) |
| `ALIAS_TIPS_GITHUB`        | `bretello/alias-tips-rs` | Repo to download the binary from                  |
| `ALIAS_TIPS_DEST`          | `~/.local/bin`           | Where to install the downloaded binary            |
| `ALIAS_TIPS_AUTO_DOWNLOAD` | `1`                      | Set to `0` to never prompt for a download         |

Runtime variables (same as djui/alias-tips):

| Variable                             | Default       | Description                                                  |
| ------------------------------------ | ------------- | ------------------------------------------------------------ |
| `ZSH_PLUGINS_ALIAS_TIPS_TEXT`        | `Alias tip: ` | Tip prefix                                                   |
| `ZSH_PLUGINS_ALIAS_TIPS_EXPAND`      | `1`           | Expand alias prefixes of the typed command                   |
| `ZSH_PLUGINS_ALIAS_TIPS_EXCLUDES`    |               | Space-separated alias names not to suggest                   |
| `ZSH_PLUGINS_ALIAS_TIPS_FORCE`       | `0`           | Interrupt the command after showing a tip, forcing the alias |
| `ZSH_PLUGINS_ALIAS_TIPS_REVEAL`      | `0`           | Show what an alias expands to when you use it                |
| `ZSH_PLUGINS_ALIAS_TIPS_REVEAL_TEXT` | `Alias for: ` | Reveal message prefix                                        |

## Differences from djui/alias-tips

- The suggestion engine is a single static Rust binary, built from this
  repository; no Python interpreter is spawned per command.
- Binary lookup, download and PATH management are handled by the plugin.

## License

MIT
