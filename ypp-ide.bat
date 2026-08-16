@echo off
if not exist "%~dp0bin\ypp\YppIDE.class" (
    javac -encoding UTF-8 -d "%~dp0bin" "%~dp0src\*.java"
)
start javaw -cp "%~dp0bin" ypp.YppIDE
