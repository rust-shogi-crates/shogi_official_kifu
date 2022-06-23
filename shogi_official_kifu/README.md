# Rust shogi crates: Official notation of moves (`rlib`)
[![crate](https://img.shields.io/crates/v/shogi_official_kifu)](https://crates.io/crates/shogi_official_kifu)
[![docs](https://docs.rs/shogi_official_kifu/badge.svg)](https://docs.rs/shogi_official_kifu)
![Rust Version](https://img.shields.io/badge/rustc-1.60+-blue.svg)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/mit-license.php)

This crate provides functions that convert moves into string representations described in <https://www.shogi.or.jp/faq/kihuhyouki.html>.

## Notations

### Official notation
Examples: `▲２８飛成` (A rook moves to 2h, after which it promotes) 
`飛` means a rook, and `成` means promotion.

### Traditional notation
Examples: `▲２八飛成` (`八` is a Chinese character that represents "8".)

## Available features
- `std`: `std`-related functionalities are made available. Enabled by default.
- `kansuji`: Functions that emit strings in traditional notation are available. Enabled by default.
