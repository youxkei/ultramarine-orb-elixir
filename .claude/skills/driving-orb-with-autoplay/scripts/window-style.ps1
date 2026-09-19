# What the running game's window actually is, read from outside it: the style, the extended style, whether
# it is visible, and where it is.
#
# Which is the one question orb's own log cannot answer — orb says what it *asked* for, and a style the
# game rewrote afterwards reads the same in that log either way. `WS_EX_DLGMODALFRAME` is 0x00000001 and
# `WS_EX_TOOLWINDOW` 0x00000080, the two `orb_core::window::DIALOG_EX_STYLE` adds; `WS_VISIBLE` is
# 0x10000000 of the style.
#
# Usage: powershell.exe -NoProfile -ExecutionPolicy Bypass -File window-style.ps1 -Exe th06nc.exe
param([Parameter(Mandatory = $true)][string]$Exe)

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class OrbWindow {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int left, top, right, bottom; }
    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr GetWindowLongPtrW(IntPtr window, int which);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr window, out RECT rect);
}
"@

$name = [System.IO.Path]::GetFileNameWithoutExtension($Exe)
$found = $false
foreach ($process in Get-Process -Name $name -ErrorAction SilentlyContinue) {
    $window = $process.MainWindowHandle
    if ($window -eq [IntPtr]::Zero) { continue }
    $found = $true
    $style = [int64][OrbWindow]::GetWindowLongPtrW($window, -16)
    $extended = [int64][OrbWindow]::GetWindowLongPtrW($window, -20)
    $rect = New-Object OrbWindow+RECT
    [void][OrbWindow]::GetWindowRect($window, [ref]$rect)
    $dialog = ($extended -band 0x1) -ne 0
    $tool = ($extended -band 0x80) -ne 0
    $visible = [OrbWindow]::IsWindowVisible($window)
    "pid {0} hwnd 0x{1:x} style 0x{2:x8} exstyle 0x{3:x8} dialog-frame {4} off-the-taskbar {5} visible {6} at {7},{8} {9}x{10}" -f `
        $process.Id, [int64]$window, $style, $extended, $dialog, $tool, $visible, `
        $rect.left, $rect.top, ($rect.right - $rect.left), ($rect.bottom - $rect.top)
}
if (-not $found) { "no window of $Exe is up" }
