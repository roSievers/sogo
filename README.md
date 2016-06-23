# Sogo
Implements the game [Sogo](https://en.wikipedia.org/wiki/Score_Four) (Four in a row 3D) in Rust to write some AIs. Learning some Rust along the way is also nice.
We plan to implement a version where you try to score "Six in a parallelogram" instead.

# UI

The 3D interface for human players is written using [Kiss3D](https://github.com/sebcrozet/kiss3d).
I'm using my [personal fork](https://github.com/roSievers/kiss3d) which might be ahead and incompatible.
For development I sometimes switch to a local version which might break builds.

# Building [![Build Status](https://travis-ci.org/roSievers/sogo.svg?branch=master)](https://travis-ci.org/roSievers/sogo)


We use Cargo as the build system. To trigger a build just use

    > cargo build

The resulting executable can be run via

    > ./target/debug/sogo

To get a better performance, compile and run with

    > cargo build --release
    > ./target/release/sogo

Builds are checked automatically for each commit and pull request.

# Code Quality [![Clippy Linting Result](https://clippy.bashy.io/github/rosievers/sogo/master/badge.svg)](https://clippy.bashy.io/github/rosievers/sogo/master/log)
As I am learning Rust on the go, the quality is expected to be low. There are a few tests but the overall coverage is still awful. I'm considering using the Clippy linter to slightly improve code quality.
