// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
// Window-server evidence only: no screen contents or Accessibility permission.
import CoreGraphics
import Foundation
guard CommandLine.arguments.count == 2,
      let pid = Int(CommandLine.arguments[1]),
      let windows = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] else {
    exit(2)
}
let visible = windows.contains { window in
    guard let owner = window[kCGWindowOwnerPID as String] as? Int,
          owner == pid,
          window[kCGWindowIsOnscreen as String] as? Bool == true,
          let bounds = window[kCGWindowBounds as String] as? [String: Any],
          let width = bounds["Width"] as? Double,
          let height = bounds["Height"] as? Double else { return false }
    return width >= 400 && height >= 300
}
exit(visible ? 0 : 1)
