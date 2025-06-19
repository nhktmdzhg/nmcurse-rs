extern crate ncurses;
mod network;
mod ui;

fn main() {
    let mut ui = network::NetworkUi::new();
    ui.scan();
    ui.display_networks();

    loop {
        let network_index = ui.select_network();
        if let Some(network_index) = network_index {
            ui.connect(network_index);
        } else {
            break;
        }
        ui.run_scan();
        ui.display_networks();
    }
}
