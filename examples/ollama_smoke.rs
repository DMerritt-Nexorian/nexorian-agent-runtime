use nexorian_agent_runtime::{LocalHttpEngine, LocalLlmEngine};

fn main() {
    let endpoint = std::env::var("OLLAMA_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:11434/api/generate".to_string());
    let model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "llama3.2".to_string());
    let mut engine = LocalHttpEngine::new(endpoint, model).with_timeout_ms(120_000);
    let output = engine.generate("Reply with the exact text: NEXORIAN_OLLAMA_INFERENCE_OK")
        .unwrap_or_else(|error| {
            eprintln!("Inference smoke test failed: {error}");
            std::process::exit(1);
        });
    if output.trim().is_empty() {
        eprintln!("Ollama returned an empty response");
        std::process::exit(1);
    }
    println!("{output}");
}
