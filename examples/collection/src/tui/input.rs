#[derive(Clone, Debug, Default)]
pub struct Input {
    value: String,
    cursor: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum InputRequest {
    InsertChar(char),
    DeletePrevChar,
    DeleteNextChar,
    GoToStart,
    GoToEnd,
    GoToPrevChar,
    GoToNextChar,
}

impl Input {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn visual_cursor(&self) -> usize {
        self.cursor
    }

    pub fn handle(&mut self, request: InputRequest) {
        match request {
            InputRequest::InsertChar(c) => {
                if self.cursor >= self.value.len() {
                    self.value.push(c);
                } else {
                    self.value.insert(self.cursor, c);
                }
                self.cursor += c.len_utf8();
            },
            InputRequest::DeletePrevChar => {
                if self.cursor > 0 {
                    let prev_char_len = self.value[..self.cursor]
                        .chars()
                        .next_back()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor -= prev_char_len;
                    self.value.remove(self.cursor);
                }
            },
            InputRequest::DeleteNextChar => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            },
            InputRequest::GoToStart => {
                self.cursor = 0;
            },
            InputRequest::GoToEnd => {
                self.cursor = self.value.len();
            },
            InputRequest::GoToPrevChar => {
                if self.cursor > 0 {
                    let prev_char_len = self.value[..self.cursor]
                        .chars()
                        .next_back()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor -= prev_char_len;
                }
            },
            InputRequest::GoToNextChar => {
                if self.cursor < self.value.len() {
                    let next_char_len = self.value[self.cursor..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor += next_char_len;
                }
            },
        }
    }
}
