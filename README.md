# uwc

[![crates.io page](https://img.shields.io/crates/v/uwc.svg)](https://crates.io/crates/uwc)

Like `wc`, but unicode-aware, and with line mode.

`uwc` can count:

* Lines
* Words
* Bytes
* Grapheme clusters
* Unicode code points

Additionally, it can operate in *line mode*, which will count things *within* lines.

## Usage example

By default, `uwc` will count lines, words, and bytes. You can specify the counters
you'd like, or ask for all counters with the `-a` flag.

```sh
$ uwc tests/fixtures/**/input
lines  words  bytes  filename
8      5      29     tests/fixtures/all_newlines/input
0      0      0      tests/fixtures/empty/input
0      0      0      tests/fixtures/empty_line_mode/input
1      9      97     tests/fixtures/flags_bp/input
1      9      97     tests/fixtures/flags_cl/input
1      9      97     tests/fixtures/flags_w/input
0      1      5      tests/fixtures/hello/input
1      9      97     tests/fixtures/i_can_eat_glass/input
8      8      29     tests/fixtures/line_mode/input
7      8      28     tests/fixtures/line_mode_no_trailing_newline/input
7      8      28     tests/fixtures/line_mode_no_trailing_newline_count_newlines/input
34     66     507    total

$ uwc -a tests/fixtures/**/input
lines  words  bytes  graphemes  codepoints  filename
8      5      29     23         24          tests/fixtures/all_newlines/input
0      0      0      0          0           tests/fixtures/empty/input
0      0      0      0          0           tests/fixtures/empty_line_mode/input
1      9      97     51         51          tests/fixtures/flags_bp/input
1      9      97     51         51          tests/fixtures/flags_cl/input
1      9      97     51         51          tests/fixtures/flags_w/input
0      1      5      5          5           tests/fixtures/hello/input
1      9      97     51         51          tests/fixtures/i_can_eat_glass/input
8      8      29     28         28          tests/fixtures/line_mode/input
7      8      28     27         27          tests/fixtures/line_mode_no_trailing_newline/input
7      8      28     27         27          tests/fixtures/line_mode_no_trailing_newline_count_newlines/input
34     66     507    314        315         total
```

You can also switch into line mode with the `--mode` flag:

```sh
$ uwc -a --mode line tests/fixtures/line_mode/input
lines  words  bytes  graphemes  codepoints  filename
0      1      1      1          1           tests/fixtures/line_mode/input:1
0      1      2      2          2           tests/fixtures/line_mode/input:2
0      1      3      3          3           tests/fixtures/line_mode/input:3
0      1      5      4          4           tests/fixtures/line_mode/input:4
0      1      1      1          1           tests/fixtures/line_mode/input:5
0      1      4      4          4           tests/fixtures/line_mode/input:6
0      1      2      2          2           tests/fixtures/line_mode/input:7
0      1      3      3          3           tests/fixtures/line_mode/input:8
0      8      21     20         20          tests/fixtures/line_mode/input:total
```

## Why?

The goal of this project is to consider unicode rules correctly when counting
things. Specifically, it should:

* Count all newline characters correctly. This includes lesser-known line breaks,
  like NEL&#160;(U+0085), FF&#160;(U+000C), LS&#160;(U+2028), and PS&#160;(U+2029).
* Count all words using the Unicode standard's word boundary rules.
* Count all complete grapheme clusters correctly, so that even edge cases like
   Z҉͈͓͈͎a̘͈̠̭l̨̯g̶̬͇̭o̝̹̗͎̙ ͟t͖̙̟̹͇̥̝͡e̥͘x͚̺̭̻͘t͉͔̩̲̘, for example, are counted correctly.

It does *not* aim to implement these unicode algorithms, however, so it makes use of
the [`unicode-segmentation`](https://crates.io/crates/unicode-segmentation) library
for most of the heavy lifting. And since Unicode support in the Rust ecosystem is
not quite mature yet, that has some consequences for this project. See the
caveats below.

## Installation

It is published on crates.io, so simply:

```sh
$ cargo install uwc
```

## Caveats

### UTF-8

It only supports UTF-8 files. UTF-16 can go on my to-do list if there is demand.
For now, you can use `iconv` to convert non-UTF-8 files first.

### Memory usage

The current implementation will always read complete lines before proceeding to
do its counts; without hand-rolling my own streaming implementation of the
Unicode line splitting algorithm, this is necessary for correctness with line
mode. The consequence of this is that if you give it files with very large
lines, it will use memory proportional to the size of the lines. If you give it
a file with no newline sequences, it will soak up the whole file into memory.
Beware.

### Speed

It is slower than `wc`. My analysis hasn't been extensive, but as far as I can
tell, the reasons are:

* It is using unicode algorithms, which are just going to be slower than
  ASCII no matter what.
* I am not that experienced with Rust, so it's quite possible I'm not doing
  something as efficiently as possible.
* My free time is limited, and I am prioritizing correctness over speed
  (though speed is good).

With that said, it is parallelized, which helps. With testing on my local
laptop with larger data sets, the speed is within an order of magnitude of
`wc`. I measured `uwc` being 1.5x slower than `wc` on a collection of 18 MiB of
text files.

### Localization

Rust, as yet, has no localization libraries, so this has some consequences. Some
counts will just be wrong, such as hyphenated words, which is locale-specific
and requires language dictionary lookups to be correct. Also, there are some
languages that have no syntactic word separators, such as Japanese, so e.g.

**私**は**ガラス**を**食べられます**。

should be 5 words, but without localization, we cannot determine that.
