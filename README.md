# BASS for Rust

These are high level Rust-ified bindings for the [BASS library](https://un4seen.com/bass.html), built on top of [`bass-sys`](https://github.com/ILikeTeaALot/bass-sys).

This is an on-going effort to cover most of the BASS API, but it's not all done yet, hence why it isn't published on crates.io.

There should be no unsafe functionality exposed; so for some things calling a function from `bass-sys` directly may be necessary.

## Completed/Stable Components

- DSP
- Mixers
- MOD Music
- Samples
- Splitters
- Streams
- Syncs

## Notes

The environment variable DYLD_LIBRARY_PATH is set to include `.` to enable library searching to find BASS at the project root for testing.