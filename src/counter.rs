use std::str;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;

use env_logger;
use unicode_segmentation::UnicodeSegmentation;

const NEL: &'static str = "\u{0085}";
const FF: &'static str = "\u{000C}";
const LS: &'static str = "\u{2028}";
const PS: &'static str = "\u{2029}";

lazy_static! {
    /// New line sequences according to:
    /// http://www.unicode.org/standard/reports/tr13/tr13-5.html
    static ref NEWLINES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("\r");
        s.insert("\n");
        s.insert("\r\n");
        s.insert(NEL);
        s.insert(FF);
        s.insert(LS);
        s.insert(PS);
        s
    };
}

/// Counts things in `&str`s.
//pub trait Count {
///// Counts something inside the given `&str`.
//fn count(&self, counters: &[Counter], s: &str) -> HashMap<Counter, u64>;
//}
/// Different types of counters.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Counter {
    /// Counts grapheme clusters. The input is required to be valid UTF-8.
    GraphemeCluster,

    /// Counts the total number of bytes.
    NumByte,

    /// Counts lines.
    Line,
}

/// Counts the given `Counter`s in the given `&str`.
pub fn count(counters: &[Counter], s: &str) -> HashMap<Counter, u64> {
    let mut counts: HashMap<Counter, u64> = HashMap::new();

    for grapheme in s.graphemes(true) {
        counters.iter().for_each(|c| {
            let count : u64 = match *c {
                Counter::GraphemeCluster => 1,
                Counter::NumByte => grapheme.len() as u64,
                Counter::Line => if NEWLINES.contains(grapheme) { 1 } else { 0 },
            };

            let entry = counts.entry(*c).or_insert(0u64);
            *entry += count;
        })
    }

    // there should always be at least one line
    if let Entry::Occupied(e) = counts.entry(Counter::Line) {
        let count = e.into_mut();
        *count += 1;
    }

    counts
}

pub enum CountMode {
    /// Performs counts for the entire input.
    Whole,

    /// Performs counts for every file.
    File,

    /// Performs counts for every line.
    Line,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_count_hello() {
        let s = "hello";
        let counters = [ Counter::GraphemeCluster, Counter::Line, Counter::NumByte ];
        let counts = count(&counters[..], s);

        let mut correct_counts = HashMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 5);
        correct_counts.insert(Counter::Line, 1);
        correct_counts.insert(Counter::NumByte, 5);

        assert_eq!(correct_counts, counts);
    }

    #[test]
    fn test_count_counts_lines() {
        let _ = env_logger::init();

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

        let counters = [ Counter::GraphemeCluster, Counter::Line, Counter::NumByte ];
        let counts = count(&counters[..], &s);

        let mut correct_counts = HashMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 23);
        correct_counts.insert(Counter::Line, 9);
        correct_counts.insert(Counter::NumByte, 29);

        assert_eq!(correct_counts, counts);
    }
}
