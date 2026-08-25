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

public enum OcrCapture {
    public static func captureNearPointer() async throws -> SelectionPayload {
        guard CGPreflightScreenCaptureAccess() else { throw OcrCaptureError.screenRecordingPermissionRequired }
        let pointer = NSEvent.mouseLocation
        let display = NSScreen.screens.first(where: { $0.frame.contains(pointer) }) ?? NSScreen.main
        guard let display,
              let number = display.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? CGDirectDisplayID else { throw OcrCaptureError.screenshotUnavailable }
        let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
        guard let sharedDisplay = content.displays.first(where: { $0.displayID == number }) else { throw OcrCaptureError.screenshotUnavailable }
        let filter = SCContentFilter(display: sharedDisplay, excludingWindows: [])
        let configuration = SCStreamConfiguration()
        configuration.width = sharedDisplay.width
        configuration.height = sharedDisplay.height
        let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: configuration)

        let request = VNRecognizeTextRequest()
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = true
        try VNImageRequestHandler(cgImage: image).perform([request])
        let observations = request.results ?? []
        let localX = (pointer.x - display.frame.minX) / display.frame.width
        let localY = (pointer.y - display.frame.minY) / display.frame.height
        guard let observation = observations.min(by: {
            distance($0.boundingBox, x: localX, y: localY) < distance($1.boundingBox, x: localX, y: localY)
        }), let text = observation.topCandidates(1).first?.string, !text.isEmpty else { throw OcrCaptureError.noTextFound }
        let bounds = observation.boundingBox
        return SelectionPayload(
            selectedText: text,
            sentence: text,
            sourceApp: NSWorkspace.shared.frontmostApplication?.localizedName,
            sourceTitle: nil,
            sourceUrl: nil,
            selectionBounds: BridgeRect(
                x: display.frame.minX + bounds.minX * display.frame.width,
                y: display.frame.minY + (1 - bounds.maxY) * display.frame.height,
                width: bounds.width * display.frame.width,
                height: bounds.height * display.frame.height
            ),
            origin: "ocr"
        )
    }

    private static func distance(_ rect: CGRect, x: Double, y: Double) -> Double {
        let dx = rect.midX - x
        let dy = rect.midY - y
        return dx * dx + dy * dy
    }
}
