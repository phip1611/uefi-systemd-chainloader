# uefi-systemd-chainloader

Simple example to demonstrate that I can do some fun in UEFI and then jump back
into the normal boot flow of the system.


# Prerequisites (on non-Nix system)

- OVMF in `/usr/share/ovmf/OVMF.fd` or provide the `OVMF` env var
- qemu-system-x86_64
- rustup


# Steps to Run

- On NixOS: `$ nix develop .`
- `$ cargo run --release`
