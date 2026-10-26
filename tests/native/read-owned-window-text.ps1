param([Parameter(Mandatory=$true)][int]$AppProcessId)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class OpenRecallStartupWindows {
  private delegate bool Visitor(IntPtr window, IntPtr state);
  [DllImport("user32.dll")] private static extern bool EnumWindows(Visitor visitor, IntPtr state);
  [DllImport("user32.dll")] private static extern bool EnumChildWindows(IntPtr parent, Visitor visitor, IntPtr state);
  [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
  [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, StringBuilder text, int limit);
  public static string[] Read(int processId) {
    var texts = new List<string>();
    Action<IntPtr> collect = window => { var text = new StringBuilder(4096); GetWindowText(window, text, text.Capacity); if (text.Length > 0) texts.Add(text.ToString()); };
    EnumWindows((window, state) => {
      uint owner; GetWindowThreadProcessId(window, out owner);
      if (owner == processId && IsWindowVisible(window)) {
        collect(window);
        EnumChildWindows(window, (child, unused) => { collect(child); return true; }, IntPtr.Zero);
      }
      return true;
    }, IntPtr.Zero);
    return texts.ToArray();
  }
}
'@
ConvertTo-Json -Compress -InputObject @([OpenRecallStartupWindows]::Read($AppProcessId))
