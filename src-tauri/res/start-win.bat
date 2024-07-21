@echo off

set "tmpWslVersion=%temp%\wsl-version-%random%.tmp"
wsl --version > %tmpWslVersion% 2> nul
if %errorlevel% neq 0 (
	exit 0
)

set "wslList=%temp%\wsl-list-%random%.tmp"
wsl --list > %wslList%
find "Ubuntu-22.04" %wslList% > nul
if %errorlevel% neq 0 (
	exit 0
)

wsl --list --running > %wslList%
find "Ubuntu-22.04" %wslList% > nul
if %errorlevel% neq 0 (
    wsl --distribution Ubuntu-22.04 --exec dbus-launch true
    exit 0
)


