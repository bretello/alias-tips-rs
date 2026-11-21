//! Blackbox tests running the built binary as a subprocess, mirroring the
//! original plugin's test_alias-tips.py TestBlackbox suite.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run the binary with `input` on the command line and `aliases` on stdin.
/// `envs` are set for the child process only (each test gets its own
/// environment, so tests can run in parallel).
/// Returns (stdout, exit code).
fn run_blackboxed(input: &str, aliases: &str, envs: &[(&str, &str)]) -> (String, Option<i32>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_alias-tips-rs"))
        .arg(input)
        .envs(envs.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn binary");
    child
        .stdin
        .as_mut()
        .expect("failed to open stdin")
        .write_all(aliases.as_bytes())
        .expect("failed to write aliases");
    let output = child.wait_with_output().expect("failed to wait for binary");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        output.status.code(),
    )
}

fn tip(prefix: &str, suggestion: &str) -> String {
    format!("\x1b[94m{prefix}\x1b[1;94m{suggestion}\x1b[0m\n")
}

/// Running without arguments exits with a usage message instead of panicking.
#[test]
fn missing_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_alias-tips-rs"))
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: alias-tips-rs <command>\n"
    );
}

#[test]
fn no_envs() {
    let (out, code) = run_blackboxed("foo", "", &[]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));
}

#[test]
fn text_env() {
    let (out, code) = run_blackboxed("bar", "f=bar", &[("ZSH_PLUGINS_ALIAS_TIPS_TEXT", "Foo")]);
    assert_eq!(out, tip("Foo", "f"));
    assert_eq!(code, Some(0));
}

#[test]
fn exclude_env() {
    const EXCLUDES: &str = "ZSH_PLUGINS_ALIAS_TIPS_EXCLUDES";
    // The excluded alias 'f' produces no tip
    let (out, code) = run_blackboxed("bar", "f=bar", &[(EXCLUDES, "f")]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));

    let (out, code) = run_blackboxed("f", "f=bar", &[(EXCLUDES, "f b")]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));

    let (out, code) = run_blackboxed("b", "f=bar\nb=baz", &[(EXCLUDES, "f b")]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));

    // But a non-excluded alias still tips
    let (out, code) = run_blackboxed("bar", "f=bar\nb=baz", &[(EXCLUDES, "b")]);
    assert_eq!(out, tip("Alias tip: ", "f"));
    assert_eq!(code, Some(0));
}

#[test]
fn no_same_length() {
    let (out, code) = run_blackboxed("ls", "ls=ls", &[]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));
}

#[test]
fn multiple() {
    let (out, code) = run_blackboxed("git status -sb", "g='git'\ngit st = git status -sb", &[]);
    assert_eq!(out, tip("Alias tip: ", "g st"));
    assert_eq!(code, Some(0));
}

#[test]
fn expand_env_on() {
    const EXPAND: &str = "ZSH_PLUGINS_ALIAS_TIPS_EXPAND";
    const ALIASES: &str = "gRv='git remote -v'\ngR='git remote'";
    let (out, code) = run_blackboxed("gR -v", ALIASES, &[(EXPAND, "1")]);
    assert_eq!(out, tip("Alias tip: ", "gRv"));
    assert_eq!(code, Some(0));

    let (out, _) = run_blackboxed("gR -v -foo", ALIASES, &[(EXPAND, "1")]);
    assert_eq!(out, tip("Alias tip: ", "gRv -foo"));

    let (out, _) = run_blackboxed(
        "g status -sb",
        "g='git'\ngit st = git status -sb",
        &[(EXPAND, "1")],
    );
    assert_eq!(out, tip("Alias tip: ", "g st"));
}

#[test]
fn expand_env_off() {
    const EXPAND: &str = "ZSH_PLUGINS_ALIAS_TIPS_EXPAND";
    const ALIASES: &str = "gRv='git remote -v'\ngR='git remote'";
    let (out, code) = run_blackboxed("gR -v", ALIASES, &[(EXPAND, "0")]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));

    let (out, code) = run_blackboxed("gR -v -foo", ALIASES, &[(EXPAND, "0")]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));

    let (out, code) = run_blackboxed(
        "g status -sb",
        "g='git'\ngit st = git status -sb",
        &[(EXPAND, "0")],
    );
    assert_eq!(out, "");
    assert_eq!(code, Some(1));
}

#[test]
fn force_env() {
    const FORCE: &str = "ZSH_PLUGINS_ALIAS_TIPS_FORCE";
    // The force exit code is checked by the zsh plugin to interrupt the command
    let (out, code) = run_blackboxed("foo", "b=foo", &[(FORCE, "1")]);
    assert_eq!(out, tip("Alias tip: ", "b"));
    assert_eq!(code, Some(10));

    // Without a tip there is no force exit code
    let (out, code) = run_blackboxed("foo", "", &[(FORCE, "1")]);
    assert_eq!(out, "");
    assert_ne!(code, Some(10));
}

#[test]
fn function_names_produce_no_tip() {
    let (out, code) = run_blackboxed("foo", "foo () {\n}\nx=foo\n", &[]);
    assert_eq!(out, "");
    assert_eq!(code, Some(1));
}
