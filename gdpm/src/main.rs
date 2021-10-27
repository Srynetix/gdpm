use color_eyre::Result;

use gdpm::run_shell;

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    run_shell()
}
