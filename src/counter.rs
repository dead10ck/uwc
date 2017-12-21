use std::str;
use std::collections::{HashSet, HashMap};

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
            Counter::Line => {
                s.graphemes(true)
                    .filter(|grapheme| NEWLINES.contains(grapheme))
                    .count()
            }
            Counter::Words => s.unicode_words().count(),
        }
    }
}

/// Different types of counters.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Counter {
    /// Counts grapheme clusters. The input is required to be valid UTF-8.
    GraphemeCluster,

    /// Counts the total number of bytes.
    NumByte,

    /// Counts lines.
    Line,

    /// Counts words.
    Words,
}

/// Counts the given `Counter`s in the given `&str`.
pub fn count(counters: &[Counter], s: &str) -> HashMap<Counter, usize> {
    debug!("counting '{}' with counters: {:#?}", s, counters);

    let counts: HashMap<Counter, usize> = counters.iter().map(|c| (*c, c.count(s))).collect();

    debug!("counted: {:#?}", counts);
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
    use env_logger;
    use super::*;

    #[test]
    fn test_count_hello() {
        let s = "hello";
        let counters = [
            Counter::GraphemeCluster,
            Counter::Line,
            Counter::NumByte,
            Counter::Words,
        ];
        let counts = count(&counters[..], s);

        let mut correct_counts = HashMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 5);
        correct_counts.insert(Counter::Line, 0);
        correct_counts.insert(Counter::NumByte, 5);
        correct_counts.insert(Counter::Words, 1);

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

        let counters = [
            Counter::GraphemeCluster,
            Counter::Line,
            Counter::NumByte,
            Counter::Words,
        ];
        let counts = count(&counters[..], &s);

        let mut correct_counts = HashMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 23);
        correct_counts.insert(Counter::Line, 8);
        correct_counts.insert(Counter::NumByte, 29);
        correct_counts.insert(Counter::Words, 5);

        assert_eq!(correct_counts, counts);
    }

    #[test]
    fn test_count_counts_words() {
        let _ = env_logger::init();

        let i_can_eat_glass = "Μπορῶ νὰ φάω σπασμένα γυαλιὰ χωρὶς νὰ πάθω τίποτα.";
        let s = String::from(i_can_eat_glass);

        //debug!("words: {:?}", i_can_eat_glass.unicode_words().collect::<Vec<&str>>());

        let counters = [
            Counter::GraphemeCluster,
            Counter::Line,
            Counter::NumByte,
            Counter::Words,
        ];

        let counts = count(&counters[..], &s);

        let mut correct_counts = HashMap::new();
        correct_counts.insert(Counter::GraphemeCluster, 50);
        correct_counts.insert(Counter::Line, 0);
        correct_counts.insert(Counter::NumByte, i_can_eat_glass.len());
        correct_counts.insert(Counter::Words, 9);

        assert_eq!(correct_counts, counts);
    }
}
