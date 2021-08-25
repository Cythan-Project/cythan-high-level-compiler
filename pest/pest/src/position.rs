// pest. The Elegant Parser
// Copyright (c) 2018 Dragoș Tiselice
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Range;

use core::str;


use alloc::rc::Rc;
use alloc::string::String;
use span;

/// A cursor position in a `&str` which provides useful methods to manually parse that string.
#[derive(Clone)]
pub struct Position {
    input: Rc<String>,
    /// # Safety:
    ///
    /// `input[pos..]` must be a valid codepoint boundary (should not panic when indexing thus).
    pos: usize,
}

impl Position {
    /// Create a new `Position` without checking invariants. (Checked with `debug_assertions`.)
    ///
    /// # Safety:
    ///
    /// `input[pos..]` must be a valid codepoint boundary (should not panic when indexing thus).
    pub(crate) unsafe fn new_unchecked(input: Rc<String>, pos: usize) -> Position {
        debug_assert!(input.get(pos..).is_some());
        Position { input, pos }
    }

    /// Attempts to create a new `Position` at the given position. If the specified position is
    /// an invalid index, or the specified position is not a valid UTF8 boundary, then None is
    /// returned.
    ///
    /// # Examples
    /// ```
    /// # use pest::Position;
    /// let cheart = '💖';
    /// let heart = "💖";
    /// assert_eq!(Position::new(heart, 1), None);
    /// assert_ne!(Position::new(heart, cheart.len_utf8()), None);
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new(input: Rc<String>, pos: usize) -> Option<Position> {
        input.get(pos..).map(|_| Position {
            input: input.clone(),
            pos,
        })
    }

    /// Creates a `Position` at the start of a `&str`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pest::Position;
    /// let start = Position::from_start("");
    /// assert_eq!(start.pos(), 0);
    /// ```
    #[inline]
    pub fn from_start(input: Rc<String>) -> Position {
        // Position 0 is always safe because it's always a valid UTF-8 border.
        Position { input, pos: 0 }
    }

    /// Returns the byte position of this `Position` as a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pest::Position;
    /// let input = "ab";
    /// let mut start = Position::from_start(input);
    ///
    /// assert_eq!(start.pos(), 0);
    /// ```
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Creates a `Span` from two `Position`s.
    ///
    /// # Panics
    ///
    /// Panics if the positions come from different inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pest::Position;
    /// let input = "ab";
    /// let start = Position::from_start(input);
    /// let span = start.span(&start.clone());
    ///
    /// assert_eq!(span.start(), 0);
    /// assert_eq!(span.end(), 0);
    /// ```
    #[inline]
    pub fn span(&self, other: &Position) -> span::Span {
        // LOGIC WAS CHANGED HERE
        if self.input == other.input
        /* && self.input.get(self.pos..other.pos).is_some() */
        {
            // This is safe because the pos field of a Position should always be a valid str index.
            unsafe { span::Span::new_unchecked(self.input.clone(), self.pos, other.pos) }
        } else {
            // TODO: maybe a panic if self.pos < other.pos
            panic!("span created from positions from different inputs")
        }
    }

    /// Returns the line and column number of this `Position`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {}
    ///
    /// let input = "\na";
    /// let mut state: Box<pest::ParserState<Rule>> = pest::ParserState::new(input);
    /// let mut result = state.match_string("\na");
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap().position().line_col(), (2, 2));
    /// ```
    #[inline]
    pub fn line_col(&self) -> (usize, usize) {
        if self.pos > self.input.len() {
            panic!("position out of bounds");
        }

        let mut pos = self.pos;
        // Position's pos is always a UTF-8 border.
        let slice = &self.input[..pos];
        let mut chars = slice.chars().peekable();

        let mut line_col = (1, 1);

        while pos != 0 {
            match chars.next() {
                Some('\r') => {
                    if let Some(&'\n') = chars.peek() {
                        chars.next();

                        if pos == 1 {
                            pos -= 1;
                        } else {
                            pos -= 2;
                        }

                        line_col = (line_col.0 + 1, 1);
                    } else {
                        pos -= 1;
                        line_col = (line_col.0, line_col.1 + 1);
                    }
                }
                Some('\n') => {
                    pos -= 1;
                    line_col = (line_col.0 + 1, 1);
                }
                Some(c) => {
                    pos -= c.len_utf8();
                    line_col = (line_col.0, line_col.1 + 1);
                }
                None => unreachable!(),
            }
        }

        line_col
    }

    /// Returns the entire line of the input that contains this `Position`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {}
    ///
    /// let input = "\na";
    /// let mut state: Box<pest::ParserState<Rule>> = pest::ParserState::new(input);
    /// let mut result = state.match_string("\na");
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap().position().line_of(), "a");
    /// ```
    #[inline]
    pub fn line_of(&self) -> &str {
        if self.pos > self.input.len() {
            panic!("position out of bounds");
        };
        // Safe since start and end can only be valid UTF-8 borders.
        &self.input[self.find_line_start()..self.find_line_end()]
    }

    pub(crate) fn find_line_start(&self) -> usize {
        if self.input.is_empty() {
            return 0;
        };
        // Position's pos is always a UTF-8 border.
        let start = self
            .input
            .char_indices()
            .rev()
            .skip_while(|&(i, _)| i >= self.pos)
            .find(|&(_, c)| c == '\n');
        match start {
            Some((i, _)) => i + 1,
            None => 0,
        }
    }

    pub(crate) fn find_line_end(&self) -> usize {
        if self.input.is_empty() {
            0
        } else if self.pos == self.input.len() - 1 {
            self.input.len()
        } else {
            // Position's pos is always a UTF-8 border.
            let end = self
                .input
                .char_indices()
                .skip_while(|&(i, _)| i < self.pos)
                .find(|&(_, c)| c == '\n');
            match end {
                Some((i, _)) => i + 1,
                None => self.input.len(),
            }
        }
    }

    /// Returns `true` when the `Position` points to the start of the input `&str`.
    #[inline]
    pub(crate) fn at_start(&self) -> bool {
        self.pos == 0
    }

    /// Returns `true` when the `Position` points to the end of the input `&str`.
    #[inline]
    pub(crate) fn at_end(&self) -> bool {
        self.pos == self.input.len()
    }

    /// Skips `n` `char`s from the `Position` and returns `true` if the skip was possible or `false`
    /// otherwise. If the return value is `false`, `pos` will not be updated.
    #[inline]
    pub(crate) fn skip(&mut self, n: usize) -> bool {
        let skipped = {
            let mut len = 0;
            // Position's pos is always a UTF-8 border.
            let mut chars = (&self.input[self.pos..]).chars();
            for _ in 0..n {
                if let Some(c) = chars.next() {
                    len += c.len_utf8();
                } else {
                    return false;
                }
            }
            len
        };

        self.pos += skipped;
        true
    }

    /// Goes back `n` `char`s from the `Position` and returns `true` if the skip was possible or `false`
    /// otherwise. If the return value is `false`, `pos` will not be updated.
    #[inline]
    pub(crate) fn skip_back(&mut self, n: usize) -> bool {
        let skipped = {
            let mut len = 0;
            // Position's pos is always a UTF-8 border.
            let mut chars = (&self.input[..self.pos]).chars().rev();
            for _ in 0..n {
                if let Some(c) = chars.next() {
                    len += c.len_utf8();
                } else {
                    return false;
                }
            }
            len
        };

        self.pos -= skipped;
        true
    }

    /// Skips until one of the given `strings` is found. If none of the `strings` can be found,
    /// this function will return `false` but its `pos` will *still* be updated.
    #[inline]
    pub(crate) fn skip_until(&mut self, strings: &[&str]) -> bool {
        for from in self.pos..self.input.len() {
            let bytes = if let Some(string) = self.input.get(from..) {
                string.as_bytes()
            } else {
                continue;
            };

            for slice in strings.iter() {
                let to = slice.len();
                if Some(slice.as_bytes()) == bytes.get(0..to) {
                    self.pos = from;
                    return true;
                }
            }
        }

        self.pos = self.input.len();
        false
    }

    /// Matches the char at the `Position` against a filter function and returns `true` if a match
    /// was made. If no match was made, returns `false` and `pos` will not be updated.
    #[inline]
    pub(crate) fn match_char_by<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if let Some(c) = (&self.input[self.pos..]).chars().next() {
            if f(c) {
                self.pos += c.len_utf8();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Matches `string` from the `Position` and returns `true` if a match was made or `false`
    /// otherwise. If no match was made, `pos` will not be updated.
    #[inline]
    pub(crate) fn match_string(&mut self, string: &str) -> bool {
        let to = self.pos + string.len();

        if Some(string.as_bytes()) == self.input.as_bytes().get(self.pos..to) {
            self.pos = to;
            true
        } else {
            false
        }
    }

    /// Case-insensitively matches `string` from the `Position` and returns `true` if a match was
    /// made or `false` otherwise. If no match was made, `pos` will not be updated.
    #[inline]
    pub(crate) fn match_insensitive(&mut self, string: &str) -> bool {
        let matched = {
            let slice = &self.input[self.pos..];
            if let Some(slice) = slice.get(0..string.len()) {
                slice.eq_ignore_ascii_case(string)
            } else {
                false
            }
        };

        if matched {
            self.pos += string.len();
            true
        } else {
            false
        }
    }

    /// Matches `char` `range` from the `Position` and returns `true` if a match was made or `false`
    /// otherwise. If no match was made, `pos` will not be updated.
    #[inline]
    pub(crate) fn match_range(&mut self, range: Range<char>) -> bool {
        if let Some(c) = (&self.input[self.pos..]).chars().next() {
            if range.start <= c && c <= range.end {
                self.pos += c.len_utf8();
                return true;
            }
        }

        false
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Position").field("pos", &self.pos).finish()
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        self.input == other.input && self.pos == other.pos
    }
}

impl Eq for Position {}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        if self.input == other.input {
            self.pos.partial_cmp(&other.pos)
        } else {
            None
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Position) -> Ordering {
        self.partial_cmp(other)
            .expect("cannot compare positions from different strs")
    }
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.pos.hash(state);
    }
}
