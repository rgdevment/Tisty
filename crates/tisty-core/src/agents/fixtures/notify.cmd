@echo off
rem Tell the person a turn ended
powershell -NoProfile -Command "[console]::beep(880,200)"
exit /b 0
