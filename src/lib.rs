use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

/// Represents a single chat entry.
pub struct ChatMessage {
    pub sender: String,   // "User", "AI", etc.
    pub content: String,  // Full message text
}

/// Handles rendering and scrolling of the chat messages.
pub struct ChatArea {
    messages: Vec<ChatMessage>,
    offset: usize,
    scrollbar_state: ScrollbarState,
    visible_capacity: usize,
}

impl ChatArea {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            offset: 0,
            scrollbar_state: ScrollbarState::default(),
            visible_capacity: 10, // default, will be updated in render
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        // Auto-scroll to bottom, clamped to max_scroll
        let max_scroll = self.messages.len().saturating_sub(self.visible_capacity);
        self.offset = max_scroll;
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let content_length = self.messages.len();
        let max_scroll = content_length.saturating_sub(self.visible_capacity);
        self.offset = (self.offset + lines).min(max_scroll);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Capacity = how many lines fit in visible area
        let visible_capacity = area.height as usize - 2; // account for borders
        self.visible_capacity = visible_capacity;

        // Total content length
        let content_length = self.messages.len();

        // Slice the messages to show only visible ones
        let items: Vec<ListItem> = self.messages.iter().skip(self.offset).take(visible_capacity).map(|m| {
            ListItem::new(format!("{}: {}", m.sender, m.content))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Chat"));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        // Update scrollbar state
        self.scrollbar_state = self.scrollbar_state.content_length(
            content_length.saturating_sub(visible_capacity).max(0)
        );
        self.scrollbar_state = self.scrollbar_state.position(self.offset);

        let split = Layout::horizontal([Constraint::Min(1), Constraint::Length(1)]).split(area);
        frame.render_widget(list, split[0]);
        frame.render_stateful_widget(scrollbar, split[1], &mut self.scrollbar_state);
    }

}

/// Handles multiline input editing.
pub struct InputArea {
    buffer: String,      // current typed text
    cursor: usize,       // cursor position in buffer
    offset: usize,       // scroll offset for display
}

impl InputArea {
    const MAX_DISPLAY_LINES: usize = 10;
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            offset: 0,
        }
    }

    pub fn calculate_display_lines(&self, width: u16) -> u16 {
        let effective_width = width.saturating_sub(4); // 2 for borders, 2 for "> "
        if effective_width == 0 {
            return 2;
        }
        let logical_lines: Vec<&str> = self.buffer.split('\n').collect();
        let mut total_lines = 0;
        for line in logical_lines {
            let line_len = line.len() as f32;
            let wrapped = (line_len / effective_width as f32).ceil() as usize;
            total_lines += wrapped.max(1);
        }
        let visible_lines = total_lines.min(Self::MAX_DISPLAY_LINES);
        (visible_lines as u16) + 2 // +2 for top and bottom borders
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor > self.buffer.len() {
            self.cursor = self.buffer.len();
        }
        self.buffer.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            // Find the start of the char before cursor
            let mut prev_start = 0;
            for (i, _) in self.buffer.char_indices() {
                if i >= self.cursor {
                    break;
                }
                prev_start = i;
            }
            self.buffer.remove(prev_start);
            self.cursor = prev_start;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            // Find the previous char boundary
            let mut prev = 0;
            for (i, _) in self.buffer.char_indices() {
                if i >= self.cursor {
                    break;
                }
                prev = i;
            }
            self.cursor = prev;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            if let Some((i, _)) = self.buffer.char_indices().find(|(i, _)| *i > self.cursor) {
                self.cursor = i;
            } else {
                // at the last char, move to end
                self.cursor = self.buffer.len();
            }
        }
    }

    fn find_current_line_col(&self) -> (usize, usize) {
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        let mut pos = 0; // byte position
        let mut current_line = 0;
        let mut current_col = 0;
        for (i, line) in lines.iter().enumerate() {
            let line_bytes = line.len();
            if pos + line_bytes >= self.cursor {
                current_line = i;
                current_col = self.buffer[pos..self.cursor].chars().count();
                break;
            }
            pos += line_bytes + 1; // +1 for \n
        }
        (current_line, current_col)
    }

    pub fn cursor_up(&mut self) {
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        if lines.is_empty() {
            return;
        }
        let (current_line, current_col) = self.find_current_line_col();
        if current_line > 0 {
            let prev_line = lines[current_line - 1];
            let prev_line_chars: Vec<char> = prev_line.chars().collect();
            let new_col = current_col.min(prev_line_chars.len());
            // Calculate byte position of prev line start
            let mut prev_line_start = 0;
            for i in 0..(current_line - 1) {
                prev_line_start += lines[i].len() + 1;
            }
            // Add byte offset for new_col chars
            let mut byte_offset = 0;
            for ch in prev_line.chars().take(new_col) {
                byte_offset += ch.len_utf8();
            }
            self.cursor = prev_line_start + byte_offset;
        }
    }

    pub fn cursor_down(&mut self) {
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        if lines.is_empty() {
            return;
        }
        let (current_line, current_col) = self.find_current_line_col();
        if current_line < lines.len() - 1 {
            let next_line = lines[current_line + 1];
            let next_line_chars: Vec<char> = next_line.chars().collect();
            let new_col = current_col.min(next_line_chars.len());
            // Calculate byte position of next line start
            let mut next_line_start = 0;
            for i in 0..(current_line + 1) {
                next_line_start += lines[i].len() + 1;
            }
            // Add byte offset for new_col chars
            let mut byte_offset = 0;
            for ch in next_line.chars().take(new_col) {
                byte_offset += ch.len_utf8();
            }
            self.cursor = next_line_start + byte_offset;
        }
    }

    pub fn newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn submit(&mut self) -> String {
        let input = self.buffer.clone();
        self.buffer.clear();
        self.cursor = 0;
        self.offset = 0;
        input
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.offset += lines;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    fn calculate_display_index(&self) -> usize {
        let prefix = "> ";
        let count_nl = self.buffer[..self.cursor].chars().filter(|&ch| ch == '\n').count();
        prefix.len() + self.cursor + count_nl * prefix.len()
    }

    fn calculate_cursor_line(&self, display: &str) -> usize {
        let display_index = self.calculate_display_index();
        let mut current_line = 0;
        let mut byte_index = 0;
        for ch in display.chars() {
            if byte_index >= display_index {
                return current_line;
            }
            if ch == '\n' {
                current_line += 1;
            }
            byte_index += ch.len_utf8();
        }
        current_line
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let full_display = format!("> {}", self.buffer.replace('\n', "\n> "));
        let lines: Vec<&str> = full_display.lines().collect();
        let total_lines = lines.len();
        let cursor_line = self.calculate_cursor_line(&full_display);
        let max_offset = total_lines.saturating_sub(Self::MAX_DISPLAY_LINES);

        // Auto-scroll to keep cursor visible
        if cursor_line < self.offset {
            self.offset = cursor_line;
        } else if cursor_line >= self.offset + Self::MAX_DISPLAY_LINES {
            self.offset = cursor_line.saturating_sub(Self::MAX_DISPLAY_LINES - 1);
        }
        self.offset = self.offset.min(max_offset);

        // Slice visible lines
        let end = (self.offset + Self::MAX_DISPLAY_LINES).min(total_lines);
        let visible_lines = &lines[self.offset..end];
        let display = visible_lines.join("\n");

        let paragraph = Paragraph::new(display)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(paragraph, area);
    }


}

/// Coordinates everything.
pub struct ChatApp {
    chat_area: ChatArea,
    input_area: InputArea,
    should_quit: bool,
    cursor_pos: Option<(u16, u16)>,
}

impl ChatApp {
    pub fn new() -> Self {
        Self {
            chat_area: ChatArea::new(),
            input_area: InputArea::new(),
            should_quit: false,
            cursor_pos: None,
        }
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input_area.newline();
                } else {
                    let input = self.input_area.submit();
                    if !input.trim().is_empty() {
                        self.chat_area.add_message(ChatMessage {
                            sender: "User".to_string(),
                            content: input,
                        });
                        // Simulate AI response
                        self.chat_area.add_message(ChatMessage {
                            sender: "AI".to_string(),
                            content: "Hello! This is a simulated response.".to_string(),
                        });
                    }
                }
            }
            KeyCode::PageUp => self.chat_area.scroll_up(5),
            KeyCode::PageDown => self.chat_area.scroll_down(5),
            KeyCode::Esc | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char(c) => self.input_area.insert_char(c),
            KeyCode::Backspace => self.input_area.backspace(),
            KeyCode::Left => self.input_area.cursor_left(),
            KeyCode::Right => self.input_area.cursor_right(),
            KeyCode::Up => self.input_area.cursor_up(),
            KeyCode::Down => self.input_area.cursor_down(),
            _ => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();
        let input_height = self.input_area.calculate_display_lines(size.width);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(input_height),
            ].as_ref())
            .split(size);
        self.chat_area.render(frame, chunks[0]);
        self.input_area.render(frame, chunks[1]);

        // Calculate cursor position
        let input_area = chunks[1];
        let full_display = format!("> {}", self.input_area.buffer.replace('\n', "\n> "));
        let lines: Vec<&str> = full_display.lines().collect();
        let total_lines = lines.len();
        let max_offset = total_lines.saturating_sub(10);
        let offset = self.input_area.get_offset().min(max_offset);
        let end = (offset + 10).min(total_lines);
        let visible_lines = &lines[offset..end];
        let display = visible_lines.join("\n");
        let display_index = self.input_area.calculate_display_index();
        // Calculate start_byte of visible display
        let mut start_byte = 0;
        for i in 0..offset {
            if i < lines.len() {
                start_byte += lines[i].len() + 1; // +1 for \n
            }
        }
        let adjusted_display_index = display_index.saturating_sub(start_byte);
        let cursor_pos = self.calculate_cursor_pos(&display, adjusted_display_index);
        if let Some((line, col)) = cursor_pos {
            let absolute_x = input_area.x + 1 + col;
            let absolute_y = input_area.y + 1 + line;
            self.cursor_pos = Some((absolute_x, absolute_y));
        } else {
            self.cursor_pos = None;
        }
    }



    fn calculate_cursor_pos(&self, display: &str, display_index: usize) -> Option<(u16, u16)> {
        let mut current_line = 0;
        let mut current_col = 0;
        let mut byte_index = 0;
        for ch in display.chars() {
            if byte_index == display_index {
                return Some((current_line as u16, current_col as u16));
            }
            if ch == '\n' {
                current_line += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
            byte_index += ch.len_utf8();
        }
        if byte_index == display_index {
            Some((current_line as u16, current_col as u16))
        } else {
            None
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn get_cursor_pos(&self) -> Option<(u16, u16)> {
        self.cursor_pos
    }
}