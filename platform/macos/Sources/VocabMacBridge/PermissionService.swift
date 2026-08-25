import ApplicationServices
import CoreGraphics
import Foundation

public enum NativePermissionStatus: String, Codable, Sendable { case granted, denied, notDetermined, restricted }

public enum PermissionService {
    public static func accessibilityStatus() -> NativePermissionStatus {
        AXIsProcessTrusted() ? .granted : .notDetermined
    }

    public static func requestAccessibility() {
        let options = ["AXTrustedCheckOptionPrompt": true] as CFDictionary
        _ = AXIsProcessTrustedWithOptions(options)
    }

    public static func screenRecordingStatus() -> NativePermissionStatus {
        CGPreflightScreenCaptureAccess() ? .granted : .notDetermined
    }
}
