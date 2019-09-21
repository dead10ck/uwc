use std::collections::HashSet;

use lazy_static::*;
use regex::bytes::Regex;

#[rustfmt::skip] pub(crate) const LF:   &'static str = "\n";       // 0xe0000a
#[rustfmt::skip] pub(crate) const CR:   &'static str = "\r";       // 0xe0000d
#[rustfmt::skip] pub(crate) const CRLF: &'static str = "\r\n";     // 0xe00d0a
#[rustfmt::skip] pub(crate) const NEL:  &'static str = "\u{0085}"; // 0x00c285
#[rustfmt::skip] pub(crate) const FF:   &'static str = "\u{000C}"; // 0x00000c
#[rustfmt::skip] pub(crate) const LS:   &'static str = "\u{2028}"; // 0xe280a8
#[rustfmt::skip] pub(crate) const PS:   &'static str = "\u{2029}"; // 0xe280a9

lazy_static! {
    /// New line sequences according to:
    /// http://www.unicode.org/standard/reports/tr13/tr13-5.html
    pub(crate) static ref NEWLINES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(CR);
        s.insert(LF);
        s.insert(CRLF);
        s.insert(NEL);
        s.insert(FF);
        s.insert(LS);
        s.insert(PS);
        s
    };

    pub(crate) static ref NEWLINE_PATTERN : Regex = {
        // need to specify this order so CRLF is preferred over
        // CR and LF on their own
        let pattern = &[ CRLF, LF, CR, NEL, FF, LS, PS ].join("|");
        Regex::new(&pattern).unwrap()
    };
}
