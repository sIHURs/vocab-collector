import Foundation
import NaturalLanguage

public enum SentenceExtractor {
    public static func sentence(in text: String, selection: Range<String.Index>) -> String {
        guard !text.isEmpty else { return "" }
        let tokenizer = NLTokenizer(unit: .sentence)
        tokenizer.string = text
        let sentenceRange = tokenizer.tokenRange(at: selection.lowerBound)
        if !sentenceRange.isEmpty {
            return text[sentenceRange].trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return text[selection].trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
