ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
SDCARD := $(ROOT)/ext/fat32-imgs/mock1.fat32.img
TTY_PATH := /dev/ttyUSB0
QEMU_ARGS ?=

CARGO_FLAGS = -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

.PHONY: all build qemu transmit objdump nm check clean install test

all: build

build:
	@mkdir -p build
	@echo "+ Building build/$(KERN).bin [build/debug]"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O elf64-aarch64 build/$(KERN).elf
	@echo "+ Making kernel image (build/$(KERN).bin)"
	@cargo objcopy $(CARGO_FLAGS) --release -- --strip-all -O binary build/$(KERN).bin

qemu: build
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd $(QEMU_ARGS)

qemu-gdb: build
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd -s -S

qemu-asm: build
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd -d in_asm

transmit: build
	@echo "+ Transmitting build/$(KERN).bin to $(TTY_PATH)"
	ttywrite -i build/$(KERN).bin $(TTY_PATH)
	screen $(TTY_PATH) 115200

objdump: build
	cargo objdump -- --disassemble --no-show-raw-insn --print-imm-hex

nm: build
	cargo nm build/$(KERN).elf

clean:
	cargo clean
	rm -rf build

install: build
	@echo "+ Installing build/$(KERN).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(KERN).elf

test:
	cargo test --target=$(HOST_TARGET)