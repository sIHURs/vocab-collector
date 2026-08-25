import AppKit
import ApplicationServices
import Foundation

public enum AccessibilityCaptureError: Error, CustomStringConvertible {
    case permissionRequired, noFocusedElement, noSelection
    public var description: String {
        switch self { case .permissionRequired: "accessibilityPermissionRequired"; case .noFocusedElement: "noFocusedElement"; case .noSelection: "noSelection" }
    }
}

public enum AccessibilityCapture {
    public static func capture() throws -> SelectionPayload {
        guard AXIsProcessTrusted() else { throw AccessibilityCaptureError.permissionRequired }
        let system = AXUIElementCreateSystemWide()
        var focused: CFTypeRef?
        guard AXUIElementCopyAttributeValue(system, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
              let focused else { throw AccessibilityCaptureError.noFocusedElement }
        let element = focused as! AXUIElement
        let selected = stringAttribute(element, kAXSelectedTextAttribute)
        guard let selected, !selected.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { throw AccessibilityCaptureError.noSelection }

        let fullText = stringAttribute(element, kAXValueAttribute)
        let selectedRange = rangeAttribute(element)
        let sentence = context(fullText: fullText, selected: selected, range: selectedRange)
        let app = NSWorkspace.shared.frontmostApplication
        return SelectionPayload(
            selectedText: selected,
            sentence: sentence,
            sourceApp: app?.localizedName,
            sourceTitle: stringAttribute(element, kAXTitleAttribute),
            sourceUrl: stringAttribute(element, kAXDocumentAttribute),
            selectionBounds: boundsAttribute(element, range: selectedRange),
            origin: "accessibility"
        )
    }

    private static func stringAttribute(_ element: AXUIElement, _ attribute: String) -> String? {
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, attribute as CFString, &value) == .success else { return nil }
        return value as? String
    }

    private static func rangeAttribute(_ element: AXUIElement) -> CFRange? {
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, kAXSelectedTextRangeAttribute as CFString, &value) == .success,
              let value, CFGetTypeID(value) == AXValueGetTypeID() else { return nil }
        var range = CFRange()
        return AXValueGetValue(value as! AXValue, .cfRange, &range) ? range : nil
    }

    private static func boundsAttribute(_ element: AXUIElement, range: CFRange?) -> BridgeRect? {
        guard var range else { return nil }
        guard let rangeValue = AXValueCreate(.cfRange, &range) else { return nil }
        var value: CFTypeRef?
        guard AXUIElementCopyParameterizedAttributeValue(element, kAXBoundsForRangeParameterizedAttribute as CFString, rangeValue, &value) == .success,
              let value else { return nil }
        var rect = CGRect.zero
        guard AXValueGetValue(value as! AXValue, .cgRect, &rect) else { return nil }
        return BridgeRect(x: rect.origin.x, y: rect.origin.y, width: rect.width, height: rect.height)
    }

    private static func context(fullText: String?, selected: String, range: CFRange?) -> String {
        guard let fullText, let range,
              let swiftRange = Range(NSRange(location: range.location, length: range.length), in: fullText) else { return selected.trimmingCharacters(in: .whitespacesAndNewlines) }
        return SentenceExtractor.sentence(in: fullText, selection: swiftRange)
    }
}
