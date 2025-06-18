use std::iter::Cycle;

use ncurses::{COLOR_PAIR, *};

pub struct Ui {
    win: WINDOW,
    spinner: Cycle<std::vec::IntoIter<&'static str>>,
}

unsafe impl Send for Ui {}

impl Ui {
    pub fn new() -> Ui {
        let _ = setlocale(LcCategory::all, "");
        initscr();
        start_color();

        // Initialize color pairs for signal levels
        init_pair(1, COLOR_RED, COLOR_BLACK);
        init_pair(2, COLOR_YELLOW, COLOR_BLACK);
        init_pair(3, COLOR_GREEN, COLOR_BLACK);
        init_pair(4, COLOR_CYAN, COLOR_BLACK);
        init_pair(5, COLOR_MAGENTA, COLOR_BLACK);
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        noecho();
        nonl();
        raw();

        use_default_colors();
        let win = newwin(LINES(), COLS(), 0, 0);
        Ui {
            win,
            spinner: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
                .into_iter()
                .cycle(),
        }
    }

    pub fn clear(&self) {
        werase(self.win);
        wrefresh(self.win);
    }

    pub unsafe fn loading_animation(&mut self, message: &str) {
        // Current terminal size
        let mut max_y: i32 = 0;
        let mut max_x: i32 = 0;
        getmaxyx(self.win, &mut max_y, &mut max_x);

        // Adjust window size if necessary
        wresize(self.win, max_y, max_x);
        mvwin(self.win, 0, 0);

        // Clear the window
        self::clear();
        refresh();

        // Draw the message with the spinner
        wattron(self.win, COLOR_PAIR(5).try_into().unwrap());
        let _ = mvwprintw(
            self.win,
            1,
            1,
            format!("{} {}", message, self.spinner.next().unwrap()).as_str(),
        );
        mvwhline(self.win, 2, 1, 0, getmaxx(self.win) - 2);
        wattroff(self.win, COLOR_PAIR(5).try_into().unwrap());

        // Refresh the window to show changes
        wrefresh(self.win);
    }

    pub(crate) fn win(&self) -> WINDOW {
        self.win
    }
}

impl Clone for Ui {
    fn clone(&self) -> Self {
        Ui {
            win: self.win,
            spinner: self.spinner.clone(),
        }
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        endwin();
    }
}
