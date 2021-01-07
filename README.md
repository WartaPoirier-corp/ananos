# Bananos

A rewrite from scratch of ananOS using UEFI as a basis.

- BIOS is outdated, so if we are going to support either BIOS or UEFI it will be UEFI
- My computer uses UEFI, and I want to boot my "Hello, world" OS on real hardware

Based on os.phill-op.com and [this post](https://gil0mendes.io/blog/an-efi-app-a-bit-rusty/)
as well as various other sources from the internet that I don't remember.

## Try it

```
nix-shell

# required until rustup 1.23 makes it way into nixpkgs:
rustup component add rust-src

cargo run
```
