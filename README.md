# Sogo
Implements the game [Sogo](https://en.wikipedia.org/wiki/Score_Four) (Four in a row 3D) in Rust to write some AIs. Learning some Rust along the way is also nice.
We plan to implement a version where you try to score "Six in a parallelogram" instead.

It would be nice to have some IO to actually play the game as a human player but that isn't a priority.

# Building [![Build Status](https://travis-ci.org/roSievers/sogo.svg?branch=master)](https://travis-ci.org/roSievers/sogo)


We use Cargo as the build system. To trigger a build just use

    > cargo build

The resulting executable can be run via

    > ./target/debug/sogo

To get a better performance, compile and run with

    > cargo build --release
    > ./target/release/sogo

Builds are checked automatically for each commit and pull request.
