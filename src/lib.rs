/// An iterator over ASCII keywords in a string.
struct AsciiKeywords<'a> {
    s: &'a [u8],
    index: usize,
}

impl<'a> AsciiKeywords<'a> {
    /// Creates a new `AsciiKeywords` iterator from a string slice.
    fn new(s: &'a str) -> Self {
        AsciiKeywords {
            s: s.as_bytes(),
            index: 0,
        }
    }
}

impl<'a> Iterator for AsciiKeywords<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.s.len() {
            return None;
        }

        let start = self.index;
        if self.s[start].is_ascii_alphabetic() {
            while self.index < self.s.len() && self.s[self.index].is_ascii_alphabetic() {
                self.index += 1;
            }
        } else if self.s[start].is_ascii_digit() {
            while self.index < self.s.len() && self.s[self.index].is_ascii_digit() {
                self.index += 1;
            }
        }

        let keyword = std::str::from_utf8(&self.s[start..self.index]).ok()?;
        if keyword.is_empty() {
            return None;
        }

        // Skip any non-alphanumeric characters
        while self.index < self.s.len() && !self.s[self.index].is_ascii_alphanumeric() {
            self.index += 1;
        }

        Some(keyword)
    }
}

/// Trait providing access to iterators over keywords in a string.
pub trait Keywords {
    /// Returns an iterator over the ASCII keywords in the string.
    ///
    /// A keyword is defined as a sequence of ASCII alphabetic or numeric characters separated by
    /// non- alphanumeric characters (e.g., whitespace, punctuation). Non-alphanumeric characters
    /// will not be returned in the output.
    ///
    /// Example usage:
    /// ```
    /// use keywords::Keywords;
    ///
    /// let text = "hello_world, testing123!";
    /// let mut keywords = text.ascii_keywords();
    ///
    /// assert_eq!(Some("hello"), keywords.next());
    /// assert_eq!(Some("world"), keywords.next());
    /// assert_eq!(Some("testing"), keywords.next());
    /// assert_eq!(Some("123"), keywords.next());
    /// assert_eq!(None, keywords.next());
    /// ```
    fn ascii_keywords(&self) -> impl Iterator<Item = &str> + '_;
}

impl Keywords for str {
    #[inline]
    fn ascii_keywords(&self) -> impl Iterator<Item = &str> + '_ {
        AsciiKeywords::new(self)
    }
}

impl Keywords for String {
    #[inline]
    fn ascii_keywords(&self) -> impl Iterator<Item = &str> + '_ {
        AsciiKeywords::new(self)
    }
}
