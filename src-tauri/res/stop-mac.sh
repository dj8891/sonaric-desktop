#!/bin/bash

# We don't need return codes for "$(command)", only stdout is needed.
# Allow `[[ -n "$(command)" ]]`, `func "$(command)"`, pipes, etc.
# shellcheck disable=SC2312
set -u

export HOMEBREW_NO_COLOR=1
export HOMEBREW_NO_EMOJI=1
export HOMEBREW_NO_AUTO_UPDATE=1

NAME="Sonaric app"
SONARIC_OPTS="--nofancy --nocolor"
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

add_to_path /sbin /usr/sbin /usr/local/sbin
add_to_path /bin /usr/bin /usr/local/bin
add_to_path /opt/homebrew/bin

check_command brew uname podman sonaric

OS="$(uname 2>/dev/null)"
# Check if we are on mac
if [[ "${OS}" != "Darwin" ]]; then
  abort "${NAME} is only supported on macOS."
fi

log "Sonaric workloads stopping..."
sonaric ${SONARIC_OPTS} stop -a || warn "Failed during: sonaric ${SONARIC_OPTS} stop -a"

log "Sonaric workloads checking..."
sonaric ${SONARIC_OPTS} ps -a

log "Service ${SONARIC_SERVICE_NAME} stopping..."
brew services stop -q ${SONARIC_SERVICE_NAME} || warn "Failed during: brew services stop -q ${SONARIC_SERVICE_NAME}"

log "Podman containers stopping..."
podman stop -a || warn "Failed during: podman stop -a"

log "Podman containers checking..."
podman ps -a

log "Podman machine stopping..."
podman machine stop || warn "Failed during: podman machine stop"

log "Podman machines checking..."
podman machine ls

log "Podman machine info checking..."
podman machine info

log "Service ${SONARIC_RUNTIME_SERVICE_NAME} stopping..."
brew services stop -q ${SONARIC_RUNTIME_SERVICE_NAME} || warn "Failed during: brew services stop -q ${SONARIC_RUNTIME_SERVICE_NAME}"

