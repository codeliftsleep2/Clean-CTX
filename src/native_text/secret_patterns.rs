use regex::Regex;
use std::sync::OnceLock;

macro_rules! pattern {
    ($name:ident, $value:expr) => {
        pub(super) fn $name() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| Regex::new($value).expect("native secret regex must compile"))
        }
    };
}

pattern!(
    pem,
    r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----"
);
pattern!(
    tokens,
    concat!(
        r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+",
        r"|\b(?:AKIA|ASIA)[0-9A-Z]{16}\b",
        r"|\bAIza[0-9A-Za-z_\-]{35}\b",
        r"|\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b",
        r"|\bgithub_pat_[A-Za-z0-9_]{22,}\b",
        r"|\bxox[baprs]-[A-Za-z0-9-]{10,}\b",
        r"|\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{16,}\b",
        r"|\bsk-(?:ant-)?[A-Za-z0-9_\-]{20,}\b",
        r"|\bhv[sbr]\.[A-Za-z0-9_-]{20,}\b",
        r"|\bpypi-[A-Za-z0-9_-]{16,}\b"
    )
);
pattern!(
    authorization,
    r"(?i)(authorization\s*[:=]\s*[A-Za-z][A-Za-z0-9._-]*\s+)\S+"
);
pattern!(
    secret_flag,
    r#"(?i)(--(?:password|passwd|pwd|secret|token|auth[_-]?token|access[_-]?token|api[_-]?key|access[_-]?key|secret[_-]?key|private[_-]?key|client[_-]?secret|credential|credentials)\s+)('[^']*'|"(?:[^"\\]|\\.)*"|[^\s]+)"#
);
pattern!(
    url_userinfo,
    r"([a-zA-Z][a-zA-Z0-9+.\-]*://[^\s:/@]+:)[^\s@/]+(@)"
);
pattern!(
    secret_assignment,
    r#"(?i)((?:password|passwd|pwd|secret|token|api[_-]?key|access[_-]?key|secret[_-]?key|private[_-]?key|auth[_-]?token|client[_-]?secret)\s*[:=]\s*)('[^']*'|"(?:[^"\\]|\\.)*"|[^\s]+)"#
);
pattern!(database_url, r"(?i)((?:postgres|mysql|mongodb)://)[^@\s]+@");

pub(super) fn initialize() {
    let _ = (
        pem(),
        tokens(),
        authorization(),
        secret_flag(),
        url_userinfo(),
        secret_assignment(),
        database_url(),
    );
}
