#!/usr/bin/env bash

set -euo pipefail

BOOT_VOL=".boot-volume"
EFI_FILE=target/x86_64-unknown-none/uefi-systemd-chainloader.efi
OVMF="${OVMF:-'/usr/share/ovmf/OVMF.fd'}"

rm -rf "$BOOT_VOL"
mkdir -p "$BOOT_VOL/EFI/BOOT"
mkdir -p "$BOOT_VOL/EFI/systemd"

if [ -f "$1" ]; then
    EFI_FILE=$1
fi

cp "$EFI_FILE" "$BOOT_VOL/EFI/BOOT/BOOTX64.EFI"
# We pretend we have systemd-boot here to test the chain loading
cp "$EFI_FILE" "$BOOT_VOL/EFI/systemd/systemd-bootx64.efi"

qemu-system-x86_64 \
    -bios $OVMF \
    -cpu qemu64 \
    -debugcon stdio \
    -display gtk \
    -drive "format=raw,file=fat:rw:$BOOT_VOL" \
    -m 512M \
    -machine q35,accel=tcg \
    -monitor vc \
    -no-reboot \
    -nodefaults \
    -smp 4 \
    -vga std
