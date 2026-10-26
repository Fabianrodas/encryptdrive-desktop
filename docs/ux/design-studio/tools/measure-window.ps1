param([string]$Exe, [string[]]$AppArgs, [int]$WaitMs = 2500)
# Launches the app and reports, for its own top-level window only: window rect, client
# rect, the native minimum track size (WM_GETMINMAXINFO, answered by winit), the monitor
# work area, and the effective DPI. No mouse or keyboard input is sent.
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class WF {
  public struct RECT { public int L, T, R, B; }
  public struct POINT { public int X, Y; }
  public struct MINMAXINFO { public POINT Reserved, MaxSize, MaxPosition, MinTrack, MaxTrack; }
  [StructLayout(LayoutKind.Sequential)] public struct MONITORINFO { public int Size; public RECT Monitor, Work; public uint Flags; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EP cb, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, ref MINMAXINFO l);
  [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr h, uint flags);
  [DllImport("user32.dll")] public static extern bool GetMonitorInfoW(IntPtr m, ref MONITORINFO i);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
  public static IntPtr Find(uint pid) { IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p); var c = new StringBuilder(64); GetClassName(h, c, 64);
      if (p == pid && c.ToString() == "Window Class" && IsWindowVisible(h) && !IsIconic(h)) { found = h; } return true; }, IntPtr.Zero);
    return found; }
}
"@
# Per-monitor-aware v2 for this thread, so every value is in physical pixels.
[WF]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$p = Start-Process -FilePath $Exe -ArgumentList $AppArgs -PassThru
try {
  $deadline = (Get-Date).AddSeconds(15); $h = [IntPtr]::Zero
  while ($h -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200; $h = [WF]::Find([uint32]$p.Id) }
  if ($h -eq [IntPtr]::Zero) { throw "NO_APP_WINDOW" }
  Start-Sleep -Milliseconds $WaitMs
  $w = New-Object WF+RECT; [WF]::GetWindowRect($h, [ref]$w) | Out-Null
  $c = New-Object WF+RECT; [WF]::GetClientRect($h, [ref]$c) | Out-Null
  $mm = New-Object WF+MINMAXINFO; [WF]::SendMessageW($h, 0x0024, [IntPtr]::Zero, [ref]$mm) | Out-Null
  $mi = New-Object WF+MONITORINFO; $mi.Size = [Runtime.InteropServices.Marshal]::SizeOf($mi)
  [WF]::GetMonitorInfoW([WF]::MonitorFromWindow($h, 2), [ref]$mi) | Out-Null
  $dpi = [WF]::GetDpiForWindow($h)
  $inside = ($w.L -ge $mi.Work.L) -and ($w.T -ge $mi.Work.T) -and ($w.R -le $mi.Work.R) -and ($w.B -le $mi.Work.B)
  [pscustomobject]@{
    args = ($AppArgs -join ' '); dpi = $dpi
    work = "$($mi.Work.L),$($mi.Work.T)-$($mi.Work.R),$($mi.Work.B)"
    window = "$($w.L),$($w.T)-$($w.R),$($w.B) ($($w.R-$w.L)x$($w.B-$w.T))"
    client = "$($c.R)x$($c.B)"
    min_track_outer = "$($mm.MinTrack.X)x$($mm.MinTrack.Y)"
    inside_work_area = $inside
    responding = $p.Responding
  }
} finally { if (-not $p.HasExited) { $p.Kill() } }
