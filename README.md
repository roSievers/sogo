# Sogo
Implements the game [Sogo](https://en.wikipedia.org/wiki/Score_Four) (Four in a row 3D) in Rust to write some AIs.
I plan to implement a version where you try to score "Six in a parallelogram" instead. It was my first larger project when I started to learn rust.

# Building [![Build Status](https://travis-ci.org/roSievers/sogo.svg?branch=master)](https://travis-ci.org/roSievers/sogo)

    > cargo build --release
    > ./target/release/sogo

# UI

The 3D interface for human players is written using [Kiss3D](https://github.com/sebcrozet/kiss3d).
