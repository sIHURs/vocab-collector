import Darwin
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
    let semaphore = DispatchSemaphore(value: 0)
    let box = AsyncResultBox<SelectionPayload>()
    Task.detached {
        do { box.result = .success(try await OcrCapture.captureNearPointer()) }
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
