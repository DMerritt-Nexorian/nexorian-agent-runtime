// examples/ollama_bridge.rs
// std is allowed in the example binary

use nexorian_agent_runtime::*;

fn main() {
    let engine = LocalHttpEngine {
        endpoint: "http://127.0.0.1:11434/api/generate".into(),
        model: "llama3.2".into(),
    };
    let mut agent = ToolAgent::new(engine);
    match agent.step("Draft a short proposal for a security audit") {
        Ok(reply) => println!("{}", reply),
        Err(e) => eprintln!("error: {}", e),
    }
}
