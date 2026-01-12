//! Human-readable output formatting.
//!
//! Formats findings and summaries for terminal output.

use crate::report::{Finding, FindingKind, SkipReason, Summary};

/// Formats a finding for display.
#[must_use]
pub fn format_finding(finding: &Finding) -> String {
    match &finding.kind {
        FindingKind::Violation {
            limit,
            actual,
            over_by,
            ..
        } => format_violation(&finding.path, *actual, *limit, *over_by),
        FindingKind::SkipWarning { reason } => format_skip_warning(&finding.path, reason),
    }
}

/// Formats a violation message.
#[must_use]
pub fn format_violation(path: &str, actual: usize, limit: usize, over_by: usize) -> String {
    format!("error[max-lines]: {path}: {actual} lines (limit: {limit}, +{over_by} over)")
}

/// Formats a skip warning message.
#[must_use]
pub fn format_skip_warning(path: &str, reason: &SkipReason) -> String {
    match reason {
        SkipReason::Binary => format!("warning[skip-binary]: {path}: binary file skipped"),
        SkipReason::Unreadable(error) => {
            format!("warning[skip-unreadable]: {path}: unreadable file skipped ({error})")
        }
        SkipReason::Missing => {
            format!("warning[skip-missing]: {path}: missing file skipped")
        }
    }
}

/// Formats the summary line with counts.
#[must_use]
pub fn format_summary(summary: &Summary) -> String {
    let error_label = if summary.errors == 1 {
        "error"
    } else {
        "errors"
    };
    format!(
        "{} files checked, {} skipped, {} passed, {} {} ({}ms)",
        summary.total,
        summary.skipped,
        summary.passed,
        summary.errors,
        error_label,
        summary.duration_ms
    )
}

/// Formats a success message when all checks pass.
#[must_use]
pub fn format_success(summary: &Summary) -> String {
    format!(
        "All checks passed! ({} files in {}ms)",
        summary.total, summary.duration_ms
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigOrigin;
    use crate::decide::MatchBy;
    use crate::report::Finding;

    #[test]
    fn format_error_line() {
        let finding = Finding {
            path: "src/lib.rs".into(),
            config_source: ConfigOrigin::BuiltIn,
            kind: FindingKind::Violation {
                limit: 10,
                actual: 12,
                over_by: 2,
                matched_by: MatchBy::Default,
            },
        };
        let line = format_finding(&finding);
        assert_eq!(
            line,
            "error[max-lines]: src/lib.rs: 12 lines (limit: 10, +2 over)"
        );
    }

    #[test]
    fn format_skip_binary() {
        let line = format_skip_warning("bin", &SkipReason::Binary);
        assert_eq!(line, "warning[skip-binary]: bin: binary file skipped");
    }

    #[test]
    fn format_skip_unreadable_and_missing() {
        let unreadable = format_skip_warning("bin", &SkipReason::Unreadable("denied".into()));
        assert_eq!(
            unreadable,
            "warning[skip-unreadable]: bin: unreadable file skipped (denied)"
        );
        let missing = format_skip_warning("bin", &SkipReason::Missing);
        assert_eq!(missing, "warning[skip-missing]: bin: missing file skipped");
    }

    #[test]
    fn format_summary_pluralization() {
        let summary = Summary {
            total: 2,
            skipped: 0,
            passed: 0,
            errors: 1,
            duration_ms: 5,
        };
        let line = format_summary(&summary);
        assert!(line.contains("1 error"));
    }

    #[test]
    fn format_summary_plural_errors() {
        let summary = Summary {
            total: 3,
            skipped: 0,
            passed: 1,
            errors: 2,
            duration_ms: 5,
        };
        let line = format_summary(&summary);
        assert!(line.contains("2 errors"));
    }

    #[test]
    fn format_finding_skip_warning() {
        let finding = Finding {
            path: "missing.txt".into(),
            config_source: ConfigOrigin::BuiltIn,
            kind: FindingKind::SkipWarning {
                reason: SkipReason::Missing,
            },
        };
        let line = format_finding(&finding);
        assert_eq!(
            line,
            "warning[skip-missing]: missing.txt: missing file skipped"
        );
    }

    #[test]
    fn format_success_message() {
        let summary = Summary {
            total: 10,
            skipped: 2,
            passed: 8,
            errors: 0,
            duration_ms: 42,
        };
        let line = format_success(&summary);
        assert_eq!(line, "All checks passed! (10 files in 42ms)");
    }
}
