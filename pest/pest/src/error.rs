// pest. The Elegant Parser
// Copyright (c) 2018 Dragoș Tiselice
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Types for different kinds of parsing failures.

use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp;
use core::fmt;
use core::mem;

use position::Position;
use span::Span;
use RuleType;

/// Parse-related error type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Error<R> {
    /// Variant of the error
    pub variant: ErrorVariant<R>,
    pub locations: Vec<LocatedError>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocatedError {
    /// Location within the input string
    pub location: InputLocation,
    /// Line/column within the input string
    pub line_col: LineColLocation,
    path: Option<String>,
    line: String,
    continued_line: Option<String>,
}

impl LocatedError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_from_pos(pos: Position) -> Self {
        Self {
            location: InputLocation::Pos(pos.pos()),
            path: None,
            line: visualize_whitespace(pos.line_of()),
            continued_line: None,
            line_col: LineColLocation::Pos(pos.line_col()),
        }
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_from_span(span: Span) -> Self {
        let end = span.end_pos();

        let mut end_line_col = end.line_col();
        // end position is after a \n, so we want to point to the visual lf symbol
        if end_line_col.1 == 1 {
            let mut visual_end = end.clone();
            visual_end.skip_back(1);
            let lc = visual_end.line_col();
            end_line_col = (lc.0, lc.1 + 1);
        };

        let mut line_iter = span.lines();
        let start_line = visualize_whitespace(&line_iter.next().unwrap_or_default());
        let continued_line = line_iter.last().map(|x| visualize_whitespace(&x));

        Self {
            location: InputLocation::Span((span.start(), end.pos())),
            path: None,
            line: start_line,
            continued_line,
            line_col: LineColLocation::Span(span.start_pos().line_col(), end_line_col),
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_owned());
        self
    }
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    fn start(&self) -> (usize, usize) {
        match self.line_col {
            LineColLocation::Pos(line_col) => line_col,
            LineColLocation::Span(start_line_col, _) => start_line_col,
        }
    }

    fn spacing(&self) -> String {
        let line = match self.line_col {
            LineColLocation::Pos((line, _)) => line,
            LineColLocation::Span((start_line, _), (end_line, _)) => cmp::max(start_line, end_line),
        };

        let line_str_len = format!("{}", line).len();

        let mut spacing = String::new();
        for _ in 0..line_str_len {
            spacing.push(' ');
        }

        spacing
    }

    fn underline(&self) -> String {
        let mut underline = String::new();

        let mut start = self.start().1;
        let end = match self.line_col {
            LineColLocation::Span(_, (_, mut end)) => {
                let inverted_cols = start > end;
                if inverted_cols {
                    mem::swap(&mut start, &mut end);
                    start -= 1;
                    end += 1;
                }

                Some(end)
            }
            _ => None,
        };
        let offset = start - 1;
        let line_chars = self.line.chars();

        for c in line_chars.take(offset) {
            match c {
                '\t' => underline.push('\t'),
                _ => underline.push(' '),
            }
        }

        if let Some(end) = end {
            if end - start > 1 {
                underline.push('^');
                for _ in 2..(end - start) {
                    underline.push('-');
                }
                underline.push('^');
            } else {
                underline.push('^');
            }
        } else {
            underline.push_str("^---")
        }

        underline
    }

    fn format(&self, before: Option<&LocatedError>, spacing: &str) -> String {
        let path = self
            .path
            .as_ref()
            .map(|path| format!("{}:", path))
            .unwrap_or_default();
        let file = if before.map(|x| self.path == x.path).unwrap_or(false) {
            format!("")
        } else {
            if before.is_none() {
                format!(
                    "{s}===> {p}{l}:{c}\n",
                    s = spacing,
                    l = self.start().0,
                    p = path,
                    c = self.start().1
                )
            } else {
                format!(
                    "{s} |=> {p}{l}:{c}\n",
                    s = spacing,
                    l = self.start().0,
                    p = path,
                    c = self.start().1
                )
            }
        };

        let pair = (self.line_col.clone(), &self.continued_line);
        if let (LineColLocation::Span(_, end), &Some(ref continued_line)) = pair {
            let has_line_gap = end.0 - self.start().0 > 1;
            if has_line_gap {
                format!(
                    "{file}\
                     {s    } |\n\
                     {ls:w$} | {line}\n\
                     {s    } | ...\n\
                     {le:w$} | {continued_line}\n\
                     {s    } | {underline}\n",
                    file = file,
                    s = spacing,
                    w = spacing.len(),
                    ls = self.start().0,
                    le = end.0,
                    line = self.line,
                    continued_line = continued_line,
                    underline = self.underline()
                )
            } else {
                format!(
                    "{file}\
                     {s    } |\n\
                     {ls:w$} | {line}\n\
                     {le:w$} | {continued_line}\n\
                     {s    } | {underline}\n",
                    file = file,
                    s = spacing,
                    w = spacing.len(),
                    ls = self.start().0,
                    le = end.0,
                    line = self.line,
                    continued_line = continued_line,
                    underline = self.underline()
                )
            }
        } else {
            format!(
                "{file}\
                 {s} |\n\
                 {l:w$} | {line}\n\
                 {s} | {underline}\n",
                s = spacing,
                w = spacing.len(),
                file = file,
                l = self.start().0,
                line = self.line,
                underline = self.underline()
            )
        }
    }
}

/// Different kinds of parsing errors.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ErrorVariant<R> {
    /// Generated parsing error with expected and unexpected `Rule`s
    ParsingError {
        /// Positive attempts
        positives: Vec<R>,
        /// Negative attempts
        negatives: Vec<R>,
    },
    /// Custom error with a message
    CustomError {
        /// Short explanation
        message: String,
    },
}

/// Where an `Error` has occurred.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InputLocation {
    /// `Error` was created by `Error::new_from_pos`
    Pos(usize),
    /// `Error` was created by `Error::new_from_span`
    Span((usize, usize)),
}

/// Line/column where an `Error` has occurred.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LineColLocation {
    /// Line/column pair if `Error` was created by `Error::new_from_pos`
    Pos((usize, usize)),
    /// Line/column pairs if `Error` was created by `Error::new_from_span`
    Span((usize, usize), (usize, usize)),
}

impl<R: RuleType> Error<R> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(variant: ErrorVariant<R>, locations: Vec<LocatedError>) -> Error<R> {
        Error { variant, locations }
    }

    pub fn renamed_rules<F>(mut self, f: F) -> Error<R>
    where
        F: FnMut(&R) -> String,
    {
        let variant = match self.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                let message = Error::parsing_error_message(&positives, &negatives, f);
                ErrorVariant::CustomError { message }
            }
            variant => variant,
        };

        self.variant = variant;

        self
    }

    fn message(&self) -> String {
        self.variant.message().to_string()
    }

    fn parsing_error_message<F>(positives: &[R], negatives: &[R], mut f: F) -> String
    where
        F: FnMut(&R) -> String,
    {
        match (negatives.is_empty(), positives.is_empty()) {
            (false, false) => format!(
                "unexpected {}; expected {}",
                Error::enumerate(negatives, &mut f),
                Error::enumerate(positives, &mut f)
            ),
            (false, true) => format!("unexpected {}", Error::enumerate(negatives, &mut f)),
            (true, false) => format!("expected {}", Error::enumerate(positives, &mut f)),
            (true, true) => "unknown parsing error".to_owned(),
        }
    }

    fn enumerate<F>(rules: &[R], f: &mut F) -> String
    where
        F: FnMut(&R) -> String,
    {
        match rules.len() {
            1 => f(&rules[0]),
            2 => format!("{} or {}", f(&rules[0]), f(&rules[1])),
            l => {
                let separated = rules
                    .iter()
                    .take(l - 1)
                    .map(|r| f(r))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}, or {}", separated, f(&rules[l - 1]))
            }
        }
    }

    pub(crate) fn format(&self) -> String {
        let spacing: String = (0..self
            .locations
            .iter()
            .map(|x| x.spacing().len())
            .max()
            .unwrap())
            .map(|_| ' ')
            .collect();
        let mut s = self
            .locations
            .iter()
            .fold((None, String::new()), |(a, mut b), c| {
                b.push_str(&c.format(a, &spacing));
                (Some(c), b)
            })
            .1;
        s.push_str(&format!("{} = {}", spacing, self.message()));
        s
    }
}

impl<R: RuleType> ErrorVariant<R> {
    ///
    /// Returns the error message for [`ErrorVariant`]
    ///
    /// If [`ErrorVariant`] is [`CustomError`], it returns a
    /// [`Cow::Borrowed`] reference to [`message`]. If [`ErrorVariant`] is [`ParsingError`], a
    /// [`Cow::Owned`] containing "expected [positives] [negatives]" is returned.
    ///
    /// [`ErrorVariant`]: enum.ErrorVariant.html
    /// [`CustomError`]: enum.ErrorVariant.html#variant.CustomError
    /// [`ParsingError`]: enum.ErrorVariant.html#variant.ParsingError
    /// [`Cow::Owned`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html#variant.Owned
    /// [`Cow::Borrowed`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html#variant.Borrowed
    /// [`message`]: enum.ErrorVariant.html#variant.CustomError.field.message
    /// # Examples
    ///
    /// ```
    /// # use pest::error::ErrorVariant;
    /// let variant = ErrorVariant::<()>::CustomError {
    ///     message: String::from("unexpected error")
    /// };
    ///
    /// println!("{}", variant.message());
    pub fn message(&self) -> Cow<str> {
        match self {
            ErrorVariant::ParsingError {
                ref positives,
                ref negatives,
            } => Cow::Owned(Error::parsing_error_message(positives, negatives, |r| {
                format!("{:?}", r)
            })),
            ErrorVariant::CustomError { ref message } => Cow::Borrowed(message),
        }
    }
}

impl<R: RuleType> fmt::Display for Error<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(feature = "std")]
impl<R: RuleType> std::error::Error for Error<R> {
    fn description(&self) -> &str {
        match self.variant {
            ErrorVariant::ParsingError { .. } => "parsing error",
            ErrorVariant::CustomError { ref message } => message,
        }
    }
}

fn visualize_whitespace(input: &str) -> String {
    input.to_owned().replace('\r', "␍").replace('\n', "␊")
}
