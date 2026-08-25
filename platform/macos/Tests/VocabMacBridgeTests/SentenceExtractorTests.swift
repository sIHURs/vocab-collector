import Foundation
import Testing
@testable import VocabMacBridge

@Test func extractsSentenceForUnicodeSelection() throws {
    let text = "A calm beginning. Café ☕️ brings clarity. Final thought."
    let range = try #require(text.range(of: "Café ☕️"))
    #expect(SentenceExtractor.sentence(in: text, selection: range) == "Café ☕️ brings clarity.")
}

@Test func returnsTrimmedSelectionWhenNoSentenceCanBeFound() throws {
    let text = "  serendipity  "
    let range = try #require(text.range(of: "serendipity"))
    #expect(SentenceExtractor.sentence(in: text, selection: range) == "serendipity")
}
