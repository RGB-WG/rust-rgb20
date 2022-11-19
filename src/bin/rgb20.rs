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
extern crate amplify;
#[macro_use]
extern crate clap;
extern crate serde_crate as serde;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

use bitcoin::OutPoint;
use clap::Parser;
use colored::Colorize;
use lnpbp::bech32::Bech32ZipString;
use lnpbp::chain::Chain;
use rgb::fungible::allocation::{AllocatedValue, OutpointValue, UtxobValue};
use rgb::{Consignment, Contract, IntoRevealedSeal, StateTransfer};
use rgb20::{Asset, Rgb20};
use seals::txout::CloseMethod;
use stens::AsciiString;
use strict_encoding::{StrictDecode, StrictEncode};

/// invalid argument name `{0}`
#[derive(Clone, Debug, Display, Error)]
#[display(doc_comments)]
pub struct InvalidName(String);

#[derive(ArgEnum, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum SchemaName {
    LegacyBasic,
    LegacyComplete,
}

impl FromStr for SchemaName {
    type Err = InvalidName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "legacy-basic" => SchemaName::LegacyBasic,
            "legacy-complete" => SchemaName::LegacyComplete,
            wrong => return Err(InvalidName(wrong.to_owned())),
        })
    }
}

#[derive(ArgEnum, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ExportFormat {
    Binary,
    Bech32,
    Base64,
    Json,
    Yaml,
}

impl FromStr for ExportFormat {
    type Err = InvalidName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bin" => ExportFormat::Binary,
            "bech32" => ExportFormat::Bech32,
            "base64" => ExportFormat::Base64,
            "json" => ExportFormat::Json,
            "yaml" => ExportFormat::Yaml,
            wrong => return Err(InvalidName(wrong.to_owned())),
        })
    }
}

#[derive(Parser, Clone, Debug)]
#[clap(
    name = "rgb20",
    bin_name = "rgb20",
    author,
    version,
    about = "Command-line tool for working with RGB20 fungible assets"
)]
pub struct Opts {
    /// Bitcoin network to use
    #[clap(short, long, default_value = "signet", env = "RGB_NETWORK")]
    pub network: Chain,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Command {
    /// Export schema
    Schema {
        /// File to save the schema to. If no file is given, exports to STDOUT.
        file: Option<PathBuf>,

        /// Export format
        #[clap(short, long, default_value = "yaml")]
        format: ExportFormat,

        /// Name of an RGB20 schema to export
        #[clap(short, long, default_value = "legacy-complete")]
        schema: SchemaName,
    },

    /// Issue a new asset
    Issue {
        /// Asset ticker (up to 8 characters, always converted to uppercase)
        #[clap(validator = ticker_validator)]
        ticker: AsciiString,

        /// Asset name (up to 32 characters)
        name: AsciiString,

        /// Precision, i.e. number of digits reserved for fractional part
        #[clap(short, long, default_value = "8")]
        precision: u8,

        /// Asset allocation, in form of <amount>@<txid>:<vout>
        allocation: Vec<OutpointValue>,

        /// Outputs controlling inflation (secondary issue);
        /// in form of <amount>@<txid>:<vout>
        #[clap(short, long)]
        inflation: Vec<OutpointValue>,

        /// Method for seal closing ('tapret1st' or 'opret1st')
        #[clap(short, long, default_value = "tapret1st")]
        method: CloseMethod,
        /// Enable renomination procedure; parameter takes argument in form of
        /// <txid>:<vout> specifying output controlling renomination right
        #[clap(short, long)]
        renomination: Option<OutPoint>,

        /// Enable epoch-based burn & replacement procedure; parameter takes
        /// argument in form of <txid>:<vout> specifying output controlling the
        /// right of opening the first epoch
        #[clap(short, long)]
        epoch: Option<OutPoint>,
    },

    /// Prepares state transition for assets transfer.
    Transfer {
        /// File with state transfer consignment, which endpoints will act as
        /// inputs.
        consignment: PathBuf,

        /// Bitcoin transaction UTXOs which will be spent by the transfer
        #[clap(short = 'u', long = "utxo", required = true)]
        outpoints: Vec<OutPoint>,

        /// List of transfer beneficiaries
        #[clap(required = true)]
        beneficiaries: Vec<UtxobValue>,

        /// Change output; one per schema state type.
        #[clap(short, long)]
        change: Vec<AllocatedValue>,

        /// File to store state transition transferring assets to the
        /// beneficiaries and onto change outputs.
        output: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let opts = Opts::parse();

    match opts.command {
        Command::Schema {
            file,
            format,
            schema,
        } => {
            let mut fd = open_file_or_stdout(file).unwrap();
            let schema = match schema {
                SchemaName::LegacyBasic => rgb20::schema(),
                SchemaName::LegacyComplete => rgb20::subschema(),
            };
            match format {
                ExportFormat::Binary => {
                    schema.strict_encode(&mut fd).unwrap();
                }
                ExportFormat::Bech32 => {
                    let data = schema.strict_serialize().unwrap();
                    fd.write_all(data.bech32_zip_string().as_bytes()).unwrap()
                }
                ExportFormat::Base64 => {
                    let data = schema.strict_serialize().unwrap();
                    fd.write_all(base64::encode(&data).as_bytes()).unwrap()
                }
                ExportFormat::Json => serde_json::to_writer(&mut fd, &schema).unwrap(),
                ExportFormat::Yaml => serde_yaml::to_writer(&mut fd, &schema).unwrap(),
            }
            fd.flush().unwrap();
        }

        Command::Issue {
            ticker,
            name,
            precision,
            allocation,
            inflation,
            method,
            renomination,
            epoch,
        } => {
            let inflation = inflation.into_iter().fold(
                BTreeMap::new(),
                |mut map, OutpointValue { value, outpoint }| {
                    // We may have only a single secondary issuance right per
                    // outpoint, so folding all outpoints
                    map.entry(outpoint)
                        .and_modify(|amount| *amount += value)
                        .or_insert(value);
                    map
                },
            );
            let contract = Contract::create_rgb20(
                opts.network,
                ticker,
                name,
                precision,
                allocation,
                inflation,
                method,
                renomination,
                epoch,
            );

            let _asset =
                Asset::try_from(&contract).expect("create_rgb20 does not match RGB20 schema");

            eprintln!(
                "{} {}\n",
                "Contract ID:".bright_green(),
                contract.contract_id().to_string().bright_yellow()
            );

            eprintln!("{}", "Contract YAML:".bright_green());
            eprintln!("{}", serde_yaml::to_string(contract.genesis()).unwrap());

            eprintln!("{}", "Contract JSON:".bright_green());
            println!("{}\n", serde_json::to_string(contract.genesis()).unwrap());

            eprintln!("{}", "Contract source:".bright_green());
            println!("{}\n", contract);

            // eprintln!("{}", "Asset details:".bright_green());
            // eprintln!("{}\n", serde_yaml::to_string(&asset).unwrap());
        }

        Command::Transfer {
            consignment,
            outpoints,
            beneficiaries,
            change,
            output,
        } => {
            let transfer = StateTransfer::strict_file_load(consignment).unwrap();

            let asset = Asset::try_from(&transfer).unwrap();

            let beneficiaries = beneficiaries
                .into_iter()
                .map(|v| (v.seal_confidential.into(), v.value))
                .collect();
            let change = change
                .into_iter()
                .map(|v| (v.into_revealed_seal(), v.value))
                .collect();
            let outpoints = outpoints.into_iter().collect();
            let transition = asset.transfer(outpoints, beneficiaries, change).unwrap();

            transition.strict_file_save(output).unwrap();
            //consignment.strict_file_save(output).unwrap();

            println!("{}", serde_yaml::to_string(&transition).unwrap());
            println!("{}", "Success".bold().bright_green());
        }
    }

    Ok(())
}

fn ticker_validator(name: &str) -> Result<(), String> {
    if name.len() < 3 || name.len() > 8 || name.chars().any(|c| c < 'A' && c > 'Z') {
        Err(
            "Ticker name must be between 3 and 8 chars, contain no spaces and \
            consist only of capital letters\
            "
            .to_string(),
        )
    } else {
        Ok(())
    }
}

pub fn open_file_or_stdout(
    filename: Option<impl AsRef<Path>>,
) -> Result<Box<dyn Write>, io::Error> {
    Ok(match filename {
        Some(filename) => {
            let file = fs::File::create(filename)?;
            Box::new(file)
        }
        None => Box::new(io::stdout()),
    })
}
