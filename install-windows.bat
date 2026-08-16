@echo off
title Y++ Language 1.0 Installer
echo ========================================================
echo               Y++ Language 1.0 Installer
echo ========================================================
echo.
echo Compiling Y++ Engine and IDE...
if not exist "bin" mkdir "bin"
javac -encoding UTF-8 -d bin src\*.java

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Java compiler (javac) not found or compilation failed!
    echo Please make sure JDK 17 or higher is installed and added to PATH.
    pause
    exit /b %ERRORLEVEL%
)

echo Compilation successful!
echo.
echo Creating launcher shortcuts...
(
echo @echo off
echo java -cp "%%~dp0bin" ypp.Ypp %%*
) > ypp.bat

(
echo @echo off
echo start javaw -cp "%%~dp0bin" ypp.YppIDE
) > ypp-ide.bat

echo ========================================================
echo   Y++ 1.0 Installed Successfully!
echo.
echo   - To launch Y++ IDE: Double-click 'ypp-ide.bat'
echo   - To run Y++ CLI: Run 'ypp.bat myfile.ypp'
echo ========================================================
echo.
pause
