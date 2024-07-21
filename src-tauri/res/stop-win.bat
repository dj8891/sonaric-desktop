@echo off

REM ------------------------------------------------------
REM Sonaric bash stop script
setlocal EnableDelayedExpansion
set LF=^


REM The above 2 empty lines are required - do not remove
set stopScript=!LF! ^
sonaric stop --nofancy --nocolor -a !LF! ^
systemctl stop sonaricd !LF! ^
echo \"Sonaric stopped on WSL\" !LF!
REM End of script
REM ------------------------------------------------------



echo Stopping Sonaric...
wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "!stopScript!"
if %errorlevel% neq 0 (
	echo Failed to stop Sonaric.
	exit 1
)

echo Sonaric stopped
