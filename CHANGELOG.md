# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2019-10-03

* If a file given on input failed to open, it would stop the whole program.
  Instead, just print the error on opening the file and continue trying the
  rest of the input files.

## [1.0.0] - 2019-10-01

* Fix a bug where line mode was showing multiple lines in a single line.
* Change to 1.0, as I think this package is feature complete.

## [0.3.1] - 2019-08-22

* Update dependencies
* Fix typo in `repository` field of the `Cargo.toml`, so the repository link
  now shows up in crates.io.

## [0.3.0] - 2019-08-02

In this release, the minor version was bumped by mistake.

* Update dependencies

## [0.2.0] - 2019-01-28

### Added

* This adds parallelization with rayon. It does this by chunking up the lines
  it reads and doing those in parallel. Local testing found 10,000 to be the
  optimal number, so that is the default. A consequence of this behavior is
  that if the input is slow, it will seem like it is doing nothing because it
  is waiting for a complete chunk before doing any counting. The `--chunk-size`
  option is given for this situation.
