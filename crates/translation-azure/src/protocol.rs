use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct TranslateRequest<'a> {
    #[serde(rename = "Text")]
    pub text: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslateResponse {
    pub detected_language: Option<DetectedLanguage>,
    pub translations: Vec<Translation>,
}

#[derive(Deserialize)]
pub(crate) struct DetectedLanguage {
    pub language: String,
}

#[derive(Deserialize)]
pub(crate) struct Translation {
    pub text: String,
    pub to: String,
}
