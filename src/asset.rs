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

use rgb::ContractState;

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
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(crate = "serde_crate"))]
pub struct Asset(ContractState);

impl Asset {}
