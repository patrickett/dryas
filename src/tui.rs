use crossterm::event;
use ratatui::{
    crossterm::event::*, layout::*, style::*, text::*, widgets::*, DefaultTerminal, Frame,
};
use strum::{EnumIter, FromRepr, IntoEnumIterator};

pub fn run() {
    // Standalone TUI does NOT run
    let terminal = ratatui::init();
    let _ = App::default().run(terminal);
    ratatui::restore();
}
#[derive(PartialEq, Default, EnumIter, FromRepr, Clone, Copy)]
pub enum Tab {
    #[default]
    Torrents,
    Settings,
    Search,
}

pub fn num_length(n: usize) -> usize {
    std::iter::successors(Some(n), |&n| (n >= 10).then_some(n / 10)).count()
}

impl Tab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Torrents => write!(f, "Torrents [1]"),
            Tab::Settings => write!(f, "Settings [2]"),
            Tab::Search => write!(f, "Search DHT [3]"),
        }
    }
}

pub enum Details {
    General,
    Trackers,
    Peers,
    HttpSources,
    Content,
}

#[derive(Default)]
pub struct SearchInput {
    value: String,
    cursor_index: usize,
}

const ITEM_HEIGHT: usize = 2;

#[derive(Default)]
struct App {
    /// This is true when the user is typing within the search bar
    editing: bool,
    search: SearchInput,

    selected_tab: Tab,
    item_index: usize,
}

impl App {
    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn move_up(&mut self) {
        // self.selected_tab = self.selected_tab.next();
    }

    pub fn move_down(&mut self) {
        // self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.search.cursor_index.saturating_sub(1);
        self.search.cursor_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.search.cursor_index.saturating_add(1);
        self.search.cursor_index = self.clamp_cursor(cursor_moved_right);
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search.value.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.search.cursor_index = 0;
    }
    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.search
            .value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.search.cursor_index)
            .unwrap_or(self.search.value.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.search.cursor_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.search.cursor_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search.value.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search.value.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search.value = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    // THIS SHIT BROKEN
    fn delete_word(&mut self) {
        let cursor_index = self.search.cursor_index;

        // Return early if the cursor is at the start or out of bounds
        if cursor_index == 0 || cursor_index > self.search.value.len() {
            return;
        }

        // Trim any whitespace directly before the cursor
        let trimmed_cursor = self.search.value[..cursor_index]
            .rfind(|c: char| !c.is_whitespace())
            .map_or(cursor_index, |i| i + 1);

        // Find the start of the previous word by scanning backwards for whitespace
        let start_of_word = self.search.value[..trimmed_cursor]
            .rfind(char::is_whitespace)
            .map_or(0, |i| i + 1);

        // Remove the range from the start of the word to the cursor
        self.search
            .value
            .replace_range(start_of_word..trimmed_cursor, "");

        // Clean up any extra whitespace that may result from the removal
        if start_of_word > 0
            && start_of_word < self.search.value.len()
            && self.search.value[start_of_word..].starts_with(' ')
        {
            self.search
                .value
                .replace_range(start_of_word..=start_of_word, "");
        }

        self.search.cursor_index -= self.clamp_cursor(trimmed_cursor - start_of_word);
        self.move_cursor_left()
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search.value.insert(index, new_char);
        self.move_cursor_right();
    }

    // fn render_traffic_info(&self, frame: &mut Frame, area: Rect) {
    //     // make download here green and upload yellow
    //     let total_download = Span::from("↓ 595.6 KiB/s").light_green();
    //     let total_separator = Span::from(" | ").dark_gray();
    //     let total_upload = Span::from("↑ 12.3 KiB/s").light_yellow();
    //     let total_traffic_line = Line::from(vec![total_download, total_separator, total_upload]);
    //     let text = Text::from(total_traffic_line).patch_style(Style::default());
    //     let help_message = Paragraph::new(text).centered();
    //     frame.render_widget(help_message, area);
    // }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let tab_list: Vec<Span> = Tab::iter().map(|tab| Span::from(tab.to_string())).collect();
        let selected_tab_index = self.selected_tab as usize;
        let tabs = Tabs::new(tab_list)
            .select(selected_tab_index)
            .divider(" | ")
            .style(Style::default().dark_gray())
            .highlight_style(Style::default().yellow().underlined());
        frame.render_widget(tabs, area);
    }

    fn render_torrent_table_compact(&self, frame: &mut Frame, area: Rect) {
        //   #   | name            | status      | down         | up         | done | seeders | peers | ratio
        // 10001 | ubuntu.iso      | downloading | 595.6 KiB/s  | 12.3 KiB/s | 55%  | 27 (80) | 5 (8) | 0.6
        // 10002 | arch.iso        | complete    |              |            | 100% |         |       | 2.0

        let header = Row::new([
            Cell::new("#"),
            Cell::new("done"),
            Cell::new("name"),
            Cell::new("status"),
            Cell::new("download"),
            Cell::new("upload"),
            Cell::new("seeders"),
            Cell::new("peers"),
            Cell::new("ratio"),
        ])
        .dark_gray()
        .bold();

        // TODO: merge downloading and seeders/active seeders
        // and merge upload with peers,active peers
        //
        // TODO: do the same thing with the total network activity "595.6 KiBs of 3(10)" and
        // move the bar at the top to at then bottom right or at the other
        // end of the tab list so that it doesnt take up its own row
        //

        let row_data = vec![
            Cell::new("1"),
            Cell::new("55%"),
            Cell::new("ubuntu-24.10-live-server-amd64.iso"),
            Cell::new("downloading"),
            Cell::new("595.6 KiB/s").green(),
            Cell::new("12.3 KiB/s").red(),
            Cell::new("27 (80)").green(),
            Cell::new("5 (8)").red(),
            Cell::new("0.6"),
        ];

        let rows = vec![Row::new(row_data)];

        let widths = [
            Constraint::Length(3),  // TODO: find the length of the number of torrents
            Constraint::Length(4),  // ...%
            Constraint::Min(10),    // growable
            Constraint::Length(11), //
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(5),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::bordered()
                // .border_style(Borders::BOTTOM)
                // .border_style(Borders::TOP)
                .style(Style::default().dark_gray()),
        );

        frame.render_widget(table, area);
    }

    fn render_torrent_table(&self, frame: &mut Frame, area: Rect) {
        // TODO: this likely will require some custom widget to render a custom table row.
        //   #   | name            | status      | down         | up         | done | seeders | peers | ratio
        // ubuntu.iso
        // 10001 | downloading | 595.6 KiB/s  | 12.3 KiB/s | 55%  | 27 (80) | 5 (8) | 0.6

        let header = Row::new([
            Cell::new("#"),
            Cell::new("done"),
            Cell::new("name"),
            Cell::new("status"),
            Cell::new("download"),
            Cell::new("upload"),
            Cell::new("seeders"),
            Cell::new("peers"),
            Cell::new("ratio"),
        ])
        .dark_gray()
        .bold();

        // TODO: merge downloading and seeders/active seeders
        // and merge upload with peers,active peers
        //
        // TODO: do the same thing with the total network activity "595.6 KiBs of 3(10)" and
        // move the bar at the top to at then bottom right or at the other
        // end of the tab list so that it doesnt take up its own row
        //

        // let row_data = vec![
        //     Cell::new("ubuntu-24.10-live-server-amd64.iso"),
        //     Cell::new("1"),
        //     Cell::new("55%"),
        //     Cell::new("downloading"),
        //     Cell::new("595.6 KiB/s").green(),
        //     Cell::new("12.3 KiB/s").red(),
        //     Cell::new("27 (80)").green(),
        //     Cell::new("5 (8)").red(),
        //     Cell::new("0.6"),
        // ];

        let rows = vec![
            // Row::new(row_data).height(2).on_gray(),
            //
            Row::new(Text::raw("ubuntu-24.10-live-server-amd64.iso")),
            Row::new(vec![
                Cell::new("1"),
                Cell::new("55%"),
                Cell::new("downloading"),
                Cell::new("595.6 KiB/s").green(),
                Cell::new("12.3 KiB/s").red(),
                Cell::new("27 (80)").green(),
                Cell::new("5 (8)").red(),
                Cell::new("0.6"),
                // Text::raw("asd asd asd asd asd"),
            ]),
            // Row::new(vec![
            //     Text::raw("Cell3-Line1Cell3-Line2Cell3-Line3"),
            //     Text::raw("Cell4-Line1Cell4-Line2Cell4-Line3"),
            // ])
            // .height(3),
        ];

        let widths = [
            Constraint::Percentage(100), // growable
        ];
        // let table = Table::new(rows, widths)

        let table = Table::default()
            .rows(rows)
            // .header(header)
            .widths(widths)
            .column_spacing(2)
            .highlight_symbol(">");

        frame.render_widget(table, area);
    }

    fn render_settings(&self, frame: &mut Frame, area: Rect) {
        let msgs: Vec<String> = vec![];
        let messages: Vec<ListItem> = msgs
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Settings"));
        frame.render_widget(messages, area);
    }

    fn render_search_input(&self, frame: &mut Frame, area: Rect) {
        // So for search, I think it makes sense to allow arrow down to select
        // the input element which it should then make it white to highlight selected element.
        // then if you [enter] you can begin typing and to exit it would be [esc]
        //
        // [enter] again while searching will submit it for a search
        //
        // you can also use [/] to go straight into typing to search again with [esc] to exit editing mode
        // let input_value = &self.search.value;

        // match self.input_mode {
        //     // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
        //     InputMode::Normal => {}

        //     // Make the cursor visible and ask ratatui to put it at the specified coordinates after
        //     // rendering
        //     #[allow(clippy::cast_possible_truncation)]
        //     InputMode::Editing => frame.set_cursor_position(Position::new(
        //         // Draw the cursor at the current position in the input field.
        //         // This position is can be controlled via the left and right arrow key
        //         input_area.x + self.character_index as u16 + 1,
        //         // Move one line down, from the border to the input line
        //         input_area.y + 1,
        //     )),
        // }
        //

        let input = Paragraph::new(self.search.value.to_owned())
            .style(if self.editing {
                Style::default()
                    .yellow()
                    .add_modifier(Modifier::RAPID_BLINK)
            } else {
                Style::default().dark_gray()
            })
            .block(Block::bordered().title(if self.editing {
                "Search [esc]"
            } else {
                "Search [/]"
            }));

        frame.render_widget(input, area);

        match self.editing {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            false => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            true => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                area.x + self.search.cursor_index as u16 + 1,
                // Move one line down, from the border to the input line
                area.y + 1,
            )),
        }
    }

    fn render_search(&self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([
            Constraint::Length(3), // searchbar
            Constraint::Min(1),
        ]);
        let [input_area, results_area] = vertical.areas(area);

        self.render_search_input(frame, input_area);

        let msgs: Vec<String> = vec!["Nothing Found".to_string()];
        let messages: Vec<ListItem> = msgs
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages =
            List::new(messages).block(Block::bordered().title("Search Results [r]").dark_gray());
        frame.render_widget(messages, results_area);
    }

    fn render_body(&self, frame: &mut Frame, area: Rect) {
        match self.selected_tab {
            Tab::Torrents => self.render_torrent_table_compact(frame, area),
            Tab::Settings => self.render_settings(frame, area),
            Tab::Search => self.render_search(frame, area),
        }
    }

    fn get_keybinds(&self) -> Vec<Span<'_>> {
        // TODO: fetch from config file

        let mut binds: Vec<&str> = Vec::new();

        // NOTE: try and keep dynamic keybinds to the front
        // so anything that is modified by state changes at the front
        // and the more common or repeated stay at the end of the list
        //
        // The idea is that if you don't know they you look bottom left and it
        // will inform based on state
        match &self.selected_tab {
            Tab::Torrents => {
                // TODO: have internal torrent_list().len()
                let torrent_list: Vec<()> = vec![];
                if !torrent_list.is_empty() {
                    // TODO: check if item_index (selected torrent)
                    // is paused or not
                    let active = false;
                    if active {
                        binds.push("Pause [space]");
                    } else {
                        binds.push("Start [space]");
                    }
                }

                binds.push("Move Up [↑] ");
                binds.push("Move Down [↓] ");

                // TODO: modal to paste magnet link or folder path
                // or BIG TODO: file exploerer to pick a torrent fiel
                binds.push("Add [a]");

                // TODO: add modal to have select list for status to filter by
                binds.push("Filter [f]");

                // TODO: add modal to have select list for columns to show
                // also add the option to change column ordering
                binds.push("Columns [c]");
            }
            Tab::Search => {
                if self.editing {
                    binds.push("Exit Search [esc]");
                } else {
                    binds.push("Search [/]");
                }
                let result_list: Vec<()> = vec![()];
                if !result_list.is_empty() {
                    // TODO: get index for selected item from dhtresults
                    binds.push("Add [a]");
                }
            }
            Tab::Settings => {
                binds.push("Edit [enter]");
            }
        };

        // TODO: quit button
        binds.push("Quit [q]");

        let separator = Span::from(" ");

        binds
            .into_iter()
            .map(|bind| Span::from(bind).on_blue().gray())
            .fold(Vec::new(), |mut acc, span| {
                if !acc.is_empty() {
                    acc.push(separator.clone()); // Insert separator if not the first element
                }
                acc.push(span); // Add transformed Span
                acc
            })
    }

    fn render_keybinds(&self, frame: &mut Frame, area: Rect) {
        let keybind_spans = self.get_keybinds();
        let text = Text::from(Line::from(keybind_spans)).patch_style(Style::default());
        // let help_message = Paragraph::new(text);
        frame.render_widget(text, area);
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            // Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [tab_area, messages_area, keymap_area] = vertical.areas(frame.area());

        // self.render_traffic_info(frame, top_info_area);
        self.render_tabs(frame, tab_area);
        self.render_body(frame, messages_area);
        self.render_keybinds(frame, keymap_area);
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), ()> {
        loop {
            terminal.draw(|frame| self.draw(frame)).expect("msg");
            let key_event = event::read().expect("msg");

            match self.editing {
                true => {
                    // put into search

                    //     InputMode::Editing if1 key.kind == KeyEventKind::Press => match key.code {
                    //         KeyCode::Enter => self.submit_message(),
                    //         KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    //         KeyCode::Backspace => self.delete_char(),
                    //         KeyCode::Left => self.move_cursor_left(),
                    //         KeyCode::Right => self.move_cursor_right(),
                    //         KeyCode::Esc => self.input_mode = InputMode::Normal,
                    //         _ => {}
                    //     },

                    if let Event::Key(key) = key_event {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => {
                                self.editing = false;
                            }
                            (KeyCode::Enter, _) => {
                                // TODO: perform search
                            }
                            (KeyCode::Left, _) => self.move_cursor_left(),
                            (KeyCode::Right, _) => self.move_cursor_right(),
                            (KeyCode::Char('w'), KeyModifiers::CONTROL) => self.delete_word(),
                            (KeyCode::Char(to_insert), _) => self.enter_char(to_insert),
                            (KeyCode::Backspace, _) => self.delete_char(),

                            _ => {}
                        }
                    }
                }
                false => {
                    if let Event::Key(key) = key_event {
                        match key.code {
                            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                            KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
                            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
                            KeyCode::Char('/') => match &self.selected_tab {
                                Tab::Torrents => {
                                    // TODO: allow filtering by name and status
                                }
                                Tab::Settings => {
                                    // TODO: allow searching settings
                                }
                                Tab::Search => {
                                    self.editing = true;
                                }
                            },
                            KeyCode::Esc => match self.selected_tab {
                                Tab::Torrents => todo!(),
                                Tab::Settings => todo!(),
                                Tab::Search => {
                                    self.editing = false;
                                }
                            },
                            KeyCode::Char('1') => {
                                self.selected_tab = Tab::Torrents;
                            }
                            KeyCode::Char('2') => {
                                self.selected_tab = Tab::Settings;
                            }
                            KeyCode::Char('3') => {
                                self.selected_tab = Tab::Search;
                            }

                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

// TODO: press i to toggle opening bottom half torrent info
// otherwise you have to click enter to select a torrent
// and it will open torrent info in a modal

// TODO: backspace or d on a selected torrent to get a confirm
// popup to remove/delete the torrent
// confirm y/N

// TODO: ? to open keybind modal
// if so we can remove bottom keybinds and/or make them toggleable
