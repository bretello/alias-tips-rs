use std::env;
use std::io::BufRead;

use alias_tips_rs::aliases;
use alias_tips_rs::config::Config;
use alias_tips_rs::exit_codes::{EXIT_FORCE, EXIT_NO_TIP, EXIT_USAGE};

const TEXT_ENV: &str = "ZSH_PLUGINS_ALIAS_TIPS_TEXT";
const TEXT_DEFAULT: &str = "Alias tip: ";

fn main() {
    // The zsh plugin passes the typed command as the only argument; any other
    // arguments are resolved aliases and ignored.
    let Some(input) = env::args().nth(1) else {
        eprintln!("usage: alias-tips-rs <command>");
        std::process::exit(EXIT_USAGE);
    };

    let config = Config::from_env();

    let mut functions: Vec<String> = Vec::new();
    let mut raw_aliases: Vec<String> = Vec::new();
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        if line.ends_with(" () {") {
            functions.push(line.strip_suffix(" () {").unwrap().to_owned());
        } else if aliases::is_alias_line(&line) {
            raw_aliases.push(line);
        }
    }

    // Don't suggest alias for functions
    if functions.contains(&input) {
        std::process::exit(EXIT_NO_TIP);
    }

    let aliases: Vec<_> = aliases::parse_aliases(raw_aliases.iter().map(String::as_str))
        .into_iter()
        .filter(|alias| !config.excludes.contains(&alias.name))
        .collect();
    let alias = aliases::suggest(&aliases, &input, config.expand);
    if alias.len() < input.len() && alias != input {
        print_tip(&alias);
        if config.force {
            std::process::exit(EXIT_FORCE);
        }
    } else {
        std::process::exit(EXIT_NO_TIP);
    }
}

/// Print a tip to stdout, using the same ANSI colours as the original plugin.
fn print_tip(suggestion: &str) {
    let prefix = env::var(TEXT_ENV).unwrap_or_else(|_| TEXT_DEFAULT.into());
    println!("\x1b[94m{prefix}\x1b[1;94m{suggestion}\x1b[0m");
}
