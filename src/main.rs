extern crate ncurses;
mod network;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
