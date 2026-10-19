//! daw-helper's integration tests, one module per file, linked as one test binary
//! (M17 step 0, D185): one link instead of one per file. Tests that must run in a
//! process of their own (a counting global allocator, the environment set before
//! anything reads it) are binaries of their own beside this one.

mod test_plugin;
mod test_vst3;
mod third_party_vst3;
