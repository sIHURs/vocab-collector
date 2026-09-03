use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use vocab_platform_api::{PlatformError, TranslationProvider, TranslationResult};
use vocab_platform_contract_tests::assert_error_is_content_free;
use vocab_translation_azure::{AzureTranslationProvider, AzureTranslatorConfig};

fn serve_once(response_body: &'static str) -> (String, thread::JoinHandle<String>) {
    serve_once_with_status("200 OK", response_body)
}

fn serve_once_with_status(
    status: &'static str,
    response_body: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buffer).unwrap_or_default();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            let text = String::from_utf8_lossy(&request);
            let Some(headers_end) = text.find("\r\n\r\n") else {
                continue;
            };
            let content_length = text[..headers_end]
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or_default();
            if request.len() >= headers_end + 4 + content_length {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
        String::from_utf8(request).unwrap()
    });
    (format!("http://{address}"), handle)
}

fn serve_sequence(
    responses: Vec<(&'static str, &'static str)>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).unwrap_or_default();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                let text = String::from_utf8_lossy(&request);
                let Some(headers_end) = text.find("\r\n\r\n") else {
                    continue;
                };
                let content_length = text[..headers_end]
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or_default();
                if request.len() >= headers_end + 4 + content_length {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            requests.push(String::from_utf8(request).unwrap());
        }
        requests
    });
    (format!("http://{address}"), handle)
}

#[test]
fn configuration_rejects_insecure_endpoints_and_redacts_secrets() {
    let key = "developer-secret-key";
    let error = AzureTranslatorConfig::new(
        "http://translator.example.test",
        key,
        Some("westeurope".into()),
        Duration::from_secs(2),
    )
    .unwrap_err();
    assert_error_is_content_free(&error, &[key]);

    let config = AzureTranslatorConfig::new(
        "https://translator.example.test",
        key,
        Some("westeurope".into()),
        Duration::from_secs(2),
    )
    .unwrap();
    let diagnostic = format!("{config:?}");
    assert!(!diagnostic.contains(key));
    assert!(!diagnostic.contains("translator.example.test"));
}

#[tokio::test(flavor = "current_thread")]
async fn automatic_source_translation_uses_only_selected_unicode_text() {
    let response = r#"[{"detectedLanguage":{"language":"en","score":1.0},"translations":[{"text":"Straße","to":"de"}]}]"#;
    let (endpoint, request) = serve_once(response);
    let config =
        AzureTranslatorConfig::for_test(endpoint, "private-key", Duration::from_secs(2)).unwrap();
    let provider = AzureTranslationProvider::new(config).unwrap();

    let result = provider.translate("street 👩🏽‍💻", "auto", "de").await.unwrap();

    assert_eq!(
        result,
        TranslationResult {
            translated_text: "Straße".into(),
            source_language: "en".into(),
            target_language: "de".into(),
        }
    );
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /translate?api-version=3.0&to=de HTTP/1.1"));
    assert!(
        request
            .to_ascii_lowercase()
            .contains("ocp-apim-subscription-key: private-key")
    );
    assert!(request.contains(r#"[{"Text":"street 👩🏽‍💻"}]"#));
    assert!(!request.contains("context"));
    assert!(!request.contains("&from="));
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_source_and_region_are_sent_without_exposing_protocol_types() {
    let response = r#"[{"translations":[{"text":"Feinheit","to":"de"}]}]"#;
    let (endpoint, request) = serve_once(response);
    let config = AzureTranslatorConfig::new(
        endpoint.replace("http://", "https://"),
        "private-key",
        Some("westeurope".into()),
        Duration::from_secs(2),
    );
    assert!(
        config.is_ok(),
        "public configuration accepts an HTTPS endpoint"
    );

    let config = AzureTranslatorConfig::for_test(endpoint, "private-key", Duration::from_secs(2))
        .unwrap()
        .with_test_region("westeurope");
    let provider = AzureTranslationProvider::new(config).unwrap();
    let result = provider.translate("nuance", "en", "de").await.unwrap();

    assert_eq!(result.source_language, "en");
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /translate?api-version=3.0&to=de&from=en HTTP/1.1"));
    assert!(
        request
            .to_ascii_lowercase()
            .contains("ocp-apim-subscription-region: westeurope")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn authentication_failure_is_content_safe_and_is_not_retried() {
    let private_text = "private selected vocabulary";
    let private_key = "private-key";
    let (endpoint, request) = serve_once_with_status(
        "401 Unauthorized",
        r#"{"error":{"message":"private upstream diagnostic"}}"#,
    );
    let config =
        AzureTranslatorConfig::for_test(endpoint, private_key, Duration::from_secs(2)).unwrap();
    let provider = AzureTranslationProvider::new(config).unwrap();

    let error = provider
        .translate(private_text, "en", "de")
        .await
        .unwrap_err();

    assert_eq!(
        error,
        PlatformError::Operation("azure translation authentication failed".into())
    );
    assert_error_is_content_free(
        &error,
        &[private_text, private_key, "private upstream diagnostic"],
    );
    request.join().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn transient_service_failures_retry_twice_then_return_the_translation() {
    let response = r#"[{"translations":[{"text":"Straße","to":"de"}]}]"#;
    let (endpoint, requests) = serve_sequence(vec![
        ("500 Internal Server Error", "{}"),
        ("503 Service Unavailable", "{}"),
        ("200 OK", response),
    ]);
    let config =
        AzureTranslatorConfig::for_test(endpoint, "private-key", Duration::from_secs(2)).unwrap();
    let provider = AzureTranslationProvider::new(config).unwrap();

    let result = provider.translate("street", "en", "de").await.unwrap();

    assert_eq!(result.translated_text, "Straße");
    assert_eq!(requests.join().unwrap().len(), 3);
}
