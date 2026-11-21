/// An alias: its name and what it expands to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub name: String,
    pub expanded: String,
}

impl Alias {
    fn new(name: impl Into<String>, expanded: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expanded: expanded.into(),
        }
    }
}

/// Whether a line from the zsh side looks like an alias definition
/// (output of `\alias`, e.g. `g='git'`).
pub fn is_alias_line(line: &str) -> bool {
    line.contains('=')
}

/// Parse raw alias lines (e.g. `g='git'`, `git st = git status -sb`) into
/// [`Alias`]es.
///
/// Lines that fail to parse (e.g. multi-line alias declarations) are skipped.
///
/// The result is sorted by descending expansion length so that the
/// suggestion loop below replaces as much of the command as possible with
/// each alias it applies.
pub fn parse_aliases<'a>(lines: impl IntoIterator<Item = &'a str>) -> Vec<Alias> {
    let mut aliases: Vec<Alias> = lines
        .into_iter()
        .filter_map(|line| {
            let (name, expanded) = line.split_once('=')?;
            let name = name.trim().trim_matches('\'');
            let expanded = expanded.trim().trim_matches('\'');
            (!name.is_empty()).then(|| Alias::new(name, expanded))
        })
        .collect();
    aliases.sort_by_key(|alias| std::cmp::Reverse(alias.expanded.len()));
    aliases
}

/// Find an alias suggestion for `input`: the command rewritten so that any
/// alias expansions it starts with are replaced by the alias name.
///
/// If `expand` is set, alias prefixes of `input` are first expanded
/// (e.g. `gR -v` → `git remote -v`) so that they can be collapsed into a
/// single alias again.
pub fn suggest(aliases: &[Alias], input: &str, expand: bool) -> String {
    let mut command = if expand {
        expand_input(input, aliases)
    } else {
        input.to_owned()
    };

    let mut prev = String::new();
    while prev != command {
        prev.clone_from(&command);
        for alias in aliases {
            let expanded = alias.expanded.as_str();
            if command == expanded || command.starts_with(&format!("{expanded} ")) {
                command = format!("{}{}", alias.name, &command[expanded.len()..]);
            }
        }
    }

    command
}

/// Replace an alias prefix of `input` with its expansion, preferring the
/// alias whose expansion is longest.
pub fn expand_input(input: &str, aliases: &[Alias]) -> String {
    let mut max_len = 0;
    let mut result = input.to_owned();
    for alias in aliases {
        if !input.starts_with(&format!("{} ", alias.name)) {
            continue;
        }
        if alias.expanded.len() <= alias.name.len() || alias.expanded.len() <= max_len {
            continue;
        }
        result = input.replacen(&alias.name, &alias.expanded, 1);
        max_len = alias.expanded.len();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aliases(list: &[(&str, &str)]) -> Vec<Alias> {
        let raw = list
            .iter()
            .map(|(name, expanded)| format!("{name}='{expanded}'"))
            .collect::<Vec<_>>();
        parse_aliases(raw.iter().map(String::as_str))
    }

    #[test]
    fn parse_empty() {
        assert!(parse_aliases([]).is_empty());
        assert!(parse_aliases([""]).is_empty());
    }

    #[test]
    fn parse_simple() {
        assert_eq!(parse_aliases(["foo=bar"]), vec![Alias::new("foo", "bar")]);
    }

    #[test]
    fn parse_quoted_expansion() {
        assert_eq!(parse_aliases(["foo='bar'"]), vec![Alias::new("foo", "bar")]);
    }

    #[test]
    fn parse_quoted_name() {
        assert_eq!(parse_aliases(["'foo'=bar"]), vec![Alias::new("foo", "bar")]);
    }

    #[test]
    fn parse_multiword_expansion() {
        assert_eq!(
            parse_aliases(["foo='bar -1'"]),
            vec![Alias::new("foo", "bar -1")]
        );
    }

    #[test]
    fn parse_expansion_with_equals() {
        assert_eq!(
            parse_aliases(["foo='bar=baz'"]),
            vec![Alias::new("foo", "bar=baz")]
        );
    }

    #[test]
    fn parse_git_alias() {
        assert_eq!(
            parse_aliases(["git st = git status -sb"]),
            vec![Alias::new("git st", "git status -sb")]
        );
    }

    #[test]
    fn parse_skips_unparseable() {
        assert!(parse_aliases(["=git", "='"]).is_empty());
    }

    /// Multi-line alias declarations are not supported (as in the original
    /// plugin): the first line is parsed, the continuation is garbage.
    #[test]
    fn parse_partial_multiline() {
        assert_eq!(parse_aliases(["foo='bar"]), vec![Alias::new("foo", "bar")]);
    }

    #[test]
    fn expand_no_aliases() {
        assert_eq!(expand_input("", &[]), "");
        assert_eq!(expand_input("gR -v", &[]), "gR -v");
    }

    #[test]
    fn expand_no_input() {
        assert_eq!(
            expand_input(
                "",
                &aliases(&[("gRv", "git remote -v"), ("gR", "git remote")])
            ),
            ""
        );
    }

    #[test]
    fn expand_prefers_longest() {
        let a = aliases(&[("gRv", "git remote -v"), ("gR", "git remote")]);
        assert_eq!(expand_input("gR -v", &a), "git remote -v");
        assert_eq!(expand_input("gR -v -foo", &a), "git remote -v -foo");
        // The longest matching expansion must win even when a shorter one
        // also matches (regression test: this used to expand to nothing
        // because aliases were sorted the wrong way around).
        let b = aliases(&[("gRv", "git remote -v")]);
        assert_eq!(expand_input("gR -v", &b), "gR -v");
        let c = aliases(&[("gRv", "git remote -v"), ("gR", "git remote")]);
        assert_eq!(expand_input("gR -v", &c), "git remote -v");
    }

    #[test]
    fn expand_no_prefix_match() {
        let a = aliases(&[("gRv", "git remote -v")]);
        assert_eq!(expand_input("gR -v", &a), "gR -v");
    }

    #[test]
    fn suggest_no_aliases() {
        assert_eq!(suggest(&[], "", false), "");
        assert_eq!(suggest(&[], "foo", false), "foo");
    }

    #[test]
    fn suggest_equal_length() {
        assert_eq!(suggest(&aliases(&[("f", "bar")]), "bar", false), "f");
    }

    #[test]
    fn suggest_suffix() {
        assert_eq!(suggest(&aliases(&[("f", "bar")]), "bar -v", false), "f -v");
    }

    #[test]
    fn suggest_picks_first_of_multiple() {
        let a = aliases(&[("f", "bar"), ("g", "bar")]);
        assert_eq!(suggest(&a, "bar -v", false), "f -v");
    }

    #[test]
    fn suggest_multiple() {
        let a = aliases(&[("g", "git"), ("git st", "git status -sb")]);
        assert_eq!(suggest(&a, "git status -sb", false), "g st");
    }

    #[test]
    fn suggest_with_expansion() {
        let a = aliases(&[("gRv", "git remote -v"), ("gR", "git remote")]);
        assert_eq!(suggest(&a, "gR -v", true), "gRv");
        assert_eq!(suggest(&a, "gR -v -foo", true), "gRv -foo");
    }

    #[test]
    fn suggest_chain() {
        // ff → bar and f → ff: the loop must converge to the shortest form
        let a = aliases(&[("ff", "bar"), ("f", "ff")]);
        assert_eq!(suggest(&a, "bar -v", false), "f -v");
        assert_eq!(suggest(&a, "bar -v", true), "f -v");
        assert_eq!(suggest(&a, "bar", false), "f");
    }

    #[test]
    fn suggest_without_expansion() {
        let a = aliases(&[("gRv", "git remote -v"), ("gR", "git remote")]);
        assert_eq!(suggest(&a, "gR -v", false), "gR -v");
        assert_eq!(suggest(&a, "gR -v -foo", false), "gR -v -foo");
    }
}
