import ApplicationServices
import CoreGraphics
import Foundation

public enum NativePermissionStatus: String, Codable, Sendable { case granted, denied, notDetermined, restricted }

public enum PermissionService {
    private static let accessibilityRequested = "vocab.accessibilityRequested"
    private static let screenRecordingRequested = "vocab.screenRecordingRequested"

    public static func accessibilityStatus() -> NativePermissionStatus {
        if AXIsProcessTrusted() { return .granted }
        return UserDefaults.standard.bool(forKey: accessibilityRequested) ? .denied : .notDetermined
    }

    public static func requestAccessibility() {
        UserDefaults.standard.set(true, forKey: accessibilityRequested)
        let options = ["AXTrustedCheckOptionPrompt": true] as CFDictionary
        _ = AXIsProcessTrustedWithOptions(options)
    }

    public static func screenRecordingStatus() -> NativePermissionStatus {
        if CGPreflightScreenCaptureAccess() { return .granted }
        return UserDefaults.standard.bool(forKey: screenRecordingRequested) ? .denied : .notDetermined
    }

    public static func requestScreenRecording() {
        UserDefaults.standard.set(true, forKey: screenRecordingRequested)
        _ = CGRequestScreenCaptureAccess()
    }
}
