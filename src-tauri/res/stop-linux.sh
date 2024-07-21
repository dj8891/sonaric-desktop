#!/bin/sh
set -e

command_exists() {
	command -v "$@" > /dev/null 2>&1
}

do_stop() {
	echo "# Stopping Sonaric daemon"

	user="$(id -un 2>/dev/null || true)"

	sh_c='sh -c'
	if [ "$user" != 'root' ]; then
		if command_exists sudo; then
			sh_c='sudo -E sh -c'
		elif command_exists su; then
			sh_c='su -c'
		else
			cat >&2 <<-'EOF'
			Error: this installer needs the ability to run commands as root.
			We are unable to find either "sudo" or "su" available to make this happen.
			EOF
			exit 1
		fi
	fi

  # stop and delete all workloads
  if command_exists sonaric; then
    $sh_c "sonaric stop --nocolor --nofancy -a"
  fi

	# check if systemctl unit is present and if it is active
	if command_exists systemctl && systemctl list-units --full --all sonaricd.service | grep -Fq 'loaded'; then
    $sh_c 'systemctl stop sonaricd'
  fi
}

do_stop
