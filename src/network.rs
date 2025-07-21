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

// Constants for UI and signal thresholds
const STRONG_SIGNAL_THRESHOLD: u8 = 66;
const MEDIUM_SIGNAL_THRESHOLD: u8 = 33;
const ENTER_KEY: i32 = 13;
const ESCAPE_KEY: i32 = 27;
const UP_ARROW: i32 = 65;
const DOWN_ARROW: i32 = 66;
const BACKSPACE_KEY: i32 = 127;
const BACKSPACE_KEY_ALT: i32 = 8;

// Error types for better error handling
#[derive(Debug)]
#[allow(dead_code)] // Allow unused variants for future use
pub enum NetworkError {
    CommandFailed(String),
    NoNetworks,
    InvalidInput,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub in_use: bool,
    pub ssid: String,
    pub bssid: String,
    pub security: String,
    pub signal: u8,
}

impl Network {
    pub fn new() -> Self {
        Network {
            in_use: false,
            ssid: String::new(),
            bssid: String::new(),
            security: String::new(),
            signal: 0,
        }
    }

    #[allow(dead_code)] // Keep for future use
    pub fn is_empty(&self) -> bool {
        self.ssid.is_empty() && self.bssid.is_empty()
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
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

        let (_max_ssid_length, max_security_length) = self.calculate_max_lengths();

        // Calculate the maximum length of SSID and security strings to fit the window
        // -6 for padding and formatting
        let max_combined_length: usize =
            std::cmp::max(0, getmaxx(self.ui.win()) - max_security_length as i32 - 6) as usize;
        // Calculate the window height, leaving space for the header and footer
        let win_height: usize = std::cmp::max(0, getmaxy(self.ui.win()) - 4) as usize;

        let (start_index, end_index) = self.calculate_display_range(win_height);

        self.draw_header_and_footer();

        self.render_networks(
            start_index,
            end_index,
            max_combined_length,
            max_security_length,
        );

        wrefresh(self.ui.win());
    }

    fn calculate_max_lengths(&self) -> (usize, usize) {
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
        (max_ssid_length, max_security_length)
    }

    fn calculate_display_range(&self, win_height: usize) -> (usize, usize) {
        let start_index: usize =
            std::cmp::max(0, self.highlight as i32 - win_height as i32 + 1) as usize;
        let end_index: usize = std::cmp::min(self.networks.len(), start_index + win_height);
        (start_index, end_index)
    }

    fn draw_header_and_footer(&self) {
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
    }

    fn render_networks(
        &self,
        start_index: usize,
        end_index: usize,
        max_combined_length: usize,
        max_security_length: usize,
    ) {
        for i in start_index..end_index {
            let color = self.get_signal_color(self.networks[i].signal);
            self.render_single_network(
                i,
                start_index,
                color,
                max_combined_length,
                max_security_length,
            );
        }
    }

    fn get_signal_color(&self, signal: u8) -> u32 {
        if signal >= STRONG_SIGNAL_THRESHOLD {
            COLOR_PAIR(3)
        } else if signal >= MEDIUM_SIGNAL_THRESHOLD {
            COLOR_PAIR(2)
        } else {
            COLOR_PAIR(1)
        }
    }

    fn render_single_network(
        &self,
        i: usize,
        start_index: usize,
        mut color: u32,
        max_combined_length: usize,
        max_security_length: usize,
    ) {
        let mut ss = String::new();
        if self.networks[i].in_use {
            color |= ncurses::A_BOLD();
            ss.push_str("> ");
        } else {
            ss.push_str("  ");
        }

        let ssid = self.format_ssid(&self.networks[i].ssid, max_combined_length);
        let security = self.format_security(&self.networks[i].security);

        ss.push_str(&format!(
            "{:<width_ssid$}{:<width_security$}  ",
            ssid,
            security,
            width_ssid = max_combined_length,
            width_security = max_security_length,
        ));

        let display_color = if i == self.highlight {
            color | ncurses::A_REVERSE()
        } else {
            color
        };

        wattron(self.ui.win(), display_color);
        let _ = mvwprintw(self.ui.win(), (i - start_index + 3) as i32, 1, &ss);
        wattroff(self.ui.win(), display_color);
    }

    fn format_ssid(&self, ssid: &str, max_length: usize) -> String {
        let mut result = if ssid.is_empty() {
            String::from("---")
        } else {
            ssid.to_string()
        };

        if result.len() > max_length {
            result.truncate(max_length - 3);
            result.push_str("...");
        }
        result
    }

    fn format_security(&self, security: &str) -> String {
        if security.is_empty() {
            String::from("---")
        } else {
            security.to_string()
        }
    }

    pub fn run_scan(&mut self) -> Result<(), NetworkError> {
        self.networks.clear();
        let output = Command::new("nmcli")
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
            .spawn();

        let mut child = match output {
            Ok(child) => child,
            Err(e) => return Err(NetworkError::CommandFailed(e.to_string())),
        };

        let reader = BufReader::new(child.stdout.take().unwrap());

        let mut network = Network::new();

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
                network = Network::new();
            }
        }

        self.networks.sort_by(|a, b| b.signal.cmp(&a.signal));
        let _ = child.wait();

        if self.networks.is_empty() {
            Err(NetworkError::NoNetworks)
        } else {
            Ok(())
        }
    }

    pub fn scan(&mut self) {
        self.ui.clear();
        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));
        let loading_thread = thread::spawn(move || {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if rx.try_recv().is_ok() {
                    break;
                } else {
                    ui.loading_animation("Scanning for networks...");
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });

        let _ = self.run_scan(); // Ignore scan errors for now
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
        if input == ESCAPE_KEY {
            nodelay(self.ui.win(), true);
            input = wgetch(self.ui.win());
            nodelay(self.ui.win(), false);

            if input == ERR {
                return ESCAPE_KEY;
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

            return ESCAPE_KEY;
        }
        input
    }

    pub fn select_network(&mut self) -> Option<usize> {
        if self.networks.is_empty() {
            return None;
        }

        let mut input = self.get_input();

        while input != ERR && input != ENTER_KEY && input != 'q' as i32 && input != ESCAPE_KEY {
            match input {
                _ if input == 'r' as i32 => {
                    self.scan();
                    self.highlight = 0;
                }
                UP_ARROW if self.highlight > 0 => {
                    self.highlight -= 1;
                }
                DOWN_ARROW if self.highlight < self.networks.len() - 1 => {
                    self.highlight += 1;
                }
                _ if input == 'd' as i32 => {
                    if self.highlight < self.networks.len() && self.networks[self.highlight].in_use
                    {
                        self.disconnect(&self.networks[self.highlight].ssid);
                        let _ = self.run_scan();
                    }
                }
                _ if input == 'f' as i32 => {
                    if self.highlight < self.networks.len()
                        && self.is_password_cached(&self.networks[self.highlight].ssid)
                    {
                        self.forget_password(&self.networks[self.highlight].ssid);
                        let _ = self.run_scan();
                    }
                }
                _ => {} // Ignore unknown input
            }

            self.display_networks();
            input = self.get_input();
        }

        if input == ERR || input == 'q' as i32 || input == ESCAPE_KEY {
            return None;
        }

        if self.highlight < self.networks.len() {
            Some(self.highlight)
        } else {
            None
        }
    }

    fn is_password_cached(&self, network: &str) -> bool {
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
            .map(|output| output.status.success())
            .unwrap_or(false)
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
        loop {
            let ch = wgetch(password_win);
            if ch == ENTER_KEY {
                break; // Enter key
            } else if ch == ESCAPE_KEY {
                password.clear(); // Escape key
                break; // Clear password and exit
            } else if ch == ncurses::KEY_BACKSPACE || ch == BACKSPACE_KEY || ch == BACKSPACE_KEY_ALT
            {
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
        if index >= self.networks.len()
            || self.networks[index].bssid.is_empty()
            || self.networks[index].in_use
        {
            return;
        }

        self.ui.clear();
        let mut cmd = Command::new("nmcli");

        if self.is_password_cached(&self.networks[index].ssid) {
            cmd.args(&["con", "up", "id", self.networks[index].ssid.as_str()]);
        } else {
            // Create a new window for password input
            let pass_win = newwin(3, COLS(), LINES() / 2 - 1, 0);
            unsafe {
                werase(pass_win);
            }
            let password = self.get_password();
            delwin(pass_win);

            cmd.args(&["dev", "wifi", "connect", &self.networks[index].bssid]);

            if !password.is_empty() {
                cmd.args(&["password", password.as_str()]);
            }
        }
        cmd.stderr(Stdio::null()).stdout(Stdio::null());
        self.ui.clear();

        self.run_loading_animation(
            format!("Connecting to {}...", self.networks[index].ssid),
            move || {
                let _ = cmd.spawn().and_then(|mut child| child.wait());
            },
        );
    }

    fn disconnect(&self, network: &str) {
        if network.is_empty() {
            return;
        }

        let mut cmd = Command::new("nmcli");
        cmd.args(&["con", "down", "id", network])
            .stderr(Stdio::null())
            .stdout(Stdio::null());
        self.ui.clear();

        self.run_loading_animation(format!("Disconnecting from {}...", network), move || {
            let _ = cmd.spawn().and_then(|mut child| child.wait());
        });
    }

    fn forget_password(&self, network: &str) {
        if network.is_empty() {
            return;
        }

        let mut cmd = Command::new("nmcli");
        cmd.args(&["connection", "delete", network])
            .stderr(Stdio::null())
            .stdout(Stdio::null());
        self.ui.clear();

        self.run_loading_animation(
            format!("Forgetting password for {}...", network),
            move || {
                let _ = cmd.spawn().and_then(|mut child| child.wait());
            },
        );
    }

    fn run_loading_animation<F>(&self, message: String, operation: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let ui_clone = Arc::new(Mutex::new(self.ui.clone()));

        let loading_thread = thread::spawn(move || {
            let mut ui = ui_clone.lock().unwrap();
            loop {
                if rx.try_recv().is_ok() {
                    break;
                } else {
                    ui.loading_animation(&message);
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });

        // Run the operation in another thread
        let operation_thread = thread::spawn(operation);

        // Wait for operation to complete
        operation_thread.join().unwrap();

        // Signal loading thread to stop
        tx.send(()).unwrap();
        loading_thread.join().unwrap();
    }
}

impl Default for NetworkUi {
    fn default() -> Self {
        Self::new()
    }
}
