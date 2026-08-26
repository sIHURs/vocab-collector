import Foundation
import Testing
@testable import VocabMacBridge

@Test func translationPayloadUsesPortableFieldNames() throws {
    let payload = TranslationPayload(translatedText: "Zufall", sourceLanguage: "en", targetLanguage: "de")
    let data = try JSONEncoder().encode(BridgeResponse.success(payload))
    let object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Any])
    let encoded = try #require(object["payload"] as? [String: Any])
    #expect(encoded["translatedText"] as? String == "Zufall")
    #expect(encoded["targetLanguage"] as? String == "de")
}
