# Toolchain

The initial build uses stable Rust 1.99.0 (MSVC x64), edition 2024, with the installed Visual Studio 18 C++ Build Tools. Core requirements are Rust 1.90 or newer. Node 24 is the intended frontend runtime; pnpm and the frontend dependency set will be compatibility-checked together before installation.

SQLite is embedded through rusqlite 0.41. The native FSRS dependency will be pinned to the approved upstream commit. Rand 0.9 is selected to match that engine's supported dependency line. Cargo.lock records exact resolved versions. Users do not need a separately installed Rust, Node or database runtime to run release installers.
