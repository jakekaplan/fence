//! JSON output format for check results.

use std::io::{self, Write};

use loq_core::report::{FindingKind, Report};
use loq_core::MatchBy;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct JsonOutput {
    version: &'static str,
    violations: Vec<JsonViolation>,
    summary: JsonSummary,
}

#[derive(Debug, Serialize)]
struct JsonViolation {
    path: String,
    lines: usize,
    max_lines: usize,
    rule: String,
}

#[derive(Debug, Serialize)]
struct JsonSummary {
    files_checked: usize,
    violations: usize,
}

pub fn write_json<W: Write>(writer: &mut W, report: &Report) -> io::Result<()> {
    let violations = report
        .findings
        .iter()
        .filter_map(|finding| {
            if let FindingKind::Violation {
                actual,
                limit,
                matched_by,
                ..
            } = &finding.kind
            {
                let rule = match matched_by {
                    MatchBy::Rule { pattern } => pattern.clone(),
                    MatchBy::Default => "default".to_string(),
                };
                Some(JsonViolation {
                    path: finding.path.clone(),
                    lines: *actual,
                    max_lines: *limit,
                    rule,
                })
            } else {
                None
            }
        })
        .collect();

    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION"),
        violations,
        summary: JsonSummary {
            files_checked: report.summary.total,
            violations: report.summary.errors,
        },
    };

    serde_json::to_writer_pretty(&mut *writer, &output)?;
    writeln!(writer)
}
