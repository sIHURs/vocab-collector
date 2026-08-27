import CoreGraphics
import Testing
@testable import VocabMacBridge

@Test func primaryDisplayOcrBoundsUsePrimaryTopLeftCoordinates() {
    let bounds = OcrCapture.normalizedBounds(
        primaryFrame: CGRect(x: 0, y: 0, width: 1440, height: 900),
        displayFrame: CGRect(x: 0, y: 0, width: 1440, height: 900),
        captureRegion: CGRect(x: 100, y: 50, width: 800, height: 400),
        recognizedBounds: CGRect(x: 0.25, y: 0.5, width: 0.1, height: 0.2)
    )

    #expect(bounds == BridgeRect(x: 300, y: 170, width: 80, height: 80))
}

@Test func displayAbovePrimaryProducesNegativeSharedY() {
    let bounds = OcrCapture.normalizedBounds(
        primaryFrame: CGRect(x: 0, y: 0, width: 1440, height: 900),
        displayFrame: CGRect(x: -1280, y: 900, width: 1280, height: 800),
        captureRegion: CGRect(x: 40, y: 60, width: 600, height: 300),
        recognizedBounds: CGRect(x: 0.5, y: 0.4, width: 0.2, height: 0.3)
    )

    #expect(bounds == BridgeRect(x: -940, y: -650, width: 120, height: 90))
}

@Test func displayBelowPrimaryProducesSharedYBeyondPrimaryHeight() {
    let bounds = OcrCapture.normalizedBounds(
        primaryFrame: CGRect(x: 0, y: 0, width: 1440, height: 900),
        displayFrame: CGRect(x: 1440, y: -1080, width: 1920, height: 1080),
        captureRegion: CGRect(x: 200, y: 100, width: 900, height: 400),
        recognizedBounds: CGRect(x: 0.1, y: 0.2, width: 0.3, height: 0.25)
    )

    #expect(bounds == BridgeRect(x: 1730, y: 1220, width: 270, height: 100))
}

@Test func negativeDisplayXIsRelativeToANonzeroPrimaryOrigin() {
    let bounds = OcrCapture.normalizedBounds(
        primaryFrame: CGRect(x: 100, y: -200, width: 1440, height: 900),
        displayFrame: CGRect(x: -1820, y: -200, width: 1920, height: 1080),
        captureRegion: CGRect(x: 64, y: 64, width: 1024, height: 512),
        recognizedBounds: CGRect(x: 0.25, y: 0.5, width: 0.125, height: 0.25)
    )

    #expect(bounds == BridgeRect(x: -1600, y: 12, width: 128, height: 128))
}
