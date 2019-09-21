use std::collections::HashSet;

use lazy_static::*;

#[rustfmt::skip] pub(crate) const LF:   &'static str = "\n";       // 0x00000a
#[rustfmt::skip] pub(crate) const CR:   &'static str = "\r";       // 0x00000d
#[rustfmt::skip] pub(crate) const CRLF: &'static str = "\r\n";     // 0x000d0a
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
}

