# OS 

An experimental OS. The goals are:

- to learn how computer works on a lower level ;
- if we get far enough without being discouraged, to experiment with new ideas for operating systems (see `specification.md` for a few ideas) ;

The goal is not to make a real OS that can run in production.

Most of the code (if not all of it) is written in Rust, following the [*Write an OS in Rust*](https://os.phil-opp.com/) tutorial.

## How to run

First, clone this repository, and `cd` into it.

To build, a few tools are needed:

- `rustup`, to install Rust, Cargo, etc. Instructions can be found [here](https://rustup.rs/).
- `cargo-xbuild` to build the `core` module for our custom bare-metal target (`cargo install cargo-xbuild`)
- The `rust-src` rustup component (`rustup component add rust-src`)
- `bootimage` to make an ISO image of our Kernel (`cargo install bootimage`)
- The `llvm-tools-preview` rustup component that is required by `bootimage` (`rustup component add llvm-tools-preview`)
- QEMU, to run the OS in a virtual machine (`sudo apt install qemu-system-x86` on Debian and Co.)

Now, type `cargo xrun`. Cargo should build the OS, and start QEMU once it is done.
