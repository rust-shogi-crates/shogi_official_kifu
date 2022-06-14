all: sharedlib include c_tests

.PHONY: sharedlib include check-include c_tests

sharedlib:
	cargo +nightly build --release --no-default-features

include: include/shogi_official_kifu.h

include/shogi_official_kifu.h: cbindgen.toml
	./generate_header.sh

check-include:
	./generate_header.sh --verify

c_tests: sharedlib include
	$(MAKE) -C c_tests
