// pest. The Elegant Parser
// Copyright (c) 2018 Dragoș Tiselice
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ptr;
use core::str;

#[cfg(feature = "pretty-print")]
use serde::ser::SerializeStruct;

use super::flat_pairs::{self, FlatPairs};
use super::pair::{self, Pair};
use super::queueable_token::QueueableToken;
use super::tokens::{self, Tokens};
use RuleType;

/// An iterator over [`Pair`]s. It is created by [`pest::state`] and [`Pair::into_inner`].
///
/// [`Pair`]: struct.Pair.html
/// [`pest::state`]: ../fn.state.html
/// [`Pair::into_inner`]: struct.Pair.html#method.into_inner
#[derive(Clone)]
pub struct Pairs<R> {
    queue: Rc<Vec<QueueableToken<R>>>,
    input: Rc<String>,
    start: usize,
    end: usize,
}

pub fn new<R: RuleType>(
    queue: Rc<Vec<QueueableToken<R>>>,
    input: Rc<String>,
    start: usize,
    end: usize,
) -> Pairs<R> {
    Pairs {
        queue,
        input,
        start,
        end,
    }
}

impl<R: RuleType> Pairs<R> {
    /// Captures a slice from the `&str` defined by the starting position of the first token `Pair`
    /// and the ending position of the last token `Pair` of the `Pairs`. This also captures
    /// the input between those two token `Pair`s.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {
    ///     a,
    ///     b
    /// }
    ///
    /// let input = "a b";
    /// let pairs = pest::state(input, |state| {
    ///     // generating Token pairs with Rule::a and Rule::b ...
    /// #     state.rule(Rule::a, |s| s.match_string("a")).and_then(|s| s.skip(1))
    /// #         .and_then(|s| s.rule(Rule::b, |s| s.match_string("b")))
    /// }).unwrap();
    ///
    /// assert_eq!(pairs.as_str(), "a b");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        if self.start < self.end {
            let start = self.pos(self.start);
            let end = self.pos(self.end - 1);
            // Generated positions always come from Positions and are UTF-8 borders.
            &self.input[start..end]
        } else {
            ""
        }
    }

    /// Captures inner token `Pair`s and concatenates resulting `&str`s. This does not capture
    /// the input between token `Pair`s.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {
    ///     a,
    ///     b
    /// }
    ///
    /// let input = "a b";
    /// let pairs = pest::state(input, |state| {
    ///     // generating Token pairs with Rule::a and Rule::b ...
    /// #     state.rule(Rule::a, |s| s.match_string("a")).and_then(|s| s.skip(1))
    /// #         .and_then(|s| s.rule(Rule::b, |s| s.match_string("b")))
    /// }).unwrap();
    ///
    /// assert_eq!(pairs.concat(), "ab");
    /// ```
    #[inline]
    pub fn concat(&self) -> String {
        self.clone()
            .fold(String::new(), |string, pair| string + pair.as_str())
    }

    /// Flattens the `Pairs`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {
    ///     a,
    ///     b
    /// }
    ///
    /// let input = "";
    /// let pairs = pest::state(input, |state| {
    ///     // generating nested Token pair with Rule::b inside Rule::a
    /// #     state.rule(Rule::a, |state| {
    /// #         state.rule(Rule::b, |s| Ok(s))
    /// #     })
    /// }).unwrap();
    /// let tokens: Vec<_> = pairs.flatten().tokens().collect();
    ///
    /// assert_eq!(tokens.len(), 4);
    /// ```
    #[inline]
    pub fn flatten(self) -> FlatPairs<R> {
        unsafe { flat_pairs::new(self.queue, self.input, self.start, self.end) }
    }

    /// Returns the `Tokens` for the `Pairs`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use pest;
    /// # #[allow(non_camel_case_types)]
    /// # #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    /// enum Rule {
    ///     a
    /// }
    ///
    /// let input = "";
    /// let pairs = pest::state(input, |state| {
    ///     // generating Token pair with Rule::a ...
    /// #     state.rule(Rule::a, |s| Ok(s))
    /// }).unwrap();
    /// let tokens: Vec<_> = pairs.tokens().collect();
    ///
    /// assert_eq!(tokens.len(), 2);
    /// ```
    #[inline]
    pub fn tokens(self) -> Tokens<R> {
        tokens::new(self.queue, self.input, self.start, self.end)
    }

    /// Peek at the first inner `Pair` without changing the position of this iterator.
    #[inline]
    pub fn peek(&self) -> Option<Pair<R>> {
        if self.start < self.end {
            Some(unsafe { pair::new(Rc::clone(&self.queue), self.input.clone(), self.start) })
        } else {
            None
        }
    }

    /// Generates a string that stores the lexical information of `self` in
    /// a pretty-printed JSON format.
    #[cfg(feature = "pretty-print")]
    pub fn to_json(&self) -> String {
        ::serde_json::to_string_pretty(self).expect("Failed to pretty-print Pairs to json.")
    }

    fn pair(&self) -> usize {
        match self.queue[self.start] {
            QueueableToken::Start {
                end_token_index, ..
            } => end_token_index,
            _ => unreachable!(),
        }
    }

    fn pair_from_end(&self) -> usize {
        match self.queue[self.end - 1] {
            QueueableToken::End {
                start_token_index, ..
            } => start_token_index,
            _ => unreachable!(),
        }
    }

    fn pos(&self, index: usize) -> usize {
        match self.queue[index] {
            QueueableToken::Start { input_pos, .. } | QueueableToken::End { input_pos, .. } => {
                input_pos
            }
        }
    }
}

impl<R: RuleType> Iterator for Pairs<R> {
    type Item = Pair<R>;

    fn next(&mut self) -> Option<Self::Item> {
        let pair = self.peek()?;
        self.start = self.pair() + 1;
        Some(pair)
    }
}

impl<R: RuleType> DoubleEndedIterator for Pairs<R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end <= self.start {
            return None;
        }

        self.end = self.pair_from_end();

        let pair = unsafe { pair::new(Rc::clone(&self.queue), self.input.clone(), self.end) };

        Some(pair)
    }
}

impl<R: RuleType> fmt::Debug for Pairs<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<R: RuleType> fmt::Display for Pairs<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.clone()
                .map(|pair| format!("{}", pair))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<R: PartialEq> PartialEq for Pairs<R> {
    fn eq(&self, other: &Pairs<R>) -> bool {
        Rc::ptr_eq(&self.queue, &other.queue)
            && self.input == other.input
            && self.start == other.start
            && self.end == other.end
    }
}

impl<R: Eq> Eq for Pairs<R> {}

impl<R: Hash> Hash for Pairs<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&*self.queue as *const Vec<QueueableToken<R>>).hash(state);
        self.input.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

#[cfg(feature = "pretty-print")]
impl<R: RuleType> ::serde::Serialize for Pairs<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let start = self.pos(self.start);
        let end = self.pos(self.end - 1);
        let pairs = self.clone().collect::<Vec<_>>();

        let mut ser = serializer.serialize_struct("Pairs", 2)?;
        ser.serialize_field("pos", &(start, end))?;
        ser.serialize_field("pairs", &pairs)?;
        ser.end()
    }
}
