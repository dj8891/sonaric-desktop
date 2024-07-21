#!/bin/bash

# We don't need return codes for "$(command)", only stdout is needed.
# Allow `[[ -n "$(command)" ]]`, `func "$(command)"`, pipes, etc.
# shellcheck disable=SC2312
set -u

export HOMEBREW_NO_COLOR=1
export HOMEBREW_NO_EMOJI=1
export HOMEBREW_NO_AUTO_UPDATE=1

NAME="Sonaric app installer"
SONARIC_OPTS="--nofancy --nocolor"
SONARIC_PACKAGE_NAME="monk-io/sonaric/sonaric"
SONARIC_SERVICE_NAME="sonaric"
SONARIC_RUNTIME_SERVICE_NAME="sonaric-runtime"

log() {
  echo "$@"
}

abort() {
  echo "ERROR: $@" >&2
  exit 1
}

execute() {
  if ! "$@"; then
    abort "Failed during: ${@}"
  fi
}

add_to_path(){
  for arg in $@; do
    if [ -d "${arg}" ]; then
      if [[ "${PATH}" == "" ]]; then
        PATH="${arg}"
      else
        case ":${PATH}:" in
          *:"${arg}":*)
            ;;
          *)
            PATH="${arg}:${PATH}"
            ;;
        esac
      fi
    fi
  done
}

check_command(){
  for arg in $@; do
    local n="${arg}"
    local p=$(which ${n} 2>/dev/null)
    if [[ "${p}" == "" ]]; then
      abort "${n} not found"
    fi
    if [ ! -x "$(command -v ${p})" ]; then
      abort "${p} not executable"
    fi
  done
}

# Fail fast with a concise message when not using bash
# Single brackets are needed here for POSIX compatibility
# shellcheck disable=SC2292
if [ -z "${BASH_VERSION:-}" ]; then
  abort "Bash is required to interpret this script."
fi

add_to_path "/sbin" "/usr/sbin" "/usr/local/sbin"
add_to_path "/bin" "/usr/bin" "/usr/local/bin"
add_to_path "/opt/homebrew/bin"

check_command id rm ln tr git awk sed arch sort head expr grep mkdir chmod uname
check_command sysctl install dirname basename osascript xcode-select softwareupdate

OS="$(uname 2>/dev/null)"
# Check if we are on mac
if [[ "${OS}" != "Darwin" ]]; then
  abort "${NAME} is only supported on macOS."
fi

OS_ARCH=amd64
case $(arch) in
  arm | arm64 | aarch | aarch64)
    OS_ARCH=arm64
    ;;
  *)
    OS_ARCH=amd64
    ;;
esac

log "${NAME} started on ${OS} ${OS_ARCH}"

export PATH="${PATH}"
log "${NAME} detects paths:"
for p in $(echo ${PATH} | tr ":" " "); do
  log " - ${p}"
done

USER="$(id -un)"
case " $(id -Gn) " in
  *" admin "*)
    GROUP="admin"
    ;;
  *)
    GROUP="$(id -gn)"
    ;;
esac

log "${NAME} started for ${USER}:${GROUP}"

if [[ "${HOME}" == "" ]]; then
  HOME="~"
fi

CLT_PATH="/Library/Developer/CommandLineTools"
CACHE_PATH="${HOME}/Library/Caches/SonaricDesktop"
SETUP_SCRIPT="${CACHE_PATH}/setup.sh"

UNAME_MACHINE="$(uname -m)"
if [[ "${UNAME_MACHINE}" == "arm64" ]]; then
  HOMEBREW_DIR="homebrew"
  # On ARM macOS, this script installs to /opt/homebrew only
  HOMEBREW_PREFIX="/opt"
else
  HOMEBREW_DIR="Homebrew"
  # On Intel macOS, this script installs to /usr/local only
  HOMEBREW_PREFIX="/usr/local"
fi

HOMEBREW_REPOSITORY="${HOMEBREW_PREFIX}/${HOMEBREW_DIR}"
HOMEBREW_CACHE="${HOME}/Library/Caches/Homebrew"
HOMEBREW_BREW_GIT_REMOTE="https://github.com/Homebrew/brew"

execute_admin() {
  local -a args="${@}"
  if [[ "${EUID:-${UID}}" != "0" ]]; then
    log osascript -e "do shell script \"${args}\" with administrator privileges"
    execute osascript -e "do shell script \"${args}\" with administrator privileges"
  else
    log "${args[@]}"
    execute "${args[@]}"
  fi
}

activate_app() {
  local -a args="${@}"
  log osascript -e "tell app \"${args}\" to activate"
  execute osascript -e "tell app \"${args}\" to activate"
}

log "Check developer command line tools"
if [[ "$(xcode-select -p 2>/dev/null)" == "" ]]; then
    log "Install developer command line tools"
    xcode-select --install 2>/dev/null

    clt_label=""
    while [[ "${clt_label}" == "" ]]; do
        sleep 2
        clt_label=$(softwareupdate -l | grep -B 1 -E 'Command Line Tools' | awk -F'*' '/^ *\*/ {print $2}' | sed -e 's/^ *Label: //' -e 's/^ *//' | sort -V | tail -n1)
    done

    # Activate app: Install Command Line Developer Tools
    activate_app "Install Command Line Developer Tools"

    if [[ -n "${clt_label}" ]]; then
      log "Installing ${clt_label}"
      softwareupdate -i "${clt_label}" || abort "Command Line Tools installation failed"
    fi
else
   log "Update developer command line tools"
   clt_label=$(softwareupdate -l | grep -B 1 -E 'Command Line Tools' | awk -F'*' '/^ *\*/ {print $2}' | sed -e 's/^ *Label: //' -e 's/^ *//' | sort -V | tail -n1)
   if [[ -n "${clt_label}" ]]; then
     log "Updating ${clt_label}"
     softwareupdate -i "${clt_label}" || abort "Command Line Tools update failed"
   fi
fi

XCODE_SELECT_PATH="$(xcode-select -p 2>/dev/null)"
if [[ "${XCODE_SELECT_PATH}" == "" ]]; then
    abort "Command Line Tools installation cancelled by user"
fi
log "Command Line Tools path: ${XCODE_SELECT_PATH}"

if [ ! -x "$(command -v $(which brew 2>/dev/null))" ]; then
  execute mkdir -p -m 700 ${CACHE_PATH}
  echo "#!/bin/bash" > ${SETUP_SCRIPT}
  echo "export PATH=${PATH}" > ${SETUP_SCRIPT}
  execute chmod +x ${SETUP_SCRIPT}

  log "Check developer command line tools path"
  if [[ "${XCODE_SELECT_PATH}" != "${CLT_PATH}" ]]; then
      log "Switch developer command line tools to path: ${CLT_PATH}"
      echo "" >> "${SETUP_SCRIPT}"
      echo "xcode-select --switch ${CLT_PATH}" >> "${SETUP_SCRIPT}"
  fi

  log "Preparing to Homebrew installation"
  if [[ ! -d "${HOMEBREW_REPOSITORY}" ]]; then
    # osascript "-e do shell script \"install -d -o $(id -un) -g admin -m 0755 /usr/local/Homebrew\" with administrator privileges"
    echo "" >> "${SETUP_SCRIPT}"
    echo "install -d -o ${USER} -g ${GROUP} -m 0755 ${HOMEBREW_REPOSITORY}" >> "${SETUP_SCRIPT}"
  fi

  log "Executing: ${SETUP_SCRIPT}"
  if [[ "${XCODE_SELECT_PATH}" != "${CLT_PATH}" ]] || [[ ! -d "${HOMEBREW_REPOSITORY}" ]]; then
    execute_admin "${SETUP_SCRIPT}"
  fi

  log "Cleaning cache: ${CACHE_PATH}"
  execute rm -rf ${CACHE_PATH}

  log "Downloading and installing Homebrew..."
  (
    cd "${HOMEBREW_REPOSITORY}" >/dev/null || return

    # we do it in four steps to avoid merge errors when reinstalling
    execute git -c init.defaultBranch=master init --quiet >/dev/null

    # "git remote add" will fail if the remote is defined in the global config
    execute git config remote.origin.url "${HOMEBREW_BREW_GIT_REMOTE}" >/dev/null
    execute git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*" >/dev/null

    # ensure we don't munge line endings on checkout
    execute git config --bool core.autocrlf false >/dev/null

    # make sure symlinks are saved as-is
    execute git config --bool core.symlinks true >/dev/null

    execute git fetch -q --force origin >/dev/null
    execute git fetch -q --force --tags origin >/dev/null
    execute git remote set-head origin --auto >/dev/null

    LATEST_GIT_TAG=$(git tag --list --sort="-version:refname" | head -n1)
    if [[ -z "${LATEST_GIT_TAG}" ]]; then
      abort "Failed to query latest Homebrew/brew Git tag."
    fi
    execute git checkout -q --force -B stable "${LATEST_GIT_TAG}" >/dev/null

    execute "${HOMEBREW_REPOSITORY}/bin/brew" update --force --quiet >/dev/null

    if [[ "${UNAME_MACHINE}" != "arm64" ]]; then
      execute ln -sf "${HOMEBREW_REPOSITORY}/bin/brew" "${HOMEBREW_PREFIX}/bin/brew" >/dev/null
    fi
  ) || abort "Downloading and installing Homebrew failed"

  add_to_path /usr/local/bin /opt/homebrew/bin
  check_command brew
fi

log "Fetching the newest version of Homebrew and installed packages"
brew update -q -f || warn "Failed to execute: brew update"

if brew list -q ${SONARIC_PACKAGE_NAME} 2>/dev/null; then
  log "Sonaric upgrading to the newest version"
  execute brew upgrade -q -f --skip-cask-deps ${SONARIC_PACKAGE_NAME}
else
  log "Sonaric newest version installation"
  execute brew install -q -f ${SONARIC_PACKAGE_NAME}
fi

check_command podman sonaric

log "Service ${SONARIC_SERVICE_NAME} stopping..."
execute brew services stop -q ${SONARIC_SERVICE_NAME}

log "Remove old versions, stale lock files and outdated downloads..."
execute brew cleanup -q

log "Service ${SONARIC_SERVICE_NAME} starting..."
execute brew services start -q ${SONARIC_SERVICE_NAME}

ITR=0
RUNNING=false
while [[ "$RUNNING" != "true" ]]; do
  ITR=$((ITR + 1))
  if sonaric ${SONARIC_OPTS} version; then
    RUNNING=true

    # If sonaric was started we should add some pause to become it ready
    log "Wait for Sonaric daemon to become ready..."
    sleep 15
  else
    log "${ITR}) Wait for Sonaric daemon to become started..."
    sleep 5
  fi
done

log "Podman machines checking..."
execute podman machine ls

log "Podman machine info checking..."
execute podman machine info

log "Service ${SONARIC_RUNTIME_SERVICE_NAME} checking..."
execute brew services info -q ${SONARIC_RUNTIME_SERVICE_NAME}

log "Service ${SONARIC_SERVICE_NAME} checking..."
execute brew services info -q ${SONARIC_SERVICE_NAME}

log "Sonaric workloads list..."
execute sonaric ${SONARIC_OPTS} ps -a

log "Sonaric workloads updating..."
execute sonaric ${SONARIC_OPTS} update --all

