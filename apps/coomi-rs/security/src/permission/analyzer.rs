use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ShellAnalysis {
    pub referenced_paths: Vec<PathBuf>,
    pub has_pipe: bool,
    pub has_chaining: bool,
    pub uses_interpreter: bool,
    pub interpreter_code: Vec<String>,
    pub primary_command: String,
    pub risk_score: u8,
}

pub struct ShellAnalyzer {
    pq: Regex,
    bp: Regex,
    interp: Regex,
    pipe: Regex,
    chain: Regex,
}

impl ShellAnalyzer {
    pub fn new() -> Self {
        Self {
            pq: Regex::new(r#""((?:[^"\\]|\\.)*?)"|'([^']*?)'"#).unwrap(),
            bp: Regex::new(r"(?:^|\s)([/~][^\s;&|><]+)").unwrap(),
            interp: Regex::new(r#"(?i)(python[23]?\s+-[cCuU]|node\s+-e|bash\s+-c|sh\s+-c)\s*["']?"#).unwrap(),
            pipe: Regex::new(r"\|").unwrap(),
            chain: Regex::new(r"&&|\|\||;").unwrap(),
        }
    }

    pub fn analyze(&self, command: &str) -> ShellAnalysis {
        let mut paths = Vec::new();
        let mut ui = false;

        for cap in self.pq.captures_iter(command) {
            let c = cap.get(1).or(cap.get(2)).map(|m| m.as_str()).unwrap_or("");
            if ip(c) {
                paths.push(PathBuf::from(c));
            }
        }

        for cap in self.bp.captures_iter(command) {
            if let Some(m) = cap.get(1) {
                let p = m.as_str().trim();
                if ip(p) {
                    paths.push(PathBuf::from(p));
                }
            }
        }

        if self.interp.is_match(command) {
            ui = true;
        }

        let hp = self.pipe.is_match(command);
        let hc = self.chain.is_match(command);

        ShellAnalysis {
            referenced_paths: paths,
            has_pipe: hp,
            has_chaining: hc,
            uses_interpreter: ui,
            interpreter_code: Vec::new(),
            primary_command: command.trim().split_whitespace().next().unwrap_or("").to_string(),
            risk_score: cr(command, hp, hc, ui),
        }
    }
}

fn ip(s: &str) -> bool {
    !s.is_empty() && s.len() < 4096 && (s.starts_with('/') || s.starts_with("~/")) && !s.starts_with("/dev/")
}

fn cr(c: &str, p: bool, ch: bool, i: bool) -> u8 {
    let mut s: u8 = 0;
    if p { s += 1; }
    if ch { s += 1; }
    if i { s += 3; }
    let l = c.to_lowercase();
    for (k, d) in [
        ("rm -rf", 5), ("rm -r", 4), ("mkfs", 8), ("dd if=", 6),
        ("sudo ", 5), ("eval ", 4), ("/etc/passwd", 4), (".ssh/", 4), ("shutdown", 6),
    ] {
        if l.contains(k) {
            s = s.saturating_add(d);
        }
    }
    s.min(10)
}