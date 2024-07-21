@echo off

REM ------------------------------------------------------
REM Sonaric bash uninstall script
setlocal EnableDelayedExpansion
set LF=^


REM The above 2 empty lines are required - do not remove
set uninstallScript=!LF! ^
if command -v sonaric ^> /dev/null 2^>^&1; then !LF! ^
	systemctl start sonaricd !LF! ^
	for try in {1..30} ; do !LF! ^
    	sonaric version ^> /dev/null 2^>^&1 ^&^& break !LF! ^
		sleep 2 !LF! ^
	done !LF! ^
	sonaric stop --nofancy --nocolor -a !LF! ^
	sonaric delete --nofancy --nocolor -a --force !LF! ^
fi !LF! ^
DEBIAN_FRONTEND=noninteractive apt-get remove --auto-remove -y -qq sonaricd !LF! ^
rm -f /etc/apt/sources.list.d/sonaric.list !LF! ^
rm -f /etc/apt/keyrings/sonaric.gpg !LF! ^
echo \"Sonaric uninstalled on WSL\" !LF!
REM End of script
REM ------------------------------------------------------



echo Uninstalling Sonaric...
wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "!uninstallScript!"
if %errorlevel% neq 0 (
	echo Failed to uninstall Sonaric.
	exit 1
)

set "startupScript=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\start-win.bat"
if exist "%startupScript%" (
    del "%startupScript%"
)

echo Sonaric uninstalled
