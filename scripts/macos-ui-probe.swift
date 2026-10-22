// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
// CI-only AX/CGEvent investigation. Never requests or changes OS permissions.
import AppKit
import ApplicationServices
import Foundation

struct ProbeError: Error, CustomStringConvertible { let description: String }
func require(_ condition: Bool, _ message: String) throws {
    if !condition { throw ProbeError(description: message) }
}
let arguments = CommandLine.arguments
guard arguments.count == 3, let pid = Int32(arguments[1]),
      ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
      ProcessInfo.processInfo.environment["RUNNER_ENVIRONMENT"] == "github-hosted" else {
    fputs("CI-only native UI probe; never run on a personal desktop\n", stderr); exit(64)
}
let evidence = URL(fileURLWithPath: arguments[2], isDirectory: true)
let app = AXUIElementCreateApplication(pid)
func attribute(_ element: AXUIElement, _ name: CFString) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name, &value) == .success ? value : nil
}
func children(_ element: AXUIElement) -> [AXUIElement] {
    (attribute(element, kAXChildrenAttribute as CFString) as? [AXUIElement] ?? []) +
    (attribute(element, kAXWindowsAttribute as CFString) as? [AXUIElement] ?? [])
}
func names(_ element: AXUIElement) -> [String] {
    [kAXTitleAttribute, kAXDescriptionAttribute, kAXValueAttribute].compactMap {
        attribute(element, $0 as CFString) as? String
    }
}
func elements() -> [AXUIElement] {
    var queue = [app]; var index = 0
    while index < queue.count && queue.count < 3000 {
        queue.append(contentsOf: children(queue[index])); index += 1
    }
    return queue
}
func waitFor(_ name: String) throws -> AXUIElement {
    let deadline = Date().addingTimeInterval(20)
    repeat {
        if let element = elements().first(where: { names($0).contains(name) }) { return element }
        Thread.sleep(forTimeInterval: 0.2)
    } while Date() < deadline
    throw ProbeError(description: "AX control unavailable: \(name)")
}
func press(_ name: String) throws {
    try require(AXUIElementPerformAction(try waitFor(name), kAXPressAction as CFString) == .success,
                "AX press failed: \(name)")
}
func typeTest() throws {
    let input = try waitFor("Type the displayed text")
    try require(AXUIElementSetAttributeValue(app, kAXFrontmostAttribute as CFString, kCFBooleanTrue) == .success,
                "Cannot bring application to foreground")
    try require(AXUIElementSetAttributeValue(input, kAXFocusedAttribute as CFString, kCFBooleanTrue) == .success,
                "Cannot focus native typing target")
    for unit in "asdf".utf16 {
        guard let down = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true),
              let up = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: false) else {
            throw ProbeError(description: "Cannot create CGEvent")
        }
        var value = unit
        down.keyboardSetUnicodeString(stringLength: 1, unicodeString: &value)
        up.keyboardSetUnicodeString(stringLength: 1, unicodeString: &value)
        down.post(tap: .cghidEventTap); up.post(tap: .cghidEventTap)
        Thread.sleep(forTimeInterval: 0.2)
    }
    _ = try waitFor("Test Complete")
}
func pngFiles(_ directory: URL) -> [URL] {
    ((try? FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)) ?? [])
        .filter { $0.pathExtension == "png" }
}
do {
    try require(AXIsProcessTrusted(), "Accessibility controller is not trusted; no permission requested")
    try press("Texts"); try press("Start"); try typeTest()
    try press("Restart"); try typeTest()
    guard let pictures = FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first else {
        throw ProbeError(description: "No Pictures directory")
    }
    let gallery = pictures.appendingPathComponent("Raccoon Typer", isDirectory: true)
    let before = Set(pngFiles(gallery))
    NSPasteboard.general.clearContents()
    try press("Save PNG")
    let deadline = Date().addingTimeInterval(30)
    var exported: URL?; var image: NSBitmapImageRep?
    repeat {
        let new = pngFiles(gallery).filter { !before.contains($0) }
        if new.count == 1, let pasted = NSImage(pasteboard: .general),
           let tiff = pasted.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff) {
            exported = new[0]; image = bitmap; break
        }
        Thread.sleep(forTimeInterval: 0.2)
    } while Date() < deadline
    guard let file = exported, let bitmap = image,
          bitmap.pixelsWide >= 400, bitmap.pixelsHigh >= 200,
          let data = bitmap.representation(using: .png, properties: [:]) else {
        throw ProbeError(description: "PNG file or OS image clipboard missing")
    }
    try FileManager.default.copyItem(at: file, to: evidence.appendingPathComponent("saved.png"))
    try data.write(to: evidence.appendingPathComponent("clipboard.png"))
    try press("Restart"); _ = try waitFor("Type the displayed text")
    try press("Abort"); _ = try waitFor("Start")
    let result: [String: Any] = ["status": "passed", "input": "OS CGEvent Unicode asdf, twice",
        "abort": "returned to custom text list", "png_file": file.lastPathComponent,
        "clipboard_image_dimensions": [bitmap.pixelsWide, bitmap.pixelsHigh],
        "hardware_layout_dpi_paste_verified": false]
    try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
        .write(to: evidence.appendingPathComponent("native-ui.json"))
} catch {
    let dump = elements().map { names($0).joined(separator: " | ") }.joined(separator: "\n")
    try? dump.write(to: evidence.appendingPathComponent("ax-tree.txt"), atomically: true, encoding: .utf8)
    fputs("Native UI probe failed: \(error)\n", stderr); exit(1)
}
