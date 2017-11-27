use counter::Counter;

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

    /// Sets the input file(s) to use. "-" gets treated as stdin.
    #[structopt(default_value = "-")]
    pub files: Vec<String>,
}

impl Opt {
    /// Gets the [`Counter`]s from the CLI options.
    pub fn get_counters(&self) -> Vec<Counter> {
        let mut counters = Vec::new();

        if self.grapheme_clusters {
            counters.push(Counter::GraphemeCluster);
        }

        if self.bytes {
            counters.push(Counter::NumByte);
        }

        if self.lines {
            counters.push(Counter::Line);
        }

        if self.words {
            counters.push(Counter::Words);
        }

        // pick some defaults if the user doesn't specify any counters
        if counters.is_empty() {
            counters.push(Counter::Line);
            counters.push(Counter::Words);
            counters.push(Counter::NumByte);
        }

        counters
    }
}
