extern crate ncurses;
mod network;
mod ui;

fn main() {
    let mut ui = network::NetworkUi::new();
    ui.scan();
    ui.display_networks();

    loop {
        let network_name = ui.select_network();
        if network_name.is_none() {
            break;
        }
        ui.connect(&network_name.unwrap());
        ui.display_networks();
    }
}
