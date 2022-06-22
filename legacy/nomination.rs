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

use bitcoin::{OutPoint, Txid};
use rgb::{AttachmentId, ContractId, Genesis, Node, NodeId};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stens::AsciiString;

use crate::asset::Error;
use crate::schema::{self, FieldType, OwnedRightType};

/// Nomination is a set of records keeping asset meta-information related to the
/// names and other aspects of asset representation.
///
/// Nomination stores values for
/// - Asset name
/// - Asset ticker
/// - Ricardian contract
/// - Decimal percision
/// taken from asset genesis and renomination state transitions.
///
/// This is purely data structure; for tracking information about renomination
/// _operation_ (operation of changing asset names and other nomination values)
/// please see [`Renomination`].
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[derive(Clone, Getters, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display("{ticker}")]
#[derive(StrictEncode, StrictDecode)]
pub struct Nomination {
    /// Asset ticker, up to 8 characters
    ticker: AsciiString,

    /// Full asset name
    name: AsciiString,

    /// Text of Ricardian contract
    ricardian_contract: Option<AttachmentId>,

    /// Number of digits after the asset decimal point
    decimal_precision: u8,
}

impl TryFrom<Genesis> for Nomination {
    type Error = Error;

    fn try_from(genesis: Genesis) -> Result<Self, Self::Error> { Nomination::try_from(&genesis) }
}

impl TryFrom<&Genesis> for Nomination {
    type Error = Error;

    fn try_from(genesis: &Genesis) -> Result<Self, Self::Error> {
        if genesis.schema_id() != schema::schema().schema_id() {
            Err(Error::WrongSchemaId)?;
        }
        let genesis_meta = genesis.metadata();

        let renomination = genesis
            .owned_rights_by_type(OwnedRightType::Renomination as u16)
            .map(|assignments| assignments.as_revealed_owned_attachments())
            .transpose()?
            .as_deref()
            .and_then(<[_]>::first)
            .map(|(_, container)| container.id);

        Ok(Nomination {
            ticker: genesis_meta
                .ascii_string(FieldType::Ticker)
                .first()
                .ok_or(Error::UnsatisfiedSchemaRequirement)?
                .clone(),
            name: genesis_meta
                .ascii_string(FieldType::Name)
                .first()
                .ok_or(Error::UnsatisfiedSchemaRequirement)?
                .clone(),
            ricardian_contract: renomination,
            decimal_precision: *genesis_meta
                .u8(FieldType::Precision)
                .first()
                .ok_or(Error::UnsatisfiedSchemaRequirement)?,
        })
    }
}

/// Renomination operation details.
///
/// Renominations
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[derive(Clone, Getters, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display("{no}:{node_id}")]
#[derive(StrictEncode, StrictDecode)]
pub struct Renomination {
    /// Unique primary key; equals to the state transition id that performs
    /// renomination operation
    node_id: NodeId,

    /// Sequential number of the epoch
    ///
    /// NB: There is no zero epoch and the first is an epoch closing genesis
    /// epoch seal
    no: usize,

    /// Contract ID to which this renomination is related to
    contract_id: ContractId,

    /// Indicates transaction output/seal which had an assigned renomination
    /// right and which closing created this renomination.
    closes: OutPoint,

    /// Seal controlling next renomination operation.
    ///
    /// This can be set to `None` in case if further renominations are
    /// prohibited
    seal: Option<OutPoint>,

    /// Witness transaction id, which should be present in the commitment
    /// medium (bitcoin blockchain or state channel) to make the operation
    /// valid
    witness: Txid,

    /// Actual asset nomination metadata
    #[cfg_attr(feature = "serde", serde(flatten))]
    nomination: Nomination,
}
