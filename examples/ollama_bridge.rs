use nexorian_agent_runtime::{LocalHttpEngine, ToolAgent};

fn main() {
    let engine = LocalHttpEngine::new(
        "http://127.0.0.1:11434/api/generate",
        "llama3.2",
    );
    let mut agent = ToolAgent::new(engine);
    match agent.step("Draft a short proposal for a security audit") {
        Ok(reply) => println!("{reply}"),
        Err(error) => {
            eprintln!("Ollama bridge failed: {error}");
            std::process::exit(1);
        }
    }
}
