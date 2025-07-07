use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

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

impl Keywords for &str {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Match<V> {
    Exact(V),
    Prefix(V),
}

impl<V> Match<V> {
    /// Extracts the inner value from the `Match`.
    pub fn into_inner(self) -> V {
        match self {
            Match::Exact(v) | Match::Prefix(v) => v,
        }
    }
}

impl<V> AsRef<V> for Match<V> {
    fn as_ref(&self) -> &V {
        match self {
            Match::Exact(v) | Match::Prefix(v) => v,
        }
    }
}

impl<V> PartialOrd for Match<V>
where
    V: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Match::Exact(a), Match::Exact(b)) => a.partial_cmp(b),
            (Match::Prefix(a), Match::Prefix(b)) => a.partial_cmp(b),
            (Match::Exact(_), Match::Prefix(_)) => Some(std::cmp::Ordering::Less),
            (Match::Prefix(_), Match::Exact(_)) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl<V> Ord for Match<V>
where
    V: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeywordMap<K, V>
where
    K: Keywords + Hash + Eq,
{
    data: Vec<V>,
    keys: HashMap<K, usize>,
    keyword_index: Vec<(String, usize)>,
}

impl<K, V> KeywordMap<K, V>
where
    K: Keywords + Hash + Eq,
{
    /// Creates a new `KeywordMap`.
    pub fn new() -> Self {
        KeywordMap {
            data: Vec::new(),
            keys: HashMap::new(),
            keyword_index: Vec::new(),
        }
    }

    /// Inserts a key-value pair into the `KeywordMap`.
    pub fn insert(&mut self, key: K, value: V) {
        let index = self.data.len();
        for keyword in key.ascii_keywords() {
            self.keyword_index.push((keyword.to_string(), index));
        }

        self.data.push(value);
        self.keys.insert(key, index);
    }

    /// Removes a key-value pair from the `KeywordMap` by its key.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.keys.remove(key.borrow())?;
        let value = self.data.remove(index);

        // Update the keyword index
        self.keyword_index.retain(|(_, idx)| *idx != index);

        // Adjust indices in the keyword index
        for (_, idx) in &mut self.keyword_index {
            if *idx > index {
                *idx -= 1;
            }
        }

        // Adjust indices in the keys map
        for (_, idx) in self.keys.iter_mut() {
            if *idx > index {
                *idx -= 1;
            }
        }

        Some(value)
    }

    /// Retrieves a value by its key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.keys
            .get(key.borrow())
            .and_then(|&index| self.data.get(index))
    }

    /// Retrieves a mutable reference to a value by its key.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.keys
            .get(key.borrow())
            .and_then(|&index| self.data.get_mut(index))
    }
}

impl<K, V> KeywordMap<K, V>
where
    K: Keywords + Hash + Eq + Borrow<str>,
{
    pub fn find_by_partial_keyword<'a>(
        &'a self,
        keyword: &str,
    ) -> impl Iterator<Item = Match<&'a V>> {
        let exact_match = self.keys.get(keyword).copied();

        let iter = self
            .keyword_index
            .iter()
            .filter(move |(k, _)| k.starts_with(keyword))
            .filter_map(move |(_, index)| {
                if let Some(exact_match_index) = exact_match {
                    if *index == exact_match_index {
                        return None; // Skip exact match if already found
                    }
                }

                Some(*index)
            })
            .map(|index| Match::Prefix(&self.data[index]));

        exact_match
            .into_iter()
            .map(|index| Match::Exact(&self.data[index]))
            .chain(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_map_insert_and_get() {
        let mut map = KeywordMap::new();
        map.insert("hello world", 1);
        map.insert("testing123", 2);

        assert_eq!(map.get(&"hello world"), Some(&1));
        assert_eq!(map.get(&"testing123"), Some(&2));
        assert_eq!(map.get(&"nonexistent"), None);
    }

    #[test]
    fn test_keyword_map_remove() {
        let mut map = KeywordMap::new();
        map.insert("hello world", 1);
        map.insert("testing123", 2);

        assert_eq!(map.remove(&"hello world"), Some(1));
        assert_eq!(map.get(&"hello world"), None);
        assert_eq!(map.get(&"testing123"), Some(&2));
    }

    #[test]
    fn test_keyword_map_find_by_keyword() {
        let mut map = KeywordMap::new();
        map.insert("hello world", 1);
        map.insert("testing123", 2);
        map.insert("hello testing", 3);
        map.insert("test", 4);

        let results: Vec<_> = map.find_by_partial_keyword("hello").collect();
        assert_eq!(results, vec![Match::Prefix(&1), Match::Prefix(&3)]);

        let results: Vec<_> = map.find_by_partial_keyword("test").collect();
        assert_eq!(
            results,
            vec![Match::Exact(&4), Match::Prefix(&2), Match::Prefix(&3)]
        );
    }
}
