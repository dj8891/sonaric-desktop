@echo off

REM ------------------------------------------------------
REM Sonaric bash install script
setlocal EnableDelayedExpansion
set LF=^


REM The above 2 empty lines are required - do not remove
set installScript=set -e !LF! ^
install -m 0755 -d /etc/apt/keyrings !LF! ^
curl -fsSL https://us-central1-apt.pkg.dev/doc/repo-signing-key.gpg ^| gpg --dearmor --yes -o /etc/apt/keyrings/sonaric.gpg !LF! ^
chmod a+r /etc/apt/keyrings/sonaric.gpg !LF! ^
echo \"deb [arch=amd64 signed-by=/etc/apt/keyrings/sonaric.gpg] https://us-central1-apt.pkg.dev/projects/sonaric-platform sonaric-releases-apt main\" ^> /etc/apt/sources.list.d/sonaric.list !LF! ^
apt-get update !LF! ^
DEBIAN_FRONTEND=noninteractive apt-get install -y sonaric !LF! ^
echo \"Sonaric installed on WSL\" !LF!
REM End of script
REM ------------------------------------------------------

REM ------------------------------------------------------
REM Sonaric bash update script
setlocal EnableDelayedExpansion
set LF=^


REM The above 2 empty lines are required - do not remove
set updateScript=!LF! ^
apt-get update !LF! ^
DEBIAN_FRONTEND=noninteractive apt-get install -yy sonaric sonaricd !LF! ^
if command -v sonaric ^> /dev/null 2^>^&1; then !LF! ^
    systemctl start sonaricd !LF! ^
	for try in {1..30} ; do !LF! ^
    	sonaric version ^> /dev/null 2^>^&1 ^&^& break !LF! ^
		sleep 2 !LF! ^
	done !LF! ^
	sonaric update --nocolor --nofancy --all !LF! ^
fi !LF! ^
echo \"Sonaric updated on WSL\" !LF!
REM End of script
REM ------------------------------------------------------

REM check for NVIDIA drivers
echo Checking NVIDIA drivers...
set "tmpNvidia=%temp%\nvidia-%random%.tmp"
nvidia-smi > %tmpNvidia% 2> nul
if %errorlevel% neq 0 (
	echo It looks like NVIDIA drivers are not installed. NVIDIA drivers are required for GPU support in Sonaric. If you have an NVIDIA GPU and wish to use Sonaric with GPU support, please install latest NVIDIA drivers from NVIDIA website (https://www.nvidia.com/Download/index.aspx^) and try again, or proceed without GPU support.
)

REM check if WSL is installed and if it is WSL 2
echo Checking WSL...
set "tmpWslVersion=%temp%\wsl-version-%random%.tmp"
wsl --version > %tmpWslVersion% 2> nul
if %errorlevel% neq 0 (
	echo It looks like WSL is not installed. Please install WSL from Microsoft Store (https://aka.ms/wslstorepage^) and try again.
	exit 1
)

echo Ensuring Ubuntu-22.04 is installed...
REM check if Ubuntu-22.04 is installed
REM if not, install it
set "wslInstall=%temp%\wsl-install-%random%.tmp"
set "wslList=%temp%\wsl-list-%random%.tmp"
wsl --list > %wslList%
find "Ubuntu-22.04" %wslList% > nul
if %errorlevel% neq 0 (
	wsl --install Ubuntu-22.04 --no-launch > %wslInstall%
	ubuntu2204 install --root >> %wslInstall%
	if !errorlevel! neq 0 (
		echo Failed to install WSL distribution. A system reboot may be required.
		exit 1
	)
	wsl --distribution Ubuntu-22.04 --exec dbus-launch true
	timeout /t 10 > nul
)

wsl --list --running > %wslList%
find "Ubuntu-22.04" %wslList% > nul
if %errorlevel% neq 0 (
    wsl --distribution Ubuntu-22.04 --exec dbus-launch true
	timeout /t 10 > nul
)

set "startupDir=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup"

REM check if Sonaric is already installed
wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "dpkg-query -W sonaric" > nul
if %errorlevel% equ 0 (
	echo Sonaric is already installed. Updating...
	wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "!updateScript!"
	if !errorlevel! neq 0 (
		echo Failed to update Sonaric Node. Please check the error message above and try again, or contact support.
		exit 1
	)
	if exist "%startupDir%" (
    	copy "%0\..\start-win.bat" "%startupDir%" > nul
	)
	echo Sonaric updated
	exit 0
)

echo Installing Sonaric...
wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "!installScript!"
if %errorlevel% neq 0 (
	echo Failed to install Sonaric. Please check the error message above and try again, or contact support.
	exit 1
)
wsl -d Ubuntu-22.04 --user root --exec /bin/bash -c "systemctl start sonaricd"
if exist "%startupDir%" (
    copy "%0\..\start-win.bat" "%startupDir%" > nul
)
echo Sonaric installed
