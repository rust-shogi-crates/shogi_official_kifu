all: sharedlib include c_tests

.PHONY: sharedlib include c_tests

sharedlib:
	cargo +nightly build --release --no-default-features --features alloc

include: include/shogi_usi_parser.h

include/shogi_usi_parser.h: cbindgen.toml
	./generate_header.sh

c_tests: sharedlib include
	$(MAKE) -C c_tests
