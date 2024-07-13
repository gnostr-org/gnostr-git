{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  buildInputs = [ autoreconfHook cargo cmake gcc gdb git python3 secp256k1 vim ];
}
