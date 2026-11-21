/// Exit code returned to the zsh plugin when it should interrupt the command
/// before execution (ZSH_PLUGINS_ALIAS_TIPS_FORCE=1). Matches the original
/// plugin's FORCE_EXIT_CODE.
pub const EXIT_FORCE: i32 = 10;
/// Exit code returned when no alias suggestion was found.
pub const EXIT_NO_TIP: i32 = 1;
/// Exit code returned when the command is called incorrectly.
pub const EXIT_USAGE: i32 = 2;
