import Foundation
import Testing
@testable import VocabMacBridge

@Test func selectionPayloadUsesFrontendFieldNames() throws {
    let payload = SelectionPayload(selectedText: "serendipity", sentence: "A moment of serendipity.", sourceApp: "Safari", sourceTitle: nil, sourceUrl: "https://example.com", selectionBounds: BridgeRect(x: 10, y: 20, width: 80, height: 18), origin: "accessibility")
    let data = try JSONEncoder().encode(BridgeResponse.success(payload))
    let object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Any])
    let encoded = try #require(object["payload"] as? [String: Any])
    #expect(encoded["selectedText"] as? String == "serendipity")
    #expect(encoded["origin"] as? String == "accessibility")
}

@Test func ocrRequestPreservesExplicitPointerCoordinates() throws {
    let request = OcrCaptureRequest(x: 137.25, y: -48.5)
    let data = try JSONEncoder().encode(request)
    let object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Double])

    #expect(object["x"] == 137.25)
    #expect(object["y"] == -48.5)
}
