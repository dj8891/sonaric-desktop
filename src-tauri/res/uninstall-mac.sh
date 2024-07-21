#!/bin/bash

# We don't need return codes for "$(command)", only stdout is needed.
# Allow `[[ -n "$(command)" ]]`, `func "$(command)"`, pipes, etc.
# shellcheck disable=SC2312
set -u

export HOMEBREW_NO_COLOR=1
export HOMEBREW_NO_EMOJI=1
export HOMEBREW_NO_AUTO_UPDATE=1

NAME="Sonaric uninstaller"
SONARIC_OPTS="--nofancy --nocolor"
SONARIC_PACKAGE_NAME="monk-io/sonaric/sonaric"
SONARIC_SERVICE_NAME="sonaric"
SONARIC_RUNTIME_SERVICE_NAME="sonaric-runtime"

log() {
  echo "$@"
}

warn() {
  echo "WARNING: $@"
}

abort() {
  echo "ERROR: $@" >&2
  exit 1
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

check_command rm tr uname

OS="$(uname 2>/dev/null)"
# Check if we are on mac
if [[ "${OS}" != "Darwin" ]]; then
  abort "${NAME} is only supported on macOS."
fi

export PATH="${PATH}"
log "${NAME} detects paths:"
for p in $(echo ${PATH} | tr ":" " "); do
  log " - ${p}"
done

if [ -x "$(command -v sonaric)" ]; then
  if [ -x "$(command -v brew)" ]; then
    if ! sonaric ${SONARIC_OPTS} version; then
      log "Service ${SONARIC_SERVICE_NAME} starting..."
      brew services start -q ${SONARIC_SERVICE_NAME} || warn "Failed during: brew services start -q ${SONARIC_SERVICE_NAME}"
      log "Wait for service ${SONARIC_SERVICE_NAME} to become ready..."
      sleep 5
    fi
  fi

  ITR=0
  TOTAL_ITRS=50
  RUNNING=false
  while [[ "$RUNNING" != "true" && ${ITR} -le ${TOTAL_ITRS} ]]; do
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

  log "Sonaric resources stopping..."
  if ! sonaric ${SONARIC_OPTS} stop -a; then
    warn "Sonaric resources stop failed"
    if [ -x "$(command -v podman)" ]; then
      log "Podman containers stopping..."
      if ! podman stop -a; then
        warn "Podman containers stop failed"
      fi
    fi
  fi

  log "Sonaric resources deleting..."
  if ! sonaric ${SONARIC_OPTS} delete -a --force; then
    warn "Sonaric resources delete failed"

    if [ -x "$(command -v podman)" ]; then
      log "Podman containers stopping..."
      if ! podman stop -a; then
        warn "Podman containers stop failed"
      fi

      log "Podman containers removing..."
      if ! podman rm -a -f -i; then
        warn "Podman containers remove failed"
      fi

      log "Podman containers cleanupping..."
      if ! podman container prune -f; then
        warn "Podman containers force prune failed"
      fi

      log "Podman system cleanupping..."
      if ! podman system prune -a -f; then
        warn "Podman system force prune failed"
      fi

      log "Sonaric resources cleanupping..."
      if ! sonaric ${SONARIC_OPTS} delete -a --force; then
        warn "Sonaric resources cleanup failed"
      fi
    fi
  fi
fi

if [ -x "$(command -v brew)" ]; then
  log "Service ${SONARIC_SERVICE_NAME} stopping..."
  brew services stop -q ${SONARIC_SERVICE_NAME} || warn "Failed during: brew services stop -q ${SONARIC_SERVICE_NAME}"
fi

if [ -x "$(command -v podman)" ]; then
  log "Podman containers stopping..."
  podman stop -a || warn "Failed during: podman stop -a"

  log "Podman machine stopping..."
  podman machine stop || warn "Failed during: podman machine stop"

  log "Podman machine removing..."
  podman machine rm -f || warn "Failed during: podman machine rm -f"
fi

if [ -x "$(command -v brew)" ]; then
  log "Service ${SONARIC_RUNTIME_SERVICE_NAME} stopping..."
  brew services stop -q ${SONARIC_RUNTIME_SERVICE_NAME} || warn "Failed during: brew services stop -q ${SONARIC_RUNTIME_SERVICE_NAME}"

  log "Sonaric uninstalling..."
  brew uninstall -q -f --ignore-dependencies ${SONARIC_PACKAGE_NAME} || warn "Failed during: brew uninstall -q -f --ignore-dependencies ${SONARIC_PACKAGE_NAME}"
fi

if [ ! -x "$(command -v podman)" ]; then
  log "Podman has already uninstalled from your system"
  rm -rf ~/.local/share/containers || warn "Failed during: rm -rf ~/.local/share/containers"
  rm -rf ~/.config/containers || warn "Failed during: rm -rf ~/.config/containers"
fi

