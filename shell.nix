let pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  buildInputs = with pkgs; [ rustup OVMF qemu gnumake glibc clang_11 gcc10 parted mtools ];
}
