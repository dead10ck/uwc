use std::collections::BTreeMap;
use std::fmt;
use std::str;

use log::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::constants::NEWLINES;

pub type Counted = BTreeMap<Counter, usize>;

/// Take all the counts in `other_counts` and sum them into `accum`.
pub fn sum_counts(accum: &mut Counted, other_counts: &Counted) {
    for (counter, count) in other_counts {
        let entry = accum.entry(*counter).or_insert(0);
        *entry += count;
    }
}

/// Sums all the `Counted` instances into a new one.
pub fn sum_all_counts<'a, I>(counts: I) -> Counted
where
    I: IntoIterator<Item = &'a Counted>,
{
    let mut totals = BTreeMap::new();

    for counts in counts {
        sum_counts(&mut totals, counts);
    }

    totals
}

/// Something that counts things in `&str`s.
pub trait Count {
    /// Counts something in the given `&str`.
    fn count(&self, s: &str) -> usize;
}

impl Count for Counter {
    fn count(&self, s: &str) -> usize {
        match *self {
            Counter::GraphemeCluster => s.graphemes(true).count(),
            Counter::NumByte => s.len(),
            Counter::Line => s
                .graphemes(true)
                .filter(|grapheme| NEWLINES.contains(grapheme))
                .count(),
            Counter::Words => s.unicode_words().count(),
            Counter::CodePoints => s.chars().count(),
        }
    }
}

/// Different types of counters.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Counter {
    /// Counts lines.
    Line,

    /// Counts words.
    Words,

    /// Counts the total number of bytes.
    NumByte,

    /// Counts grapheme clusters. The input is required to be valid UTF-8.
    GraphemeCluster,

    /// Counts unicode code points
    CodePoints,
}

/// A convenience array of all counter types.
pub const ALL_COUNTERS: [Counter; 5] = [
    Counter::GraphemeCluster,
    Counter::NumByte,
    Counter::Line,
    Counter::Words,
    Counter::CodePoints,
];

/// A convenience array of the default counter types.
pub const DEFAULT_COUNTERS: [Counter; 3] = [Counter::Line, Counter::Words, Counter::NumByte];

impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Counter::GraphemeCluster => "graphemes",
            Counter::NumByte => "bytes",
            Counter::Line => "lines",
            Counter::Words => "words",
            Counter::CodePoints => "codepoints",
        };

        write!(f, "{}", s)
    }
}

/// Counts the given `Counter`s in the given `&str`.
pub fn count<'a, I>(counters: I, s: &str) -> Counted
where
    I: IntoIterator<Item = &'a Counter>,
{
    let counts: Counted = counters.into_iter().map(|c| (*c, c.count(s))).collect();
    debug!("s: {}, counted: {:#?}", s, counts);
    counts
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::counter;
    use crate::constants::*;
    use env_logger;

    #[test]
    fn test_count_hello() {
        let s = "hello";
        let counts = count(&counter::ALL_COUNTERS[..], s);

        let mut correct_counts = BTreeMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 5);
        correct_counts.insert(Counter::Line, 0);
        correct_counts.insert(Counter::NumByte, 5);
        correct_counts.insert(Counter::Words, 1);
        correct_counts.insert(Counter::CodePoints, 5);

        assert_eq!(correct_counts, counts);
    }

    #[test]
    fn test_count_counts_lines() {
        let _ = env_logger::try_init();

        // * \r\n is a single graheme cluster
        // * trailing newlines are counted
        // * NEL is 2 bytes
        // * FF is 1 byte
        // * LS is 3 bytes
        // * PS is 3 bytes
        let mut s = String::from("foo\r\nbar\n\nbaz");
        s += NEL;
        s += "quux";
        s += FF;
        s += LS;
        s += "xi";
        s += PS;
        s += "\n";

        debug!("NEL: {:?}", NEL.as_bytes());
        debug!("FF: {:?}", FF.as_bytes());
        debug!("LS: {:?}", LS.as_bytes());
        debug!("PS: {:?}", PS.as_bytes());

        debug!("s: {}", s);

        for grapheme in s.graphemes(true) {
            debug!("grapheme: {}", grapheme);
        }

        let counts = count(&counter::ALL_COUNTERS[..], &s);

        let mut correct_counts = BTreeMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 23);
        correct_counts.insert(Counter::Line, 8);
        correct_counts.insert(Counter::NumByte, 29);
        correct_counts.insert(Counter::Words, 5);

        // one more than grapheme clusters because of \r\n
        correct_counts.insert(Counter::CodePoints, 24);

        assert_eq!(correct_counts, counts);
    }

    #[test]
    fn test_count_counts_words() {
        let _ = env_logger::try_init();

        let i_can_eat_glass =
            "Μπορῶ νὰ φάω σπασμένα γυαλιὰ χωρὶς νὰ πάθω τίποτα.";
        let s = String::from(i_can_eat_glass);

        //debug!("words: {:?}", i_can_eat_glass.unicode_words().collect::<Vec<&str>>());

        let counts = count(&counter::ALL_COUNTERS[..], &s);

        let mut correct_counts = BTreeMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 50);
        correct_counts.insert(Counter::Line, 0);
        correct_counts.insert(Counter::NumByte, i_can_eat_glass.len());
        correct_counts.insert(Counter::Words, 9);
        correct_counts.insert(Counter::CodePoints, 50);

        assert_eq!(correct_counts, counts);
    }

    #[test]
    fn test_count_counts_codepoints() {
        let _ = env_logger::try_init();

        // these are NOT the same! One is e + ́́ , and one is é, a single codepoint
        let one = "é";
        let two = "é";

        let counters = [Counter::CodePoints];

        let counts = count(&counters[..], &one);

        let mut correct_counts = BTreeMap::new();
        correct_counts.insert(Counter::CodePoints, 1);

        assert_eq!(correct_counts, counts);

        let counts = count(&counters[..], &two);

        let mut correct_counts = BTreeMap::new();
        correct_counts.insert(Counter::CodePoints, 2);

        assert_eq!(correct_counts, counts);
    }
}
