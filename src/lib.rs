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

macro_rules! tool {
    ($name:ident, $id:literal) => {
        pub struct $name;
        impl CustomerTool for $name {
            fn name(&self) -> &'static str {
                $id
            }
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
tool!(ContractReview, "contract_review");
tool!(CodeAudit, "code_audit");
tool!(ProposalWrite, "proposal_write");
tool!(MeetingSummary, "meeting_summary");
tool!(SupportReply, "support_reply");

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
                        .unwrap_or_else(|| format!("{{\"error\":\"unknown tool\"}}"));
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
