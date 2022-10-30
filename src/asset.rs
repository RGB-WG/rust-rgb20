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

use std::collections::btree_set;

use bitcoin::OutPoint;
use chrono::{Date, Utc};
use rgb::{
    ConsignmentType, ContractId, ContractState, InmemConsignment, NodeId, OwnedValue, Schema,
    SchemaId,
};

use crate::Rgb20Schemata;

/// Type of the subschema under which asset is created
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum Subschema {
    /// Simple asset schema. Assets to not support inflation and renomination.
    Simple,

    /// Inflationary schema: asset can have secondary issuance, can be burned
    /// or replaced - but can not be renominated.
    Inflationary,

    /// Root RGB20 schema supporting all asset operations
    Full,
}

impl Subschema {
    pub fn bech32(self) -> &'static str {
        match self {
            Subschema::Simple => Schema::RGB20_SIMPLE_BECH32,
            Subschema::Inflationary => Schema::RGB20_INFLATIONARY_BECH32,
            Subschema::Full => Schema::RGB20_ROOT_BECH32,
        }
    }
}

/// RGB20 asset information.
///
/// Structure presents complete set of RGB20 asset-related data which can be
/// extracted from the genesis or a consignment. It is not the source of the
/// truth, and the presence of the data in the structure does not imply their
/// validity, since the structure constructor does not validates blockchain or
/// LN-based transaction commitments or satisfaction of schema requirements.
///
/// The main reason of the structure is:
/// 1) to persist *cached* copy of the asset data without the requirement to
///    parse all stash transition each time in order to extract allocation
///    information;
/// 2) to present data from asset genesis or consignment for UI in convenient
///    form.
/// 3) to orchestrate generation of new state transitions taking into account
///    known asset information.
///
/// (1) is important for wallets, (2) is for more generic software, like
/// client-side-validated data explorers, developer & debugging tools etc and
/// (3) for asset-management software.
///
/// In both (2) and (3) case there is no need to persist the structure; genesis
/// /consignment should be persisted instead and the structure must be
/// reconstructed each time from that data upon the launch
#[derive(Getters, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub struct Asset {
    #[getter(as_copy)]
    id: ContractId,

    #[getter(as_copy)]
    subschema: Subschema,

    #[getter(as_copy)]
    issued: Date<Utc>,

    ticker: String,
    name: String,
    contract: String,
    #[getter(as_copy)]
    precision: u8,

    known_allocations: Vec<(OutPoint, u64)>,
    known_supply: u64,
    #[getter(as_copy)]
    max_supply: u64,
    #[getter(as_copy)]
    is_total_supply_known: bool,
    #[getter(as_copy)]
    burned_supply: u64,
    #[getter(as_copy)]
    replaced_supply: u64,

    #[getter(as_copy)]
    can_be_renominated: bool,
    #[getter(as_copy)]
    can_be_inflated: bool,
    #[getter(as_copy)]
    can_be_burned: bool,
    #[getter(as_copy)]
    can_be_replaced: bool,
}

impl Asset {
    /// Lists all known allocations for the given bitcoin transaction
    /// [`OutPoint`]
    pub fn known_coins(&self) -> btree_set::Iter<OwnedValue> { self.0.owned_values.iter() }

    /// Lists all known allocations for the given bitcoin transaction
    /// [`OutPoint`]
    pub fn outpoint_coins(&self, outpoint: OutPoint) -> Vec<OwnedValue> {
        self.known_coins()
            .filter(|a| a.seal == outpoint)
            .cloned()
            .collect()
    }
}

impl<T> TryFrom<&InmemConsignment<T>> for Asset
where T: ConsignmentType
{
    type Error = Error;

    fn try_from(consignment: &InmemConsignment<T>) -> Result<Self, Self::Error> {
        let state = ContractState::from(consignment);
        let asset = Asset(state);
        asset.validate()?;
        Ok(asset)
    }
}

impl Asset {
    fn validate(&self) -> Result<(), Error> {
        if self.0.schema_id != Schema::rgb20_root().schema_id() {
            Err(Error::WrongSchemaId)?;
        }
        // TODO: Validate the state
        Ok(())
    }
}

/// Errors generated during RGB20 asset information parsing from the underlying
/// genesis or consignment data
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// genesis schema id does not match any of RGB20 schemata
    WrongSchemaId,

    /// genesis defines a seal referencing witness transaction while there
    /// can't be a witness transaction for genesis
    GenesisSeal,

    /// epoch seal definition for node {0} contains confidential data
    EpochSealConfidential(NodeId),

    /// nurn & replace seal definition for node {0} contains confidential data
    BurnSealConfidential(NodeId),

    /// inflation assignment (seal or state) for node {0} contains confidential
    /// data
    InflationAssignmentConfidential(NodeId),

    /// not of all epochs referenced in burn or burn & replace operation
    /// history are known from the consignment
    NotAllEpochsExposed,
}
