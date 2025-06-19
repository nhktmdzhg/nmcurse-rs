use ncurses::ll::werase;
use ncurses::{COLOR_PAIR, *};
use std::sync::mpsc;

use super::ui::Ui;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::vec::Vec;
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct Network {
    pub in_use: bool,
    pub ssid: String,
    pub bssid: String,
    pub security: String,
    pub signal: u8,
}

pub struct NetworkUi {
    highlight: usize,
    networks: Vec<Network>,
    ui: Ui,
}

impl NetworkUi {
    pub fn new() -> Self {
        NetworkUi {
            highlight: 0,
            networks: Vec::new(),
            ui: Ui::new(),
        }
    }

    pub fn display_networks(&mut self) {
        unsafe { werase(self.ui.win()) };
        if self.networks.is_empty() {
            wrefresh(self.ui.win());
            return;
        }

        let mut max_ssid_length = 3;
        let mut max_security_length = 3;
        for network in &self.networks {
            if network.ssid.len() > max_ssid_length {
                max_ssid_length = network.ssid.len();
            }
            if network.security.len() > max_security_length {
                max_security_length = network.security.len();
            }
        }

        // Calculate the maximum length of SSID and security strings to fit the window
        // -6 for padding and formatting
        let max_combined_length: usize =
            std::cmp::max(0, getmaxx(self.ui.win()) - max_security_length as i32 - 6) as usize;
        // Calculate the window height, leaving space for the header and footer
        let win_height: usize = std::cmp::max(0, getmaxy(self.ui.win()) - 4) as usize;

        let start_index: usize =
            std::cmp::max(0, self.highlight as i32 - win_height as i32 + 1) as usize;
        let end_index: usize = std::cmp::min(self.networks.len(), start_index + win_height);

        // Display the header
        wattron(self.ui.win(), COLOR_PAIR(5));
        let _ = mvwprintw(self.ui.win(), 1, 3, "Available Networks");
        mvwhline(self.ui.win(), 2, 1, 0, getmaxx(self.ui.win()) - 2);
        mvwhline(
            self.ui.win(),
            getmaxy(self.ui.win()) - 1,
            1,
            0,
            getmaxx(self.ui.win()) - 2,
        );

        // Display the footer
        let _ = mvwprintw(
            self.ui.win(),
            getmaxy(self.ui.win()) - 1,
            3,
            "[r: Rescan, d: Disconnect, f: Forget, enter: Connect, q: Quit]",
        );

        wattroff(self.ui.win(), COLOR_PAIR(5));

        let mut color: u32;
        for i in start_index..end_index {
            if self.networks[i].signal >= 66 {
                color = COLOR_PAIR(3);
            } else if self.networks[i].signal >= 33 {
                color = COLOR_PAIR(2);
            } else {
                color = COLOR_PAIR(1);
            }

            let mut ss: String = String::new();
            if self.networks[i].in_use {
                color |= ncurses::A_BOLD;
                ss.push_str("> ");
            } else {
                ss.push_str("  ");
            }

            let mut ssid = if self.networks[i].ssid.is_empty() {
                String::from("---")
            } else {
                self.networks[i].ssid.clone()
            };
            let security = if self.networks[i].security.is_empty() {
                String::from("---")
            } else {
                self.networks[i].security.clone()
            };

            // Truncate SSID and security strings to fit the window
            if ssid.len() > max_combined_length {
                ssid.truncate(max_combined_length - 3);
                ssid.push_str("...");
            }
            ss.push_str(
                format!(
                    "{:<width_ssid$}{:<width_security$}  ",
                    ssid,
                    security,
                    width_ssid = max_combined_length,
                    width_security = max_security_length,
                )
                .as_str(),
            );

            wattron(
                self.ui.win(),
                if i == self.highlight {
                    color | ncurses::A_REVERSE
                } else {
                    color
                },
            );
            //Adjust position relative to the visible range
            let _ = mvwprintw(self.ui.win(), (i - start_index + 3) as i32, 1, &ss);
            wattroff(
                self.ui.win(),
                if i == self.highlight {
                    color | ncurses::A_REVERSE
                } else {
                    color
                },
            );
        }

        wrefresh(self.ui.win());
    }

    pub fn run_scan(&mut self) {
        self.networks.clear();
        let mut output = Command::new("nmcli")
            .args(&[
                "-f",
                "IN-USE,SSID,BSSID,SECURITY,SIGNAL",
                "--mode",
                "multiline",
                "--terse",
                "dev",
                "wifi",
                "list",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let reader = BufReader::new(output.stdout.take().unwrap());

        let mut network = Network {
            in_use: false,
            ssid: String::new(),
            bssid: String::new(),
            security: String::new(),
            signal: 0,
        };

        for line in reader.lines().filter_map(|l| l.ok()) {
            if line.starts_with("IN-USE:") {
                network.in_use = line[7..].trim() == "*";
            } else if line.starts_with("SSID:") {
                network.ssid = line[5..].trim().to_string();
            } else if line.starts_with("BSSID:") {
                network.bssid = line[6..].trim().to_string();
            } else if line.starts_with("SECURITY:") {
                network.security = line[9..].trim().to_string();
            } else if line.starts_with("SIGNAL:") {
                network.signal = line[7..].trim().parse::<u8>().unwrap_or(0);
                self.networks.push(network.clone());

                network = Network {
                    in_use: false,
                    ssid: String::new(),
                    bssid: String::new(),
                    security: String::new(),
                    signal: 0,
                };
            }
        }

        self.networks.sort_by(|a, b| {
            if a.signal > b.signal {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        output.wait().unwrap();
    }

    pub fn scan(&mut self) {
        self.ui.clear();
        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));
        let loading_thread = thread::spawn(move || unsafe {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                } else {
                    ui.loading_animation("Scanning for networks...");
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });

        self.run_scan();
        tx.send(()).unwrap();
        loading_thread.join().unwrap();
    }

    fn get_input(&self) -> i32 {
        set_escdelay(0);
        let mut input = wgetch(self.ui.win());
        if input == ERR {
            return ERR;
        }

        // Fix for alt/escape/arrows (also f-keys on some terminals)
        if input == 27 {
            nodelay(self.ui.win(), true);
            input = wgetch(self.ui.win());
            nodelay(self.ui.win(), false);

            if input == ERR {
                return 27;
            }

            if input != 91 {
                return input;
            }

            // Arrow keys
            nodelay(self.ui.win(), true);
            input = wgetch(self.ui.win());
            nodelay(self.ui.win(), false);

            if input != ERR {
                return input;
            }

            return 27;
        }
        input
    }

    pub fn select_network(&mut self) -> Option<usize> {
        let mut input = self.get_input();

        while input != ERR && input != 13 && input != 'q' as i32 && input != 27 {
            if input == 'r' as i32 {
                self.scan();
                self.highlight = 0;
            } else if self.highlight > 0 && input == 65 {
                self.highlight -= 1;
            } else if self.highlight < self.networks.len() - 1 && input == 66 {
                self.highlight += 1;
            } else if input == 'd' as i32 {
                if self.networks[self.highlight].in_use {
                    self.disconnect(&self.networks[self.highlight].ssid);
                    self.run_scan();
                }
            } else if input == 'f' as i32 {
                if self.is_password_cached(&self.networks[self.highlight].ssid) {
                    self.forget_password(&self.networks[self.highlight].ssid);
                    self.run_scan();
                }
            }

            self.display_networks();
            input = self.get_input();
        }

        if input == ERR || input == 'q' as i32 || input == 27 {
            return None;
        }

        Some(self.highlight)
    }

    fn is_password_cached(&self, network: &String) -> bool {
        Command::new("nmcli")
            .args(&[
                "-t",
                "-f",
                "802-11-wireless-security.psk",
                "connection",
                "show",
                network,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|child| child.wait_with_output())
            .unwrap()
            .status
            .success()
    }

    fn get_password(&self) -> Zeroizing<String> {
        let height = 3;
        let width = 50;
        let password_win = newwin(height, width, (LINES() - height) / 2, (COLS() - width) / 2);
        box_(password_win, 0, 0);
        let _ = mvwprintw(password_win, 1, 1, "Enter password:");
        wrefresh(password_win);

        // Capture user input
        let mut password = Zeroizing::new(String::new());
        let mut ch: i32;
        loop {
            ch = wgetch(password_win);
            if ch == 13 {
                break; // Enter key
            } else if ch == 27 {
                password.clear(); // Escape key
                break; // Clear password and exit
            } else if ch == ncurses::KEY_BACKSPACE || ch == 127 || ch == 8 {
                // Handle backspace
                if !password.is_empty() {
                    password.pop();

                    let mut x: i32 = 0;
                    let mut y: i32 = 0;
                    getyx(password_win, &mut y, &mut x);
                    mvwaddch(password_win, y, x - 1, ' ' as u32);
                    wmove(password_win, y, x - 1);
                }
            } else if !(ch as u8 as char).is_ascii_control() {
                password.push(ch as u8 as char);
                waddch(password_win, 'o' as u32);
            }
            wrefresh(password_win);
        }
        delwin(password_win);
        password
    }

    pub fn connect(&self, index: usize) {
        if self.networks[index].bssid.is_empty() || self.networks[index].in_use {
            return;
        }

        self.ui.clear();
        let mut cmd = Command::new("nmcli");
        if self.is_password_cached(&self.networks[index].ssid) {
            cmd.args(&["con", "up", "id", self.networks[index].ssid.as_str()]);
        } else {
            // Create a new window for password input
            let pass_win = newwin(3, COLS(), LINES() / 2 - 1, 0);
            ncurses::werase(pass_win);
            let password = self.get_password();
            delwin(pass_win);

            cmd.args(&["dev", "wifi", "connect", &self.networks[index].bssid]);

            if !password.is_empty() {
                cmd.args(&["password", password.as_str()]);
            }
        }
        cmd.stderr(Stdio::null()).stdout(Stdio::null());
        self.ui.clear();

        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));
        let network_for_thread = self.networks[index].ssid.clone();
        let loading_thread = thread::spawn(move || unsafe {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                } else {
                    ui.loading_animation(
                        format!("Connecting to {}...", network_for_thread).as_str(),
                    );
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });
        let _ = cmd.spawn().and_then(|mut child| child.wait());
        tx.send(()).unwrap();
        loading_thread.join().unwrap();
    }

    fn disconnect(&self, network: &String) {
        if network.is_empty() {
            return;
        }

        let mut cmd = Command::new("nmcli");
        cmd.args(&["con", "down", "id", network])
            .stderr(Stdio::null())
            .stdout(Stdio::null());
        self.ui.clear();

        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));
        let network_for_thread = network.clone();
        let loading_thread = thread::spawn(move || unsafe {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                } else {
                    ui.loading_animation(
                        format!("Disconnecting from {}...", network_for_thread).as_str(),
                    );
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });
        let _ = cmd.spawn().and_then(|mut child| child.wait());
        tx.send(()).unwrap();
        loading_thread.join().unwrap();
    }

    fn forget_password(&self, network: &String) {
        if network.is_empty() {
            return;
        }

        let mut cmd = Command::new("nmcli");
        cmd.args(&["connection", "delete", network])
            .stderr(Stdio::null())
            .stdout(Stdio::null());
        self.ui.clear();

        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));
        let network_for_thread = network.clone();
        let loading_thread = thread::spawn(move || unsafe {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                } else {
                    ui.loading_animation(
                        format!("Forgetting password for {}...", network_for_thread).as_str(),
                    );
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });
        let _ = cmd.spawn().and_then(|mut child| child.wait());
        tx.send(()).unwrap();
        loading_thread.join().unwrap();
    }
}

unsafe impl Sync for NetworkUi {}
