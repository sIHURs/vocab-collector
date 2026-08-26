use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    Selection,
    SelectionBounds,
    ScreenshotOcr,
    Translation,
    NonActivatingWindow,
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Selection => "selection",
            Self::SelectionBounds => "selection bounds",
            Self::ScreenshotOcr => "screenshot OCR",
            Self::Translation => "translation",
            Self::NonActivatingWindow => "non-activating window",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub selection_capture: bool,
    pub selection_bounds: bool,
    pub screenshot_ocr: bool,
    pub translation: bool,
    pub non_activating_window: bool,
}
