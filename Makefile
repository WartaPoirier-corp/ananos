OVMF_DIR=/nix/store/4n9rlkldjys08rgd8xq6rm1zmrgn5h5q-OVMF-202011-fd
OVMF_FW=$(OVMF_DIR)/FV/OVMF_CODE.fd
OVMF_VARS=$(OVMF_DIR)/FV/OVMF_VARS.fd
PROJECT=bananos
USB_DEV=/dev/sdc

# Inspired by
# - https://github.com/IsaacWoods/pebble/blob/master/Makefile
# - https://wiki.osdev.org/UEFI

all: run

build:
	cargo build

efi_dir: build
	mkdir -p build/fat/EFI/BOOT
	cp target/x86_64-unknown-uefi/debug/$(PROJECT).efi build/fat/EFI/BOOT/BootX64.efi
	echo '\EFI\BOOT\BOOTX64.EFI' > build/fat/startup.nsh
	cp -f $(OVMF_FW) build
	cp -f $(OVMF_VARS) build

run: efi_dir
	qemu-system-x86_64 \
		--enable-kvm \
		-nodefaults \
		-vga std \
		-machine q35,accel=kvm:tcg \
		-m 128M \
		-monitor vc:1024x768 \
		-serial stdio \
		-drive if=pflash,format=raw,readonly,file=./build/OVMF_CODE.fd \
		-drive if=pflash,format=raw,file=./build/OVMF_VARS2.fd \
		-drive format=raw,file=fat:rw:./build/fat

usb: efi_dir
	# Create a GPT table on the USB key
	sudo parted $(USB_DEV) -s -a minimal mklabel gpt
	sudo parted $(USB_DEV) -s -a minimal mkpart EFI FAT16 2048s 93716s
	sudo parted $(USB_DEV) -s -a minimal toggle 1 boot
	# Create a FAT16 partition
	dd if=/dev/zero of=build/fat.img bs=512 count=91669
	mkfs.vfat -F 32 build/fat.img -n BOOT
	# Copy the files
	mcopy -i build/fat.img -s build/fat/* ::
	# Copy the partition to the USB key
	sudo dd if=build/fat.img of=$(USB_DEV) bs=512 count=91669 seek=2048 conv=notrunc
	# remove the temporary disk image
	rm build/fat.img

.PHONY: build
