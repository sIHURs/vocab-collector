import Darwin
import AppKit
import Foundation

public struct BridgeRect: Codable, Equatable, Sendable {
    public let x: Double
    public let y: Double
    public let width: Double
    public let height: Double
}

public struct SelectionPayload: Codable, Equatable, Sendable {
    public let selectedText: String
    public let sentence: String
    public let sourceApp: String?
    public let sourceTitle: String?
    public let sourceUrl: String?
    public let selectionBounds: BridgeRect?
    public let origin: String
}

public struct BridgeResponse<Payload: Codable & Sendable>: Codable, Sendable {
    public let ok: Bool
    public let payload: Payload?
    public let error: String?

    public static func success(_ payload: Payload) -> Self { .init(ok: true, payload: payload, error: nil) }
    public static func failure(_ error: String) -> Self { .init(ok: false, payload: nil, error: error) }
}

enum BridgeJSON {
    static func pointer<T: Encodable>(for value: T) -> UnsafeMutablePointer<CChar>? {
        guard let data = try? JSONEncoder().encode(value), let json = String(data: data, encoding: .utf8) else { return nil }
        return strdup(json)
    }
}

private final class AsyncResultBox<Value>: @unchecked Sendable { var result: Result<Value, Error>? }
private final class AsyncValueBox<Value>: @unchecked Sendable { var value: Value? }

@_cdecl("vocab_mac_permission_status")
public func vocabMacPermissionStatus(_ kind: Int32) -> UnsafeMutablePointer<CChar>? {
    let status = kind == 0 ? PermissionService.accessibilityStatus() : PermissionService.screenRecordingStatus()
    return BridgeJSON.pointer(for: BridgeResponse<String>.success(status.rawValue))
}

@_cdecl("vocab_mac_request_accessibility")
public func vocabMacRequestAccessibility() -> UnsafeMutablePointer<CChar>? {
    PermissionService.requestAccessibility()
    return BridgeJSON.pointer(for: BridgeResponse<String>.success(PermissionService.accessibilityStatus().rawValue))
}

@_cdecl("vocab_mac_capture_selection")
public func vocabMacCaptureSelection() -> UnsafeMutablePointer<CChar>? {
    do { return BridgeJSON.pointer(for: BridgeResponse<SelectionPayload>.success(try AccessibilityCapture.capture())) }
    catch { return BridgeJSON.pointer(for: BridgeResponse<SelectionPayload>.failure(String(describing: error))) }
}

@_cdecl("vocab_mac_request_screen_recording")
public func vocabMacRequestScreenRecording() -> UnsafeMutablePointer<CChar>? {
    PermissionService.requestScreenRecording()
    return BridgeJSON.pointer(for: BridgeResponse<String>.success(PermissionService.screenRecordingStatus().rawValue))
}

@_cdecl("vocab_mac_capture_ocr")
public func vocabMacCaptureOcr() -> UnsafeMutablePointer<CChar>? {
    captureOcr(OcrCaptureRequest(x: NSEvent.mouseLocation.x, y: NSEvent.mouseLocation.y))
}

@_cdecl("vocab_mac_capture_ocr_at")
public func vocabMacCaptureOcrAt(_ x: Double, _ y: Double) -> UnsafeMutablePointer<CChar>? {
    captureOcr(OcrCaptureRequest(x: x, y: y))
}

private func captureOcr(_ request: OcrCaptureRequest) -> UnsafeMutablePointer<CChar>? {
    let semaphore = DispatchSemaphore(value: 0)
    let box = AsyncResultBox<SelectionPayload>()
    Task.detached {
        do { box.result = .success(try await OcrCapture.capture(near: request)) }
        catch { box.result = .failure(error) }
        semaphore.signal()
    }
    semaphore.wait()
    switch box.result {
    case .success(let payload): return BridgeJSON.pointer(for: BridgeResponse<SelectionPayload>.success(payload))
    case .failure(let error): return BridgeJSON.pointer(for: BridgeResponse<SelectionPayload>.failure(String(describing: error)))
    case nil: return BridgeJSON.pointer(for: BridgeResponse<SelectionPayload>.failure("ocrUnavailable"))
    }
}

@_cdecl("vocab_mac_free_string")
public func vocabMacFreeString(_ pointer: UnsafeMutablePointer<CChar>?) { free(pointer) }

@_cdecl("vocab_mac_translate")
public func vocabMacTranslate(_ text: UnsafePointer<CChar>, _ source: UnsafePointer<CChar>, _ target: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>? {
    let semaphore = DispatchSemaphore(value: 0)
    let box = AsyncValueBox<TranslationOutcome>()
    let input = String(cString: text)
    let sourceIdentifier = String(cString: source)
    let targetIdentifier = String(cString: target)
    Task { @MainActor in
        TranslationHost.translate(text: input, sourceIdentifier: sourceIdentifier, targetIdentifier: targetIdentifier) { result in
            box.value = result
            semaphore.signal()
        }
    }
    guard semaphore.wait(timeout: .now() + 60) == .success else {
        return BridgeJSON.pointer(for: BridgeResponse<TranslationPayload>.failure("translationTimedOut"))
    }
    switch box.value {
    case .success(let payload): return BridgeJSON.pointer(for: BridgeResponse<TranslationPayload>.success(payload))
    case .failure(let error): return BridgeJSON.pointer(for: BridgeResponse<TranslationPayload>.failure(error))
    case nil: return BridgeJSON.pointer(for: BridgeResponse<TranslationPayload>.failure("translationUnavailable"))
    }
}

@_cdecl("vocab_mac_configure_capture_window")
public func vocabMacConfigureCaptureWindow() -> UnsafeMutablePointer<CChar>? {
    if Thread.isMainThread {
        MainActor.assumeIsolated { configureCaptureWindow() }
    } else {
        DispatchQueue.main.sync { MainActor.assumeIsolated { configureCaptureWindow() } }
    }
    return BridgeJSON.pointer(for: BridgeResponse<String>.success("configured"))
}

@MainActor
private func configureCaptureWindow() {
    guard let window = NSApplication.shared.windows.first(where: { $0.title == "Capture" }) else { return }
    window.level = .floating
    window.hidesOnDeactivate = false
    window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary]
    window.styleMask.insert(.nonactivatingPanel)
}
