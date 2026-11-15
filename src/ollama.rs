use alloc::string::String;

pub struct LocalHttpEngine {
    pub endpoint: String,
    pub model: String,
    pub timeout_ms: u64,
}

impl LocalHttpEngine {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            timeout_ms: 30_000,
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[cfg(feature = "std")]
impl crate::LocalLlmEngine for LocalHttpEngine {
    fn generate(&mut self, prompt: &str) -> Result<String, &'static str> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|_| "failed to build HTTP client")?;
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });
        let response = client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .map_err(|err| {
                if err.is_timeout() {
                    "Ollama request timed out"
                } else {
                    "failed to connect to Ollama"
                }
            })?;
        if !response.status().is_success() {
            return Err("Ollama returned a non-success HTTP status");
        }
        let body = response
            .text()
            .map_err(|_| "failed to read Ollama response")?;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).map_err(|_| "Ollama returned invalid JSON")?;
        parsed
            .get("response")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or("Ollama response is missing the string 'response' field")
    }
}

#[cfg(not(feature = "std"))]
impl crate::LocalLlmEngine for LocalHttpEngine {
    fn generate(&mut self, _prompt: &str) -> Result<String, &'static str> {
        Err("enable the std feature to use the HTTP Ollama engine")
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    fn mock_server(status: &str, body: &'static str, delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let address = listener.local_addr().expect("mock address");
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut request = vec![0_u8; 8192];
                let _ = stream.read(&mut request);
                thread::sleep(delay);
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        format!("http://{address}/api/generate")
    }

    #[test]
    fn posts_ollama_payload_and_parses_response() {
        let endpoint = mock_server(
            "200 OK",
            r#"{"response":"hello from mock"}"#,
            Duration::ZERO,
        );
        let mut engine = LocalHttpEngine::new(endpoint, "llama3.2");
        assert_eq!(engine.generate("say hello").unwrap(), "hello from mock");
    }

    #[test]
    fn reports_http_status_errors() {
        let endpoint = mock_server(
            "500 Internal Server Error",
            r#"{"error":"model unavailable"}"#,
            Duration::ZERO,
        );
        let mut engine = LocalHttpEngine::new(endpoint, "llama3.2");
        assert_eq!(
            engine.generate("prompt"),
            Err("Ollama returned a non-success HTTP status")
        );
    }

    #[test]
    fn rejects_malformed_json() {
        let endpoint = mock_server("200 OK", "not-json", Duration::ZERO);
        let mut engine = LocalHttpEngine::new(endpoint, "llama3.2");
        assert_eq!(
            engine.generate("prompt"),
            Err("Ollama returned invalid JSON")
        );
    }

    #[test]
    fn enforces_connection_timeout() {
        let endpoint = mock_server(
            "200 OK",
            r#"{"response":"late"}"#,
            Duration::from_millis(250),
        );
        let mut engine = LocalHttpEngine::new(endpoint, "llama3.2").with_timeout_ms(50);
        assert_eq!(
            engine.generate("prompt"),
            Err("Ollama request timed out")
        );
    }

    #[test]
    fn reports_connection_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let address = listener.local_addr().expect("local address");
        drop(listener);
        let mut engine =
            LocalHttpEngine::new(format!("http://{address}/api/generate"), "llama3.2");
        assert_eq!(
            engine.generate("prompt"),
            Err("failed to connect to Ollama")
        );
    }
}
