extern crate ncurses;
mod network;
mod ui;

use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn print_version() {
    println!("{NAME} v{VERSION}");
}

fn print_help() {
    println!("{NAME} v{VERSION}");
    println!("A network management tool using ncurses");
    println!();
    println!("USAGE:");
    println!("    {NAME} [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -v, --version    Print version information");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    let mut ui = network::NetworkUi::new();

    // Initial scan with error handling
    ui.scan();
    ui.display_networks();

    loop {
        let network_index = ui.select_network();
        if let Some(network_index) = network_index {
            ui.connect(network_index);
        } else {
            break;
        }
        let _ = ui.run_scan();
        ui.display_networks();
    }

    Ok(())
}
