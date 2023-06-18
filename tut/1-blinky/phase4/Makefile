ROOT := $(shell git rev-parse --show-toplevel)

KERN := blinky

# these have to be provided as flags since otherwise `cargo test` breaks
# (I believe this issue is related: https://github.com/rust-lang/cargo/issues/7359)
CARGO_FLAGS = -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

.PHONY: all debug release qemu objdump clean install test

all: release

# builds have to be done separately since cargo-objcopy just ignores the flags for some reason :(
# UPDATE: this is fixed as of commit 32df067c, but it's not part of a release yet :(
# (see https://github.com/rust-embedded/cargo-binutils/commit/32df067ce6a18872618c686c0760ff267870eb2c)
debug:
	@mkdir -p build
	@echo "+ Building build/$(KERN).bin [build/debug]"
	@cargo objcopy $(CARGO_FLAGS) -- --strip-all -O elf64-aarch64 build/$(KERN).elf
	@echo "+ Making kernel image (build/$(KERN).bin)"
	@cargo objcopy $(CARGO_FLAGS) -- --strip-all -O binary build/$(KERN).bin

release:
	@mkdir -p build
	@echo "+ Building build/$(KERN).elf [build/release]"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O elf64-aarch64 build/$(KERN).elf
	@echo "+ Making kernel image (build/$(KERN).bin)"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O binary build/$(KERN).bin

qemu:
	./qemu.sh build/$(KERN).elf -d in_asm

objdump:
	cargo objdump -- --disassemble --no-show-raw-insn --print-imm-hex

clean:
	cargo clean
	rm -rf build

install:
	@echo "+ Installing build/$(KERN).bin [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(KERN).bin

test:
	cargo test --target=$(HOST_TARGET)