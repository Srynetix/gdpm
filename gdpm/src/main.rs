use gdpm::run_shell;

fn main() {
    if let Err(error) = run_shell() {
        eprintln!(
            "Error during execution: {} {}",
            error.as_fail(),
            error.backtrace()
        );
    }
}
