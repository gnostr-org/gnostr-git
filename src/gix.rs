#![deny(unsafe_code, rust_2018_idioms)]
use gnostr_git::plumbing;
#[cfg(feature = "pretty-cli")]
fn main() -> anyhow::Result<()> {
    plumbing::main()
}

#[cfg(not(feature = "pretty-cli"))]
compile_error!("Please set 'pretty-cli' feature flag");
