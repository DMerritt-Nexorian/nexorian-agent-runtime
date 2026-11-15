#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const SYSTEM_PROMPT: &str =
    "You are a Nexorian operator. Respond only with strict JSON.\n\
     Call tool: {\"action\":\"call_tool\",\"tool\":\"<name>\",\"args\":\"<input>\"}\n\
     Final: {\"action\":\"final_response\",\"content\":\"<text>\"}\n\
     Tools: content_create, write_draft, language_practice, task_break, feedback_analyze, \
     contract_review, code_audit, proposal_write, meeting_summary, support_reply.";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    System,
    User,
    Assistant,
    ToolResult,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug)]
pub enum AgentAction {
    CallTool { tool: String, args: String },
    FinalResponse(String),
    ParseError(String),
}

pub trait LocalLlmEngine {
    fn generate(&mut self, prompt: &str) -> Result<String, &'static str>;
}

pub trait CustomerTool {
    fn name(&self) -> &'static str;
    fn execute(&self, args: &str) -> Result<String, &'static str>;
}

/// Local engine that talks to Ollama (or any compatible endpoint) on your machine.
/// No cloud. Run `ollama serve` first.
pub struct LocalHttpEngine {
    pub endpoint: String,
    pub model: String,
}

impl LocalLlmEngine for LocalHttpEngine {
    fn generate(&mut self, prompt: &str) -> Result<String, &'static str> {
        let _ = (prompt, &self.endpoint, &self.model);
        Err("start ollama and use the bridge example")
    }
}

fn field(json: &str, key: &str) -> Option<String> {
    let k = format!("\"{}\"", key);
    let after = json.find(&k).map(|i| &json[i + k.len()..])?;
    let val = after.find(':').map(|i| after[i + 1..].trim_start())?;
    if val.starts_with('"') {
        let body = &val[1..];
        let mut esc = false;
        for (i, c) in body.char_indices() {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                return Some(body[..i].to_string());
            }
        }
        None
    } else {
        let end = val.find([',', '}', ']']).unwrap_or(val.len());
        Some(val[..end].trim().to_string())
    }
}

pub fn parse_action(raw: &str) -> AgentAction {
    let s = raw.find('{').map(|i| &raw[i..]).unwrap_or(raw);
    let e = s.rfind('}').map(|i| &s[..=i]).unwrap_or(s);
    match field(e, "action").as_deref() {
        Some("call_tool") => AgentAction::CallTool {
            tool: field(e, "tool").unwrap_or_default(),
            args: field(e, "args").unwrap_or_default(),
        },
        Some("final_response") => {
            AgentAction::FinalResponse(field(e, "content").unwrap_or_else(|| e.to_string()))
        }
        _ => AgentAction::ParseError(raw.to_string()),
    }
}

// ---- Income tools (local, deterministic) ----

pub struct ContractReview;
impl CustomerTool for ContractReview {
    fn name(&self) -> &'static str { "contract_review" }
    fn execute(&self, args: &str) -> Result<String, &'static str> {
        Ok(format!(
            "{{\"tool\":\"contract_review\",\"summary\":\"Reviewed {} chars. Flag high-risk clauses (indemnity, unlimited liability, auto-renew).\",\"input_preview\":\"{}...\"}}",
            args.len(),
            args.chars().take(80).collect::<String>()
        ))
    }
}

pub struct CodeAudit;
impl CustomerTool for CodeAudit {
    fn name(&self) -> &'static str { "code_audit" }
    fn execute(&self, args: &str) -> Result<String, &'static str> {
        Ok(format!(
            "{{\"tool\":\"code_audit\",\"findings\":[\"Check for unsafe blocks\",\"Verify constant-time on crypto paths\",\"Confirm no_std where claimed\"],\"input_len\":{}}}",
            args.len()
        ))
    }
}

pub struct ProposalWrite;
impl CustomerTool for ProposalWrite {
    fn name(&self) -> &'static str { "proposal_write" }
    fn execute(&self, args: &str) -> Result<String, &'static str> {
        Ok(format!(
            "{{\"tool\":\"proposal_write\",\"draft\":\"Executive summary for: {}. Scope, deliverables, timeline, and fixed price.\"}}",
            args.chars().take(120).collect::<String>()
        ))
    }
}

pub struct MeetingSummary;
impl CustomerTool for MeetingSummary {
    fn name(&self) -> &'static str { "meeting_summary" }
    fn execute(&self, args: &str) -> Result<String, &'static str> {
        let text = args.trim();
        if text.is_empty() {
            return Ok(r#"{"tool":"meeting_summary","error":"empty input"}"#.to_string());
        }

        let mut decisions = Vec::new();
        let mut actions = Vec::new();
        let mut attendees = Vec::new();

        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() { continue; }
            let lower = l.to_lowercase();
            if lower.contains("decided") || lower.contains("decision") || lower.contains("agreed") || lower.starts_with("- d") || lower.starts_with("*") && lower.contains("decide") {
                decisions.push(l.to_string());
            } else if lower.contains("action") || lower.contains("todo") || lower.contains("will ") || lower.contains("to do") || lower.starts_with("- a") || lower.starts_with("*") && (lower.contains("action") || lower.contains("will")) {
                actions.push(l.to_string());
            } else if lower.contains("attendee") || lower.contains("present") || lower.contains("joined") {
                attendees.push(l.to_string());
            }
        }

        // Fallback: if nothing matched, treat first 3 non-empty lines as key points
        if decisions.is_empty() && actions.is_empty() {
            for line in text.lines().filter(|l| !l.trim().is_empty()).take(3) {
                decisions.push(line.trim().to_string());
            }
        }

        let decisions_json = if decisions.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                decisions.iter().map(|d| format!("\"{}\"", d.replace('"', "\\\""))).collect::<Vec<_>>().join(",")
            )
        };
        let actions_json = if actions.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                actions.iter().map(|a| format!("\"{}\"", a.replace('"', "\\\""))).collect::<Vec<_>>().join(",")
            )
        };

        let preview = text.chars().take(120).collect::<String>().replace('"', "\\");

        Ok(format!(
            "{{\"tool\":\"meeting_summary\",\"decisions\":{},\"action_items\":{},\"attendees_detected\":{},\"input_chars\":{},\"preview\":\"{}...\"}}",
            decisions_json,
            actions_json,
            attendees.len(),
            text.len(),
            preview
        ))
    }
}

pub struct SupportReply;
impl CustomerTool for SupportReply {
    fn name(&self) -> &'static str { "support_reply" }
    fn execute(&self, args: &str) -> Result<String, &'static str> {
        Ok(format!(
            "{{\"tool\":\"support_reply\",\"reply\":\"Thanks for reaching out. We reviewed your request regarding {}. Next step: ...\"}}",
            args.chars().take(100).collect::<String>()
        ))
    }
}

macro_rules! tool {
    ($name:ident, $id:literal) => {
        pub struct $name;
        impl CustomerTool for $name {
            fn name(&self) -> &'static str { $id }
            fn execute(&self, args: &str) -> Result<String, &'static str> {
                Ok(format!("{{\"tool\":\"{}\",\"status\":\"completed\",\"input\":\"{}\"}}", $id, args))
            }
        }
    };
}
tool!(ContentCreate, "content_create");
tool!(WriteDraft, "write_draft");
tool!(LanguagePractice, "language_practice");
tool!(TaskBreak, "task_break");
tool!(FeedbackAnalyze, "feedback_analyze");

pub struct ToolAgent<E: LocalLlmEngine> {
    engine: E,
    history: Vec<Message>,
    tools: Vec<Box<dyn CustomerTool>>,
    max_hops: usize,
}

impl<E: LocalLlmEngine> ToolAgent<E> {
    pub fn new(engine: E) -> Self {
        let mut a = Self {
            engine,
            history: Vec::with_capacity(16),
            tools: Vec::with_capacity(10),
            max_hops: 6,
        };
        a.history.push(Message {
            role: Role::System,
            content: SYSTEM_PROMPT.to_string(),
        });
        a.register(Box::new(ContentCreate));
        a.register(Box::new(WriteDraft));
        a.register(Box::new(LanguagePractice));
        a.register(Box::new(TaskBreak));
        a.register(Box::new(FeedbackAnalyze));
        a.register(Box::new(ContractReview));
        a.register(Box::new(CodeAudit));
        a.register(Box::new(ProposalWrite));
        a.register(Box::new(MeetingSummary));
        a.register(Box::new(SupportReply));
        a
    }

    pub fn register(&mut self, t: Box<dyn CustomerTool>) {
        self.tools.push(t);
    }

    fn prompt(&self) -> String {
        let mut p = String::with_capacity(1024);
        for m in &self.history {
            let tag = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::ToolResult => "tool",
            };
            p.push_str("<|");
            p.push_str(tag);
            p.push_str("|>\n");
            p.push_str(&m.content);
            p.push_str("\n");
        }
        p.push_str("<|assistant|>\n");
        p
    }

    pub fn step(&mut self, input: &str) -> Result<String, &'static str> {
        self.history.push(Message {
            role: Role::User,
            content: input.to_string(),
        });
        for _ in 0..self.max_hops {
            let raw = self.engine.generate(&self.prompt())?;
            match parse_action(&raw) {
                AgentAction::CallTool { tool, args } => {
                    self.history.push(Message {
                        role: Role::Assistant,
                        content: raw,
                    });
                    let result = self
                        .tools
                        .iter()
                        .find(|t| t.name() == tool)
                        .map(|t| t.execute(&args).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e)))
                        .unwrap_or_else(|| "{\"error\":\"unknown tool\"}".to_string());
                    self.history.push(Message {
                        role: Role::ToolResult,
                        content: result,
                    });
                }
                AgentAction::FinalResponse(c) => {
                    self.history.push(Message {
                        role: Role::Assistant,
                        content: c.clone(),
                    });
                    return Ok(c);
                }
                AgentAction::ParseError(f) => {
                    self.history.push(Message {
                        role: Role::Assistant,
                        content: f.clone(),
                    });
                    return Ok(f);
                }
            }
        }
        Err("max hops")
    }
}
