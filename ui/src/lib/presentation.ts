export type DesktopWindow = "main" | "capture";
export type DocumentWindow = DesktopWindow | "ocr-overlay";
export type PresentationFamily = "shared" | "windows";
export type Presentation = `${PresentationFamily}-${DesktopWindow}`;

export function selectPresentation(
  window: DesktopWindow,
  family: PresentationFamily,
): Presentation {
  return `${family}-${window}`;
}

export function markDocumentWindow(root: HTMLElement, window: DocumentWindow): void {
  root.dataset.window = window;
}
