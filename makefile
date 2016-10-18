arch ?= x86_64
target ?= $(arch)-mezzo
kernel := build/kernel-$(arch).bin
mezzo := target/$(target)/debug/libmezzo.a
iso := build/os-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_sources := $(wildcard src/arch/$(arch)/*.asm)
assembly_objects := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_sources))

all:: $(kernel)

clean::
	@rm -rf build

debug:: $(iso)
	@qemu-system-x86_64 -d int -no-reboot -cdrom $(iso)

run:: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso:: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2>/dev/null
	@rm -r build/isofiles

$(kernel): cargo $(mezzo) $(assembly_objects) $(linker_script)
	@ld --nmagic --script $(linker_script) --gc-sections -o $(kernel) $(assembly_objects) $(mezzo)

cargo:
	@xargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@
