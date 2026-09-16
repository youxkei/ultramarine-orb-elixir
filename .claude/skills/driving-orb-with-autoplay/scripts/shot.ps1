# A picture of the window of a game orb has no drawing hook in. th06c draws through a DXGI swapchain, so
# `BitBlt` off the screen and `CopyFromScreen` both come back black; `PrintWindow` with
# `PW_RENDERFULLCONTENT` asks the window itself to render into a bitmap.
#
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File shot.ps1 -Exe th06c.exe -Out C:\path\to.png
param(
  [Parameter(Mandatory = $true)][string]$Exe,
  [Parameter(Mandatory = $true)][string]$Out
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Shot {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct Rect { public int Left, Top, Right, Bottom; }
}
"@

# Aware of the DPI, or the bitmap is the window scaled down and the picture comes out part-empty.
[Shot]::SetProcessDPIAware() | Out-Null

$name = [System.IO.Path]::GetFileNameWithoutExtension($Exe)
$process = Get-Process -Name $name -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $process) {
  Write-Output "no window of $Exe to take a picture of"
  exit 1
}

$hwnd = $process.MainWindowHandle
$rect = New-Object Shot+Rect
[Shot]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top
$bitmap = New-Object System.Drawing.Bitmap $width, $height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$hdc = $graphics.GetHdc()
# 2 is PW_RENDERFULLCONTENT, which is what makes a swapchain's own content come out at all.
$ok = [Shot]::PrintWindow($hwnd, $hdc, 2)
$graphics.ReleaseHdc($hdc)
$graphics.Dispose()
if (-not $ok) {
  Write-Output "the window of $Exe would not render"
  exit 1
}
$bitmap.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$bitmap.Dispose()
Write-Output "took a $width by $height picture of $Exe into $Out"
