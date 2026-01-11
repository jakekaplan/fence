#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("loq panicked. This is a bug.");
        eprintln!("{info}");
        eprintln!("Please report at: https://github.com/your-org/loq/issues");
    }));

    loq_cli::run_env().into()
}
