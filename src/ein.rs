#![deny(rust_2018_idioms, unsafe_code)]
use gnostr_git::porcelain;
fn main() -> anyhow::Result<()> {
    porcelain::main()
}

#[cfg(not(feature = "pretty-cli"))]
compile_error!("Please set 'pretty-cli' feature flag");
