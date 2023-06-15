ROOT := $(shell git rev-parse --show-toplevel)

KERN := blinky

# these have to be provided as flags since otherwise `cargo test` breaks
# (I believe this issue is related: https://github.com/rust-lang/cargo/issues/7359)
CARGO_FLAGS = -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

.PHONY: all debug release qemu objdump clean install test

all: release

# builds have to be done separately since cargo-objcopy just ignores the flags for some reason :(
debug:
	@mkdir -p build
	@echo "+ Building build/$(KERN).elf [build/debug]"
	@cargo build $(CARGO_FLAGS)
	@cargo objcopy -- --strip-all -O binary build/$(KERN).elf

release:
	@mkdir -p build
	@echo "+ Building build/$(KERN).elf [build/release]"
	@cargo build --release $(CARGO_FLAGS)
	@cargo objcopy --release -- --strip-all -O binary build/$(KERN).elf

qemu:
	./qemu.sh build/$(KERN).elf -d in_asm

objdump:
	cargo objdump -- --disassemble --no-show-raw-insn --print-imm-hex

clean:
	cargo clean
	rm -rf build

install:
	@echo "+ Installing build/$(KERN).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(KERN).elf

test:
	cargo test --target=$(HOST_TARGET)