# Must be run in the crate's root directory
cbindgen --config cbindgen.toml --crate shogi_official_kifu --output include/shogi_official_kifu.h $@
