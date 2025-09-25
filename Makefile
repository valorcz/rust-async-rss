
.PHONY: all run clean update

all: clean run

run:
	RUST_LOG=debug cargo run

update:
	cargo update

clean:
	cargo clean

