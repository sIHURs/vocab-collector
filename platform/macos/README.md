# macOS bridge

`native/` contains the Swift static-library boundary for Accessibility selection capture, Vision OCR, Natural Language, Apple Translation, notifications, and window positioning. The sibling `rust/` crate adapts that ABI to the portable platform contracts so other operating systems can implement the same capabilities later.
