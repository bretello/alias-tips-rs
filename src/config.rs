use std::env;

const EXPAND_ENV: &str = "ZSH_PLUGINS_ALIAS_TIPS_EXPAND";
const EXPAND_DEFAULT: bool = true;
const FORCE_ENV: &str = "ZSH_PLUGINS_ALIAS_TIPS_FORCE";
const FORCE_DEFAULT: bool = false;
const EXCLUDES_ENV: &str = "ZSH_PLUGINS_ALIAS_TIPS_EXCLUDES";

/// Runtime configuration, read from `ZSH_PLUGINS_ALIAS_TIPS_*` environment
/// variables set by the zsh plugin.
#[derive(Debug, Default, Clone)]
pub struct Config {
    /// Expand alias prefixes of the typed command before looking for
    /// suggestions (e.g. `gR -v` → `git remote -v`).
    pub expand: bool,

    /// Alias names that should never be suggested (space separated).
    pub excludes: Vec<String>,

    /// Exit with a special code after printing a tip so that the zsh plugin
    /// interrupts the command, forcing the use of the alias.
    pub force: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let excludes = env::var(EXCLUDES_ENV)
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect();

        Self {
            expand: env_bool(EXPAND_ENV, EXPAND_DEFAULT),
            excludes,
            force: env_bool(FORCE_ENV, FORCE_DEFAULT),
        }
    }
}

/// Read `key` from the environment as a boolean, falling back to `default`.
fn env_bool(key: &str, default: bool) -> bool {
    let Ok(value) = env::var(key) else {
        return default;
    };
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "y" | "t" => true,
        "false" | "no" | "0" | "f" | "n" => false,
        value => {
            eprintln!("alias-tips: ignoring invalid boolean for {key}: {value:?}");
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_bool_defaults_when_unset() {
        const KEY: &str = "ALIAS_TIPS_RS_TEST_BOOL_UNSET";
        env::remove_var(KEY);
        assert!(!env_bool(KEY, false));
        assert!(env_bool(KEY, true));
    }

    #[test]
    fn env_bool_parses_truthy() {
        const KEY: &str = "ALIAS_TIPS_RS_TEST_BOOL_TRUE";
        for value in ["true", "yes", "1", "y", "t", "TRUE", "Yes"] {
            env::set_var(KEY, value);
            assert!(env_bool(KEY, false), "expected {value:?} to be true");
        }
    }

    #[test]
    fn env_bool_parses_falsy() {
        const KEY: &str = "ALIAS_TIPS_RS_TEST_BOOL_FALSE";
        for value in ["false", "no", "0", "f", "n", "FALSE", "No"] {
            env::set_var(KEY, value);
            assert!(!env_bool(KEY, true), "expected {value:?} to be false");
        }
    }

    #[test]
    fn env_bool_invalid_falls_back_to_default() {
        const KEY: &str = "ALIAS_TIPS_RS_TEST_BOOL_INVALID";
        env::set_var(KEY, "bogus");
        assert!(env_bool(KEY, true));
        assert!(!env_bool(KEY, false));
    }
}
