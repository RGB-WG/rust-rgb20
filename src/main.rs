// RGB20 Library: high-level API to RGB fungible assets.
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all copyright
// and related and neighboring rights to this software to the public domain
// worldwide. This software is distributed without any warranty.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
extern crate clap;
extern crate serde_crate as serde;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(
    name = "rgb20",
    bin_name = "rgb20",
    author,
    version,
    about = "Command-line tool for working with RGB20 fungible assets"
)]
pub struct Opts {
    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Command {}

fn main() -> Result<(), String> {
    let opts = Opts::parse();

    match opts.command {}

    Ok(())
}
