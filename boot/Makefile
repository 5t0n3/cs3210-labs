ROOT := $(shell git rev-parse --show-toplevel)

BOOT := boot
TTY_PATH := /dev/ttyUSB0

CARGO_FLAGS = -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

.PHONY: all build qemu transmit objdump nm check clean install test

all: build

build:
	@mkdir -p build
	@echo "+ Building build/$(BOOT).bin [build/debug]"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O elf64-aarch64 build/$(BOOT).elf
	@echo "+ Making kernel image (build/$(BOOT).bin)"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O binary build/$(BOOT).bin

qemu: build
	./qemu.sh build/$(BOOT).bin

transmit: build
	@echo "+ Transmitting build/$(BOOT).bin to $(TTY_PATH)"
	ttywrite -i build/$(BOOT).bin $(TTY_PATH)

objdump: build
	cargo objdump -- --disassemble --no-show-raw-insn --print-imm-hex

nm: build
	cargo nm build/$(BOOT).elf

clean:
	cargo clean
	rm -rf build

install: build
	@echo "+ Installing build/$(BOOT).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(BOOT).elf

test:
	cargo test --target=$(HOST_TARGET)