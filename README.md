# Efisnake

Efisnake is a snake game running entirely in UEFI. This allows it to be run on any machine that supports UEFI booting without requiring an operating system to be installed.

## Demo

This is Efisnake running in QEMU:

https://github.com/user-attachments/assets/79fd1adf-1e80-438e-8031-8c101984ad9b

## Installation

Clone the repository:

```bash
git clone https://github.com/simon0302010/efisnake.git
cd efisnake
```

### Building

Now build the project for your UEFI target:

For x86_64 target, use:

```bash
cargo build --target x86_64-unknown-uefi --release
```

For aarch64 target, use:

```bash
cargo build --target aarch64-unknown-uefi --release
```

For 32-bit x86 target, use:

```bash
cargo build --target i686-unknown-uefi --release
```

You should now have the binary located at `target/<target>/release/efisnake.efi`.

### Preparing a Bootable USB Drive

1. Format a USB drive to FAT32.
2. Create a directory structure on the USB drive as follows: `EFI/BOOT/`.
3. Copy the built `efisnake.efi` file to the `EFI/BOOT/` directory and rename it to `BOOTX64.EFI` for x86_64, `BOOTAA64.EFI` for aarch64, or `BOOTIA32.EFI` for i686.

## Running

1. Insert the USB drive into the target machine.
2. Boot the machine and enter the UEFI boot menu (usually by pressing a key like F12, F10 or F8 during startup).
3. Select the USB drive to boot from it.
4. The Efisnake game should start automatically.

### Controls

Use the arrow keys to control the snake. Press `q` to exit the game. The snake dies if it runs into the walls or itself. If the snake dies, you can restart the game by pressing space.
