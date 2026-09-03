use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use vocab_platform_api::{PlatformError, TranslationProvider, TranslationResult};
use vocab_platform_contract_tests::assert_error_is_content_free;
use vocab_translation_deepl::{DeepLTranslationProvider, DeepLTranslatorConfig};

fn serve(
    status: &'static str,
    body: &'static str,
    retry_after: Option<u64>,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buffer).unwrap_or_default();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if String::from_utf8_lossy(&request).contains("\r\n\r\n") {
                break;
            }
        }
        let retry = retry_after.map_or(String::new(), |seconds| {
            format!("Retry-After: {seconds}\r\n")
        });
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        String::from_utf8(request).unwrap()
    });
    (format!("http://{address}"), handle)
}

#[tokio::test(flavor = "current_thread")]
async fn auto_source_unicode_translation_uses_the_portable_contract() {
    let body = r#"{"translations":[{"detected_source_language":"EN","text":"Straße"}]}"#;
    let (endpoint, request) = serve("200 OK", body, None);
    let config =
        DeepLTranslatorConfig::for_test(endpoint, "private-key", Duration::from_secs(2)).unwrap();
    let provider = DeepLTranslationProvider::new(config).unwrap();

    let result = provider.translate("street 👩🏽‍💻", "auto", "de").await.unwrap();

    assert_eq!(
        result,
        TranslationResult {
            translated_text: "Straße".into(),
            source_language: "en".into(),
            target_language: "de".into()
        }
    );
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /v2/translate HTTP/1.1"));
    assert!(
        request.contains("Authorization: DeepL-Auth-Key private-key")
            || request
                .to_ascii_lowercase()
                .contains("authorization: deepl-auth-key private-key")
    );
    assert!(request.contains("street 👩🏽‍💻"));
    assert!(!request.contains("source_lang"));
    assert!(!request.contains("context"));
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_source_is_adapted_without_changing_the_portable_result() {
    let body = r#"{"translations":[{"detected_source_language":"ZH","text":"hello"}]}"#;
    let (endpoint, request) = serve("200 OK", body, None);
    let provider = DeepLTranslationProvider::new(
        DeepLTranslatorConfig::for_test(endpoint, "key", Duration::from_secs(2)).unwrap(),
    )
    .unwrap();

    let result = provider.translate("你好", "zh-Hans", "en").await.unwrap();

    assert_eq!(result.target_language, "en");
    assert!(request.join().unwrap().contains("ZH-HANS"));
}

#[tokio::test(flavor = "current_thread")]
async fn authentication_failure_is_content_safe() {
    let (endpoint, request) = serve(
        "403 Forbidden",
        r#"{"message":"private upstream body"}"#,
        None,
    );
    let provider = DeepLTranslationProvider::new(
        DeepLTranslatorConfig::for_test(endpoint, "private-key", Duration::from_secs(2)).unwrap(),
    )
    .unwrap();

    let error = provider
        .translate("private selected text", "auto", "de")
        .await
        .unwrap_err();

    assert_eq!(
        error,
        PlatformError::Operation("deepl translation authentication failed".into())
    );
    assert_error_is_content_free(
        &error,
        &[
            "private-key",
            "private selected text",
            "private upstream body",
        ],
    );
    request.join().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn retry_after_cannot_exceed_the_overall_budget() {
    let (endpoint, request) = serve("429 Too Many Requests", "{}", Some(2));
    let provider = DeepLTranslationProvider::new(
        DeepLTranslatorConfig::for_test(endpoint, "key", Duration::from_millis(100)).unwrap(),
    )
    .unwrap();

    let error = provider.translate("hello", "auto", "de").await.unwrap_err();

    assert_eq!(
        error,
        PlatformError::Operation("deepl translation timed out".into())
    );
    request.join().unwrap();
}

#[test]
fn configuration_accepts_free_and_pro_https_endpoints_and_redacts_values() {
    for endpoint in ["https://api-free.deepl.com", "https://api.deepl.com"] {
        let config =
            DeepLTranslatorConfig::new(endpoint, "private-key", Duration::from_secs(2)).unwrap();
        let debug = format!("{config:?}");
        assert!(!debug.contains("private-key"));
        assert!(!debug.contains("deepl.com"));
    }
    assert!(
        DeepLTranslatorConfig::new("http://api-free.deepl.com", "key", Duration::from_secs(2))
            .is_err()
    );
}
