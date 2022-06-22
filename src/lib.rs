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

#![recursion_limit = "256"]
// Coding conventions
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    missing_docs
)]

//! RGB20 library for working with fungible asset types, operating under
//! schemata, defined with LNPBP-20 standard:
//! - Root RGB20 schema, returned by [`schema::schema()`] with id
//!   [`SCHEMA_ID_BECH32`]
//! - RGB20 subschema, returned by [`schema::subschema()`], prohibiting asset
//!   replacement procedure and having id [`SUBSCHEMA_ID_BECH32`]
//! - High-level RGB20 API performing asset issuance, transfers and other
//!   asset-management operations

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate rgb;
#[macro_use]
extern crate stens;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
extern crate serde_with;

pub mod schema;
mod create;
mod asset;
mod transitions;

pub use asset::{Asset, Error};
pub use create::Rgb20;
pub use schema::{schema, subschema, SCHEMA_ID_BECH32, SUBSCHEMA_ID_BECH32};
