extern crate ncurses;
mod network;
mod ui;

fn main() {
    let mut ui = network::NetworkUi::new();
    ui.scan();
    ui.display_networks();

    loop {
        let network_index = ui.select_network();
        if network_index.is_none() {
            break;
        }
        ui.connect(network_index.unwrap());
        ui.run_scan();
        ui.display_networks();
    }
}
