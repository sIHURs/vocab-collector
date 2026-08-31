export type DesktopWindow = "main" | "capture";
export type PresentationFamily = "shared" | "windows";
export type Presentation = `${PresentationFamily}-${DesktopWindow}`;

export function selectPresentation(
  window: DesktopWindow,
  family: PresentationFamily,
): Presentation {
  return `${family}-${window}`;
}
