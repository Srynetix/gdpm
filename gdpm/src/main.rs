use colored::Colorize;
use log::error;

use gdpm::run_shell;

fn main() {
    if let Err(error) = run_shell() {
        error!(
            "{}",
            format!(
                "Error during execution: {} {}",
                error.as_fail(),
                error.backtrace()
            )
            .color("red")
        );
    }
}
