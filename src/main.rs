extern crate ncurses;
mod network;
mod ui;

fn main() {
    let mut ui = network::NetworkUi::new();
    ui.scan();
    ui.display_networks();

    loop {
        let network_name = ui.select_network();
        if network_name.is_empty() {
            break; // Exit if no network is selected
        }
        ui.connect(&network_name);
        ui.display_networks();
    }
}
