use m6502::app::App;
use m6502::binutils::*;

use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = env_logger::builder()
        .format_timestamp(None)
        .format_target(true)
        // By default show all debug messages and above
        .filter_level(log::LevelFilter::Debug)
        // Let user override with envronment variables
        .parse_default_env()
        .try_init();

    let cli = Cli::parse();

    let computer = build_computer(cli.clone());
    // TODO Start the computer in a separate thread, with the correct
    // communication stuff done

    let app = App::new(&computer);

    let items: Vec<String> = app.get_execution_history().iter()
        .map(|x| format!("{:04x}: {}", x.0, x.1)).collect();
    println!("Execution history:");
    for item in items {
        println!("{}", item);
    }

    // TODO shut down the computer. We should also do this when
    // there is an error.

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}