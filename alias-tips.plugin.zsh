# alias-tips-rs plugin
#
# Suggests aliases for the command you just typed.
# Rust port of djui/alias-tips (https://github.com/djui/alias-tips).
#
# Configuration:
#   ALIAS_TIPS_BIN       path to the alias-tips-rs binary (skips autodetection)
#   ALIAS_TIPS_GITHUB    GitHub repo to download the binary from
#                        (default: bretello/alias-tips-rs)
#   ALIAS_TIPS_DEST      where to install the downloaded binary
#                        (default: ~/.local/bin)
#   ALIAS_TIPS_AUTO_DOWNLOAD  set to 0 to never prompt for a download
#
# Environment variables understood by the binary (same as djui/alias-tips):
#   ZSH_PLUGINS_ALIAS_TIPS_TEXT      tip prefix (default: "Alias tip: ")
#   ZSH_PLUGINS_ALIAS_TIPS_EXPAND    expand alias prefixes of the typed
#                                    command (default: 1)
#   ZSH_PLUGINS_ALIAS_TIPS_EXCLUDES  space-separated aliases not to suggest
#   ZSH_PLUGINS_ALIAS_TIPS_FORCE     force alias usage by interrupting the
#                                    command (default: 0)

typeset -g ALIAS_TIPS_BIN=${ALIAS_TIPS_BIN:-}
typeset -g ALIAS_TIPS_GITHUB=${ALIAS_TIPS_GITHUB:-bretello/alias-tips-rs}
typeset -g ALIAS_TIPS_DEST=${ALIAS_TIPS_DEST:-$HOME/.local/bin}

# Resolved binary path, empty until the binary has been located/downloaded.
typeset -g _alias_tips__bin=""

# Map the machine architecture to a release target triple.
_alias_tips__target() {
  local os=$(uname -s) arch=$(uname -m)
  case $arch in
    x86_64|amd64)
      case $os in
        Linux) echo x86_64-unknown-linux-musl ;;
        Darwin) echo x86_64-apple-darwin ;;
      esac
      ;;
    aarch64|arm64)
      case $os in
        Linux) echo aarch64-unknown-linux-musl ;;
        Darwin) echo aarch64-apple-darwin ;;
      esac
      ;;
    armv7l|armv7)
      [[ $os == Linux ]] && echo armv7-unknown-linux-musleabihf
      ;;
    *) return 1 ;;
  esac
}

# Locate the alias-tips-rs binary: $ALIAS_TIPS_BIN, then $PATH, then the
# download location. Returns 0 and sets REPLY to the binary path.
_alias_tips__find_bin() {
  if [[ -n $ALIAS_TIPS_BIN ]]; then
    if [[ -x $ALIAS_TIPS_BIN ]]; then
      REPLY=$ALIAS_TIPS_BIN
      return 0
    fi
    echo "alias-tips-rs: ALIAS_TIPS_BIN is not executable: $ALIAS_TIPS_BIN" >&2
    return 1
  fi

  local bin=$(command -v alias-tips-rs)
  if [[ -n $bin ]]; then
    REPLY=$bin
    return 0
  fi

  if [[ -x $ALIAS_TIPS_DEST/alias-tips-rs ]]; then
    REPLY=$ALIAS_TIPS_DEST/alias-tips-rs
    return 0
  fi

  return 1
}

# Download the binary from GitHub releases into $1 (target triple), $2 (dest).
# Returns 0 on success.
_alias_tips__download() {
  local target=$1 dest=$2
  local url="https://github.com/${ALIAS_TIPS_GITHUB}/releases/latest/download/alias-tips-rs-${target}.tar.gz"

  echo "alias-tips-rs: downloading $url" >&2
  mkdir -p $dest
  local tmp=$(mktemp -d)
  if ! curl -fsSL $url | tar -xz -C $tmp; then
    echo "alias-tips-rs: download failed. Install the binary manually from:" >&2
    echo "  https://github.com/${ALIAS_TIPS_GITHUB}/releases" >&2
    rm -rf $tmp
    return 1
  fi

  mv $tmp/alias-tips-rs $dest/alias-tips-rs && chmod +x $dest/alias-tips-rs
  rm -rf $tmp
  echo "alias-tips-rs: installed to $dest/alias-tips-rs" >&2

  # Offer to prepend the install directory to PATH for this session if the
  # binary is not reachable through it (front, so it wins over system
  # binaries of the same name).
  if [[ -z ${path[(r)$dest]} ]]; then
    if _alias_tips__prompt "add $dest to PATH? [y/N] "; then
      echo
      path=($dest $path)
      echo "alias-tips-rs: added $dest to PATH (this session only)" >&2
      echo "alias-tips-rs: to make it permanent, add to your zshrc:" >&2
      echo "  export PATH=\"$dest:\$PATH\"" >&2
    else
      echo
    fi
  fi
}

# Resolve the binary to use, downloading it if the user agrees.
# Sets _alias_tips__bin; returns 1 if no binary could be resolved.
_alias_tips__resolve_bin() {
  if _alias_tips__find_bin; then
    _alias_tips__bin=$REPLY
    return 0
  fi

  local target
  if ! target=$(_alias_tips__target); then
    echo "alias-tips-rs: unsupported OS/architecture: $(uname -s)/$(uname -m)" >&2
    return 1
  fi

  if [[ ${ALIAS_TIPS_AUTO_DOWNLOAD:-1} == 0 ]]; then
    return 1
  fi

  echo "alias-tips-rs: binary not found." >&2
  if _alias_tips__prompt "download it from github.com/${ALIAS_TIPS_GITHUB}? [y/N] "; then
    echo
    _alias_tips__download $target $ALIAS_TIPS_DEST || return 1
    _alias_tips__bin=$ALIAS_TIPS_DEST/alias-tips-rs
    return 0
  fi
  echo
  return 1
}

# Ask a yes/no question on stdin. Works in interactive shells (line editing)
# and when stdin is piped/scripted, unlike read -q which requires a terminal.
_alias_tips__prompt() {
  local answer
  printf "alias-tips-rs: %s" "$1" >&2
  if ! read answer; then
    return 1
  fi
  [[ $answer == [yY]* ]]
}

_alias_tips__preexec() {
  local cmd=$1
  local cmd_expanded=$2

  # Locate the binary the first time a tip could be shown
  if [[ -z $_alias_tips__bin ]]; then
    if ! _alias_tips__resolve_bin; then
      # Give up for this session: prompting again on every command would be
      # insufferable. Restarting the shell retries.
      add-zsh-hook -d preexec _alias_tips__preexec
      return 0
    fi
  fi

  if [[ $cmd != $cmd_expanded ]]; then
    if [[ ${ZSH_PLUGINS_ALIAS_TIPS_REVEAL-0} == 1 ]] \
    && [[ ${${ZSH_PLUGINS_ALIAS_TIPS_REVEAL_EXCLUDES-()}[(I)$1]} == 0 ]]; then
      local reveal_text=${ZSH_PLUGINS_ALIAS_TIPS_REVEAL_TEXT-Alias for: }
      local color_dark_gray=$'\e[1;30m'
      local color_reset=$'\e[0m'
      echo -e "$color_dark_gray$reveal_text$cmd_expanded $color_reset"
    fi
    if [[ ${ZSH_PLUGINS_ALIAS_TIPS_EXPAND-1} == 0 ]]; then
      return 0
    fi
  fi

  local git_aliases=""
  if hash git 2>/dev/null && [[ -d .git ]]; then
    # alias.foo bar      => git foo = git bar
    # alias.foo !git bar => git foo = git bar
    git_aliases=$(git config --get-regexp "^alias\." | \
      sed 's/[ ]/ = /' | \
      sed 's/^alias\./git /' | \
      sed 's/ = \([^!]\)/ = git \1/' | \
      sed 's/ = !/ = /')
  fi

  local shell_aliases
  shell_aliases=$(\alias)

  local shell_functions
  shell_functions=$(\functions | \grep -E -a '^[a-zA-Z].+ \(\) \{$')

  # Exit code returned from the binary when we want to force use of aliases.
  local force_exit_code=10
  echo $shell_functions "\n" $git_aliases "\n" $shell_aliases | \
    $_alias_tips__bin $cmd
  local ret=$?
  if [[ $ret = $force_exit_code ]]; then kill -s INT $$; fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _alias_tips__preexec
