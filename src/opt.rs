use std::collections::BTreeSet;
use std::str::FromStr;

use crate::counter::{self, Counter};

#[derive(StructOpt, Debug)]
#[structopt(about = "Counts things in strings.")]
pub struct Opt {
    /// Counts the grapheme clusters
    #[structopt(short = "c", long = "grapheme-clusters")]
    pub grapheme_clusters: bool,

    /// Counts the number of bytes
    #[structopt(short = "b", long = "bytes")]
    pub bytes: bool,

    /// Counts the number of lines
    #[structopt(short = "l", long = "lines")]
    pub lines: bool,

    /// Counts the number of words
    #[structopt(short = "w", long = "words")]
    pub words: bool,

    /// Counts the number of Unicode code points
    #[structopt(short = "p", long = "code-points")]
    pub codepoints: bool,

    /// Counts everything. (The default counters are: lines, words, bytes)
    #[structopt(short = "a", long = "all")]
    pub all: bool,

    /// Don't print the field names on the first line.
    #[structopt(short = "n", long = "no-header")]
    pub no_header: bool,

    /// Don't print the output with elastic tabstops. Instead, fields will just be
    /// separated with hard tab characters. Use this if you want streaming output,
    /// or if you want the output to be more scriptable.
    #[structopt(short = "e", long = "no-elastic")]
    pub no_elastic: bool,

    /// The counting mode.
    #[structopt(
        short = "m",
        long = "mode",
        default_value = "file",
        help = "The format checker to use. Line mode will count things \
                within lines, and by default, it will not count newline \
                characters. See --count-newlines."
    )]
    #[structopt(raw(possible_values = "&[\"file\", \"line\"]"))]
    pub mode: CountMode,

    /// When in line mode, count newline characters.
    #[structopt(long = "count-newlines")]
    pub count_newlines: bool,

    /// How many "chunks" of the file to operate on in parallel. (As of this
    /// version, "chunks" means lines.) You probably don't need to mess with this.
    /// uwc will wait until it reads this many chunks (or the end of the file)
    /// to start counting. For normal files, you won't notice this, but if
    /// you're piping a slow command into uwc, you may wonder why it doesn't
    /// seem to be counting anything. You can set this value lower for this case.
    #[structopt(long = "chunk-size", default_value = "10000")]
    pub chunk_size: usize,

    /// Sets the input file(s) to use. "-" gets treated as stdin.
    #[structopt(default_value = "-")]
    pub files: Vec<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, StructOpt)]
pub enum CountMode {
    /// Performs counts for every file.
    File,

    /// Performs counts for every line.
    Line,
}

impl FromStr for CountMode {
    type Err = String;

    fn from_str(s: &str) -> Result<CountMode, String> {
        match s {
            "file" | "f" => Ok(CountMode::File),
            "line" | "l" => Ok(CountMode::Line),
            _ => Err(format!("Unknown count mode: {}", s)),
        }
    }
}

impl Opt {
    /// Gets the [`Counter`]s from the CLI options.
    pub fn get_counters(&self) -> BTreeSet<Counter> {
        let mut counters = BTreeSet::new();

        if self.all {
            counters.extend(&counter::ALL_COUNTERS[..]);
            return counters;
        }

        if self.grapheme_clusters {
            counters.insert(Counter::GraphemeCluster);
        }

        if self.bytes {
            counters.insert(Counter::NumByte);
        }

        if self.lines {
            counters.insert(Counter::Line);
        }

        if self.words {
            counters.insert(Counter::Words);
        }

        if self.codepoints {
            counters.insert(Counter::CodePoints);
        }

        // pick some defaults if the user doesn't specify any counters
        if counters.is_empty() {
            counters.extend(&counter::DEFAULT_COUNTERS[..]);
        }

        counters
    }

    /// Determines if the input buffer should count newlines.
    pub fn should_keep_newlines(&self) -> bool {
        match self.mode {
            CountMode::File => true,
            CountMode::Line => self.count_newlines,
        }
    }
}
