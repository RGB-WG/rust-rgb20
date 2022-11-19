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

//! RGB20 schemata defining fungible asset smart contract prototypes.

use std::str::FromStr;

use rgb::schema::{
    DiscreteFiniteFieldFormat, GenesisSchema, Occurrences, Schema, SchemaId, StateSchema,
    TransitionSchema,
};
use rgb::script::OverrideRules;
use rgb::vm::embedded::constants::*;
use rgb::ValidationScript;
use stens::{PrimitiveType, StructField, TypeRef, TypeSystem};

/// Field types for RGB20 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum FieldType {
    /// Asset ticker
    ///
    /// Used within context of genesis or renomination state transition
    Ticker = FIELD_TYPE_TICKER,

    /// Asset name
    ///
    /// Used within context of genesis or renomination state transition
    Name = FIELD_TYPE_NAME,

    /// Decimal precision
    Precision = FIELD_TYPE_PRECISION,

    /// Ricardian contract for the asset
    Contract = FIELD_TYPE_CONTRACT_TEXT,

    /// Supply issued with the genesis, secondary issuance or burn & replace
    /// state transition
    IssuedSupply = FIELD_TYPE_ISSUED_SUPPLY,

    /// Supply burned with the burn or burn & replace state transition
    BurnedSupply = FIELD_TYPE_BURN_SUPPLY,

    /// Timestamp for genesis
    Timestamp = FIELD_TYPE_TIMESTAMP,

    /// UTXO containing the burned asset
    BurnUtxo = FIELD_TYPE_BURN_UTXO,

    /// Proofs of the burned supply
    HistoryProof = FIELD_TYPE_HISTORY_PROOF,
}

impl From<FieldType> for rgb::schema::FieldType {
    #[inline]
    fn from(ft: FieldType) -> Self { ft as rgb::schema::FieldType }
}

/// Owned right types used by RGB20 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum OwnedRightType {
    /// Inflation control right (secondary issuance right)
    Inflation = STATE_TYPE_INFLATION_RIGHT,

    /// Asset ownership right
    Assets = STATE_TYPE_OWNERSHIP_RIGHT,

    /// Right to open a new burn & replace epoch
    OpenEpoch = STATE_TYPE_ISSUE_EPOCH_RIGHT,

    /// Right to perform burn or burn & replace operation
    BurnReplace = STATE_TYPE_ISSUE_REPLACEMENT_RIGHT,

    /// Right to perform asset renomination
    Renomination = STATE_TYPE_RENOMINATION_RIGHT,
}

impl From<OwnedRightType> for rgb::schema::OwnedRightType {
    #[inline]
    fn from(t: OwnedRightType) -> Self { t as rgb::schema::OwnedRightType }
}

/// State transition types defined by RGB20 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionType {
    /// Secondary issuance
    Issue = TRANSITION_TYPE_ISSUE_FUNGIBLE,

    /// Asset transfer
    Transfer = TRANSITION_TYPE_VALUE_TRANSFER,

    /// Opening of the new burn & replace asset epoch
    Epoch = TRANSITION_TYPE_ISSUE_EPOCH,

    /// Asset burn operation
    Burn = TRANSITION_TYPE_ISSUE_BURN,

    /// Burning and replacement (re-issuance) of the asset
    BurnAndReplace = TRANSITION_TYPE_ISSUE_REPLACE,

    /// Renomination (change in the asset name, ticker, contract text of
    /// decimal precision).
    Renomination = TRANSITION_TYPE_RENOMINATION,

    /// Operation splitting rights assigned to the same UTXO
    RightsSplit = TRANSITION_TYPE_RIGHTS_SPLIT,
}

impl From<TransitionType> for rgb::schema::TransitionType {
    #[inline]
    fn from(t: TransitionType) -> Self { t as rgb::schema::TransitionType }
}

fn type_system() -> TypeSystem {
    type_system! {
        "OutPoint" :: {
            StructField::with("Txid"),
            StructField::primitive(PrimitiveType::U16),
        },
        "Txid" :: { StructField::array(PrimitiveType::U8, 32) },
        "HistoryProof" :: {
            // Format of the proof defined as an ASCII string
            StructField::ascii_string(),
            // Data for the proof
            StructField::bytes()
        }
    }
}

fn burn() -> TransitionSchema {
    use Occurrences::*;

    TransitionSchema {
        metadata: type_map! {
            FieldType::BurnedSupply => Once,
            // Normally issuer should aggregate burned assets into a
            // single UTXO; however if burn happens as a result of
            // mistake this will be impossible, so we allow to have
            // multiple burned UTXOs as a part of a single operation
            FieldType::BurnUtxo => OnceOrMore,
            FieldType::HistoryProof => NoneOrMore
        },
        closes: type_map! {
            OwnedRightType::BurnReplace => Once
        },
        owned_rights: type_map! {
            OwnedRightType::BurnReplace => NoneOrOnce
        },
        public_rights: none!(),
    }
}

fn renomination() -> TransitionSchema {
    use Occurrences::*;

    TransitionSchema {
        metadata: type_map! {
            FieldType::Ticker => NoneOrOnce,
            FieldType::Name => NoneOrOnce,
            FieldType::Contract => NoneOrOnce,
            FieldType::Precision => NoneOrOnce
        },
        closes: type_map! {
            OwnedRightType::Renomination => Once
        },
        owned_rights: type_map! {
            OwnedRightType::Renomination => NoneOrOnce
        },
        public_rights: none!(),
    }
}

/// Trait extension adding RGB20 schema constructors to the RGB Core Lib
/// [`Schema`] object.
pub trait Rgb20Schemata {
    /// Schema identifier for RGB20 fungible asset supporting all possible asset
    /// operations.
    const RGB20_ROOT_BECH32: &'static str =
        "rgbsh1hacf8gg863veu292hdnttynqzk5xdvyk5q2fxep3e85j4ttzd05s2j4ern";

    /// Schema identifier for RGB20 fungible asset allowing only inflation operation.
    const RGB20_INFLATIONARY_BECH32: &'static str =
        "rgbsh1qmts2pmfxt9e6tpuevk2v0dza30d9v4n0cq6vtm0jtppnyz5xrss4gj9wd";

    /// Schema identifier for RGB20 fungible asset allowing just asset transfers.
    const RGB20_SIMPLE_BECH32: &'static str =
        "rgbsh13c3e8ywrmsu9j0k3er0lgzp9memn5c55rw5svf0l9n3sfntv76zqehteur";

    /// Builds & returns complete RGB20 schema (root schema object)
    fn rgb20_root() -> Schema;

    /// RGB20 subschema which allows only inflation
    fn rgb20_inflationary() -> Schema;

    /// RGB20 subschema which allows simple asset transfers and no asset
    /// modifications (renomination, inflation, burn & replace procedures).
    fn rgb20_simple() -> Schema;
}

impl Rgb20Schemata for Schema {
    fn rgb20_root() -> Schema {
        use Occurrences::*;

        Schema {
            rgb_features: none!(),
            root_id: none!(),
            genesis: GenesisSchema {
                metadata: type_map! {
                    FieldType::Ticker => Once,
                    FieldType::Name => Once,
                    FieldType::Contract => NoneOrOnce,
                    FieldType::Precision => Once,
                    FieldType::Timestamp => Once,
                    // We need this field in order to be able to verify pedersen
                    // commitments
                    FieldType::IssuedSupply => Once
                },
                owned_rights: type_map! {
                    OwnedRightType::Inflation => NoneOrMore,
                    OwnedRightType::OpenEpoch => NoneOrOnce,
                    OwnedRightType::Assets => NoneOrMore,
                    OwnedRightType::Renomination => NoneOrOnce
                },
                public_rights: none!(),
            },
            type_system: type_system(),
            extensions: none!(),
            transitions: type_map! {
                TransitionType::Issue => TransitionSchema {
                    metadata: type_map! {
                        // We need this field in order to be able to verify pedersen
                        // commitments
                        FieldType::IssuedSupply => Once
                    },
                    closes: type_map! {
                        OwnedRightType::Inflation => OnceOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::OpenEpoch => NoneOrOnce,
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!(),
                },
                TransitionType::Transfer => TransitionSchema {
                    metadata: none!(),
                    closes: type_map! {
                        OwnedRightType::Assets => OnceOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!()
                },
                TransitionType::Epoch => TransitionSchema {
                    metadata: none!(),
                    closes: type_map! {
                        OwnedRightType::OpenEpoch => Once
                    },
                    owned_rights: type_map! {
                        OwnedRightType::OpenEpoch => NoneOrOnce,
                        OwnedRightType::BurnReplace => NoneOrOnce
                    },
                    public_rights: none!()
                },
                TransitionType::Burn => burn(),
                TransitionType::BurnAndReplace => TransitionSchema {
                    metadata: type_map! {
                        FieldType::BurnedSupply => Once,
                        // Normally issuer should aggregate burned assets into a
                        // single UTXO; however if burn happens as a result of
                        // mistake this will be impossible, so we allow to have
                        // multiple burned UTXOs as a part of a single operation
                        FieldType::BurnUtxo => OnceOrMore,
                        // We need this field in order to be able to verify pedersen
                        // commitments
                        FieldType::IssuedSupply => Once,
                        FieldType::HistoryProof => NoneOrMore
                    },
                    closes: type_map! {
                        OwnedRightType::BurnReplace => Once
                    },
                    owned_rights: type_map! {
                        OwnedRightType::BurnReplace => NoneOrOnce,
                        OwnedRightType::Assets => OnceOrMore
                    },
                    public_rights: none!()
                },
                TransitionType::Renomination => renomination(),
                // Allows split of rights if they were occasionally allocated to the
                // same UTXO, for instance both assets and issuance right. Without
                // this type of transition either assets or inflation rights will be
                // lost.
                TransitionType::RightsSplit => TransitionSchema {
                    metadata: type_map! {},
                    closes: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::Assets => NoneOrMore,
                        OwnedRightType::OpenEpoch => NoneOrOnce,
                        OwnedRightType::BurnReplace => NoneOrMore,
                        OwnedRightType::Renomination => NoneOrOnce
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::Assets => NoneOrMore,
                        OwnedRightType::OpenEpoch => NoneOrOnce,
                        OwnedRightType::BurnReplace => NoneOrMore,
                        OwnedRightType::Renomination => NoneOrOnce
                    },
                    public_rights: none!()
                }
            },
            field_types: type_map! {
                // Rational: if we will use just 26 letters of English alphabet (and
                // we are not limited by them), we will have 26^8 possible tickers,
                // i.e. > 208 trillions, which is sufficient amount
                FieldType::Ticker => TypeRef::ascii_string(),
                FieldType::Name => TypeRef::ascii_string(),
                // Contract text may contain URL, text or text representation of
                // Ricardian contract, up to 64kb. If the contract doesn't fit, a
                // double SHA256 hash and URL should be used instead, pointing to
                // the full contract text, where hash must be represented by a
                // hexadecimal string, optionally followed by `\n` and text URL.
                FieldType::Contract => TypeRef::ascii_string(),
                FieldType::Precision => TypeRef::u8(),
                // We need this b/c allocated amounts are hidden behind Pedersen
                // commitments
                FieldType::IssuedSupply => TypeRef::u64(),
                // Supply in either burn or burn-and-replace procedure
                FieldType::BurnedSupply => TypeRef::u64(),
                // While UNIX timestamps allow negative numbers; in context of RGB
                // Schema, assets can't be issued in the past before RGB or Bitcoin
                // even existed; so we prohibit all the dates before RGB release
                // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
                FieldType::Timestamp => TypeRef::i64(),
                FieldType::HistoryProof => TypeRef::new("HistoryProof"),
                FieldType::BurnUtxo => TypeRef::new("OutPoint")
            },
            owned_right_types: type_map! {
                // How much issuer can issue tokens on this path. If there is no
                // limit, than `core::u64::MAX` / sum(inflation_assignments)
                // must be used, as this will be a de-facto limit to the
                // issuance
                OwnedRightType::Inflation => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                OwnedRightType::Assets => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                OwnedRightType::OpenEpoch => StateSchema::Declarative,
                OwnedRightType::BurnReplace => StateSchema::Declarative,
                OwnedRightType::Renomination => StateSchema::Declarative
            },
            public_right_types: none!(),
            script: ValidationScript::Embedded,
            override_rules: OverrideRules::AllowAnyVm,
        }
    }

    fn rgb20_inflationary() -> Schema {
        use Occurrences::*;

        Schema {
            rgb_features: none!(),
            root_id: SchemaId::from_str(Schema::RGB20_ROOT_BECH32)
                .expect("Broken root schema ID for RGB20 sub-schema"),
            type_system: none!(),
            genesis: GenesisSchema {
                metadata: type_map! {
                    FieldType::Ticker => Once,
                    FieldType::Name => Once,
                    FieldType::Precision => Once,
                    FieldType::Timestamp => Once,
                    // We need this field in order to be able to verify pedersen
                    // commitments
                    FieldType::IssuedSupply => Once
                },
                owned_rights: type_map! {
                    OwnedRightType::Inflation => NoneOrMore,
                    OwnedRightType::Assets => NoneOrMore
                },
                public_rights: none!(),
            },
            extensions: none!(),
            transitions: type_map! {
                TransitionType::Issue => TransitionSchema {
                    metadata: type_map! {
                        // We need this field in order to be able to verify pedersen
                        // commitments
                        FieldType::IssuedSupply => Once
                    },
                    closes: type_map! {
                        OwnedRightType::Inflation => OnceOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!(),
                },
                TransitionType::Transfer => TransitionSchema {
                    metadata: none!(),
                    closes: type_map! {
                        OwnedRightType::Assets => OnceOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!()
                },
                // Allows split of rights if they were occasionally allocated to the
                // same UTXO, for instance both assets and issuance right. Without
                // this type of transition either assets or inflation rights will be
                // lost.
                TransitionType::RightsSplit => TransitionSchema {
                    metadata: type_map! {},
                    closes: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::Assets => NoneOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Inflation => NoneOrMore,
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!()
                }
            },
            field_types: type_map! {
                // Rational: if we will use just 26 letters of English alphabet (and
                // we are not limited by them), we will have 26^8 possible tickers,
                // i.e. > 208 trillions, which is sufficient amount
                FieldType::Ticker => TypeRef::ascii_string(),
                FieldType::Name => TypeRef::ascii_string(),
                FieldType::Precision => TypeRef::u8(),
                // We need this b/c allocated amounts are hidden behind Pedersen
                // commitments
                FieldType::IssuedSupply => TypeRef::u64(),
                // While UNIX timestamps allow negative numbers; in context of RGB
                // Schema, assets can't be issued in the past before RGB or Bitcoin
                // even existed; so we prohibit all the dates before RGB release
                // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
                FieldType::Timestamp => TypeRef::i64()
            },
            owned_right_types: type_map! {
                // How much issuer can issue tokens on this path. If there is no
                // limit, than `core::u64::MAX` / sum(inflation_assignments)
                // must be used, as this will be a de-facto limit to the
                // issuance
                OwnedRightType::Inflation => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                OwnedRightType::Assets => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit)
            },
            public_right_types: none!(),
            script: ValidationScript::Embedded,
            override_rules: OverrideRules::AllowAnyVm,
        }
    }

    fn rgb20_simple() -> Schema {
        use Occurrences::*;

        Schema {
            rgb_features: none!(),
            root_id: SchemaId::from_str(Schema::RGB20_ROOT_BECH32)
                .expect("Broken root schema ID for RGB20 sub-schema"),
            type_system: none!(),
            genesis: GenesisSchema {
                metadata: type_map! {
                    FieldType::Ticker => Once,
                    FieldType::Name => Once,
                    FieldType::Precision => Once,
                    FieldType::Timestamp => Once,
                    // We need this field in order to be able to verify pedersen
                    // commitments
                    FieldType::IssuedSupply => Once
                },
                owned_rights: type_map! {
                    OwnedRightType::Assets => NoneOrMore
                },
                public_rights: none!(),
            },
            extensions: none!(),
            transitions: type_map! {
                TransitionType::Transfer => TransitionSchema {
                    metadata: none!(),
                    closes: type_map! {
                        OwnedRightType::Assets => OnceOrMore
                    },
                    owned_rights: type_map! {
                        OwnedRightType::Assets => NoneOrMore
                    },
                    public_rights: none!()
                }
            },
            field_types: type_map! {
                // Rational: if we will use just 26 letters of English alphabet (and
                // we are not limited by them), we will have 26^8 possible tickers,
                // i.e. > 208 trillions, which is sufficient amount
                FieldType::Ticker => TypeRef::ascii_string(),
                FieldType::Name => TypeRef::ascii_string(),
                FieldType::Precision => TypeRef::u8(),
                // We need this b/c allocated amounts are hidden behind Pedersen
                // commitments
                FieldType::IssuedSupply => TypeRef::u64(),
                // While UNIX timestamps allow negative numbers; in context of RGB
                // Schema, assets can't be issued in the past before RGB or Bitcoin
                // even existed; so we prohibit all the dates before RGB release
                // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
                FieldType::Timestamp => TypeRef::i64()
            },
            owned_right_types: type_map! {
                OwnedRightType::Assets => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit)
            },
            public_right_types: none!(),
            script: ValidationScript::Embedded,
            override_rules: OverrideRules::AllowAnyVm,
        }
    }
}

#[cfg(test)]
mod test {
    use lnpbp::bech32::Bech32ZipString;
    use rgb::schema::SchemaVerify;
    use rgb::Validity;
    use strict_encoding::{StrictDecode, StrictEncode};

    use super::*;

    #[test]
    fn schema_id() {
        let id = Schema::rgb20_root().schema_id();
        assert_eq!(id.to_string(), Schema::RGB20_ROOT_BECH32);
        assert_eq!(
            id.to_string(),
            "rgbsh1hacf8gg863veu292hdnttynqzk5xdvyk5q2fxep3e85j4ttzd05s2j4ern"
        );
    }

    #[test]
    fn subschema_inflationary_id() {
        let id = Schema::rgb20_inflationary().schema_id();
        assert_eq!(id.to_string(), Schema::RGB20_INFLATIONARY_BECH32);
        assert_eq!(
            id.to_string(),
            "rgbsh1qmts2pmfxt9e6tpuevk2v0dza30d9v4n0cq6vtm0jtppnyz5xrss4gj9wd"
        );
    }

    #[test]
    fn subschema_simple_id() {
        let id = Schema::rgb20_simple().schema_id();
        assert_eq!(id.to_string(), Schema::RGB20_SIMPLE_BECH32);
        assert_eq!(
            id.to_string(),
            "rgbsh13c3e8ywrmsu9j0k3er0lgzp9memn5c55rw5svf0l9n3sfntv76zqehteur"
        );
    }

    #[test]
    fn schema_strict_encode() {
        let data = Schema::rgb20_root()
            .strict_serialize()
            .expect("RGB-20 schema serialization failed");

        let bech32data = data.bech32_zip_string();
        println!("{}", bech32data);

        let schema20 =
            Schema::strict_deserialize(data).expect("RGB-20 schema deserialization failed");

        assert_eq!(Schema::rgb20_root(), schema20);
        assert_eq!(
            format!("{:#?}", Schema::rgb20_root()),
            format!("{:#?}", schema20)
        );
        assert_eq!(
            bech32data,
            "z1qxz4qjcwsgcpqld5lp94e4rcqx87xnskrmqe3vy3qscar828q8wuzp3cyp6kvjhaysfy4736xwhl8hjs3shg\
            6fdk7yu5h5jmjsnvj5gp4w8rvvx8a6fy2jtueg2q9pxctl3sxd0cxqqvephjqk46ew5qgxdr296zmt4eeatts6r\
            d50n694jmltsn4gszw2rgjaqv2xjnw8eelyjvfvwq4eh0rzfqr4sks2j4g7xdqjw6adtuxf868xasvawtw4llwm\
            6pcntxhjdnhtw40l7s5quemlx7h2j4eu7cc33dfs3s4tt5xthhfldayax7d0yxd74lulmuplhvqe8sr54gn3mcq\
            t6sylhqf2u"
        );
    }

    #[test]
    fn subschema_verify() {
        let status = Schema::rgb20_inflationary().schema_verify(&Schema::rgb20_root());
        assert_eq!(status.validity(), Validity::Valid);

        let status = Schema::rgb20_simple().schema_verify(&Schema::rgb20_root());
        assert_eq!(status.validity(), Validity::Valid);
    }
}
