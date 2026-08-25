import AppKit
import Foundation
import SwiftUI
@preconcurrency import Translation

public struct TranslationPayload: Codable, Equatable, Sendable {
    public let translatedText: String
    public let sourceLanguage: String
    public let targetLanguage: String
}

public enum TranslationOutcome: Sendable {
    case success(TranslationPayload)
    case failure(String)
}

private struct TranslationTaskView: View {
    let text: String
    let source: Locale.Language
    let target: Locale.Language
    let completion: @Sendable (TranslationOutcome) -> Void
    @State private var configuration: TranslationSession.Configuration?

    var body: some View {
        Color.clear.frame(width: 1, height: 1)
            .onAppear { configuration = .init(source: source, target: target) }
            .translationTask(configuration) { session in
                do {
                    let response = try await session.translate(text)
                    completion(.success(.init(
                        translatedText: response.targetText,
                        sourceLanguage: response.sourceLanguage.languageCode?.identifier ?? "und",
                        targetLanguage: response.targetLanguage.languageCode?.identifier ?? target.languageCode?.identifier ?? "und"
                    )))
                } catch { completion(.failure(String(describing: error))) }
            }
    }
}

@MainActor
public final class TranslationHost {
    private static var active: [UUID: NSWindow] = [:]

    public static func translate(text: String, sourceIdentifier: String, targetIdentifier: String,
                                 completion: @escaping @Sendable (TranslationOutcome) -> Void) {
        let id = UUID()
        let source = Locale.Language(identifier: sourceIdentifier)
        let target = Locale.Language(identifier: targetIdentifier)
        let view = TranslationTaskView(text: text, source: source, target: target) { result in
            Task { @MainActor in
                active[id]?.close()
                active[id] = nil
                completion(result)
            }
        }
        let window = NSWindow(contentViewController: NSHostingController(rootView: view))
        window.setFrame(NSRect(x: -10_000, y: -10_000, width: 1, height: 1), display: false)
        window.isReleasedWhenClosed = false
        active[id] = window
        window.orderBack(nil)
    }
}
