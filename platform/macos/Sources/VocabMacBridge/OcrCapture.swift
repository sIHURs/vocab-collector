import AppKit
import CoreGraphics
import Foundation
import ScreenCaptureKit
import Vision

public enum OcrCaptureError: Error, CustomStringConvertible {
    case screenRecordingPermissionRequired, screenshotUnavailable, noTextFound
    public var description: String {
        switch self { case .screenRecordingPermissionRequired: "screenRecordingPermissionRequired"; case .screenshotUnavailable: "screenshotUnavailable"; case .noTextFound: "noTextFound" }
    }
}

public struct OcrCaptureRequest: Codable, Equatable, Sendable {
    public let x: Double
    public let y: Double

    public init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }
}

public enum OcrCapture {
    public static func captureNearPointer() async throws -> SelectionPayload {
        try await capture(near: NSEvent.mouseLocation)
    }

    public static func capture(near request: OcrCaptureRequest) async throws -> SelectionPayload {
        try await capture(near: CGPoint(x: request.x, y: request.y))
    }

    private static func capture(near pointer: CGPoint) async throws -> SelectionPayload {
        guard CGPreflightScreenCaptureAccess() else { throw OcrCaptureError.screenRecordingPermissionRequired }
        let display = NSScreen.screens.first(where: { $0.frame.contains(pointer) }) ?? NSScreen.main
        guard let display,
              let number = display.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? CGDirectDisplayID else { throw OcrCaptureError.screenshotUnavailable }
        let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
        guard let sharedDisplay = content.displays.first(where: { $0.displayID == number }) else { throw OcrCaptureError.screenshotUnavailable }
        let ownWindows = content.windows.filter { $0.owningApplication?.bundleIdentifier == Bundle.main.bundleIdentifier }
        let filter = SCContentFilter(display: sharedDisplay, excludingWindows: ownWindows)
        let configuration = SCStreamConfiguration()
        let relativeX = pointer.x - display.frame.minX
        let relativeY = display.frame.maxY - pointer.y
        let regionWidth = min(900.0, display.frame.width)
        let regionHeight = min(420.0, display.frame.height)
        let region = CGRect(
            x: max(0, min(relativeX - regionWidth / 2, display.frame.width - regionWidth)),
            y: max(0, min(relativeY - regionHeight / 2, display.frame.height - regionHeight)),
            width: regionWidth,
            height: regionHeight
        )
        let scale = Double(sharedDisplay.width) / display.frame.width
        configuration.sourceRect = region
        configuration.width = Int(region.width * scale)
        configuration.height = Int(region.height * scale)
        let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: configuration)

        let request = VNRecognizeTextRequest()
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = true
        try VNImageRequestHandler(cgImage: image).perform([request])
        let observations = request.results ?? []
        let localX = (relativeX - region.minX) / region.width
        let localY = 1 - (relativeY - region.minY) / region.height
        guard let observation = observations.filter({ $0.confidence >= 0.55 }).min(by: {
            distance($0.boundingBox, x: localX, y: localY) < distance($1.boundingBox, x: localX, y: localY)
        }), let recognized = observation.topCandidates(1).first else { throw OcrCaptureError.noTextFound }
        let (text, bounds) = nearestWord(in: recognized, x: localX, y: localY) ?? (recognized.string, observation.boundingBox)
        guard !text.isEmpty else { throw OcrCaptureError.noTextFound }
        return SelectionPayload(
            selectedText: text,
            sentence: recognized.string,
            sourceApp: NSWorkspace.shared.frontmostApplication?.localizedName,
            sourceTitle: nil,
            sourceUrl: nil,
            selectionBounds: BridgeRect(
                x: display.frame.minX + region.minX + bounds.minX * region.width,
                y: display.frame.minY + region.minY + (1 - bounds.maxY) * region.height,
                width: bounds.width * region.width,
                height: bounds.height * region.height
            ),
            origin: "ocr"
        )
    }

    private static func distance(_ rect: CGRect, x: Double, y: Double) -> Double {
        let dx = rect.midX - x
        let dy = rect.midY - y
        return dx * dx + dy * dy
    }

    private static func nearestWord(in recognized: VNRecognizedText, x: Double, y: Double) -> (String, CGRect)? {
        let text = recognized.string
        var candidates: [(String, CGRect)] = []
        text.enumerateSubstrings(in: text.startIndex..<text.endIndex, options: .byWords) { word, range, _, _ in
            guard let word, let box = try? recognized.boundingBox(for: range) else { return }
            candidates.append((word, box.boundingBox))
        }
        return candidates.min { distance($0.1, x: x, y: y) < distance($1.1, x: x, y: y) }
    }
}
