#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        #[allow(clippy::print_stderr)] // Panic hooks must write to stderr
        {
            eprintln!("loq panicked. This is a bug.");
            eprintln!("{info}");
            eprintln!("Please report at: https://github.com/jakekaplan/loq/issues");
        }
    }));

    loq_cli::run_env().into()
}
