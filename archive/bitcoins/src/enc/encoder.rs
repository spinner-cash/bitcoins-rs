//! Defines parameterized Bitcoin encoders for Mainnet, Testnet, and Signet.

use std::marker::PhantomData;

use coins_core::{
    enc::bases::{decode_base58, encode_base58},
    enc::{AddressEncoder, EncodingError, EncodingResult},
    hashes::MarkedDigestOutput,
};

use crate::{
    enc::bases::{decode_bech32, encode_bech32},
    types::script::{ScriptPubkey, ScriptType},
};

/// The available Bitcoin Address types, implemented as a type enum around strings.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Address {
    /// Legacy Pay to Pubkeyhash
    Pkh(String),
    /// Legacy Pay to Scripthash
    Sh(String),
    /// Witness Pay to Pubkeyhash
    Wpkh(String),
    /// Witness Pay to Scripthash
    Wsh(String),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr = match &self {
            Address::Pkh(s) => s,
            Address::Sh(s) => s,
            Address::Wpkh(s) => s,
            Address::Wsh(s) => s,
        };
        write!(f, "{}", addr)
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        match &self {
            Address::Pkh(s) => s,
            Address::Sh(s) => s,
            Address::Wpkh(s) => s,
            Address::Wsh(s) => s,
        }
    }
}

impl Address {
    /// Get a clone of the string underlying the address type.
    pub fn as_string(&self) -> String {
        match &self {
            Address::Pkh(s) => s.clone(),
            Address::Sh(s) => s.clone(),
            Address::Wpkh(s) => s.clone(),
            Address::Wsh(s) => s.clone(),
        }
    }

    /// Convert the address to an `addr()` descriptor
    pub fn to_descriptor(&self) -> String {
        format!("addr({})", self.as_string())
    }
}

/// NetworkParams holds the encoding paramteres for a bitcoin-like network. Currently this is
/// composed of the address version bytes for Legacy PKH and SH addresses, and the bech32
/// human-readable prefix for witness addresses.
pub trait NetworkParams {
    /// The BECH32 HRP. "bc" for mainnet.
    const HRP: &'static str;
    /// The Legacy PKH base58check version byte. 0x00 for mainnet.
    const PKH_VERSION: u8;
    /// The Legacy SH base58check version byte. 0x05 for mainnet.
    const SH_VERSION: u8;
}

/// Marker trait to simplify encoder representation elsewhere
pub trait BitcoinEncoderMarker:
    AddressEncoder<Address = Address, Error = EncodingError, RecipientIdentifier = ScriptPubkey>
{
}

/// The standard encoder for Bitcoin networks. Parameterized by a `NetworkParams` type and an
/// `coins_bip32::Encoder`. It exposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitcoinEncoder<P: NetworkParams>(PhantomData<fn(P) -> P>);

impl<P: NetworkParams> AddressEncoder for BitcoinEncoder<P> {
    type Address = Address;
    type Error = EncodingError;
    type RecipientIdentifier = ScriptPubkey;

    fn encode_address(s: &ScriptPubkey) -> EncodingResult<Address> {
        match s.standard_type() {
            ScriptType::Pkh(payload) => {
                // s.items contains the op codes. we want only the pkh
                Ok(Address::Pkh(encode_base58(
                    P::PKH_VERSION,
                    payload.as_slice(),
                )))
            }
            ScriptType::Sh(payload) => {
                // s.items contains the op codes. we want only the sh
                Ok(Address::Sh(encode_base58(
                    P::SH_VERSION,
                    payload.as_slice(),
                )))
            }
            ScriptType::Wsh(_) => Ok(Address::Wsh(encode_bech32(P::HRP, s.items())?)),
            ScriptType::Wpkh(_) => Ok(Address::Wpkh(encode_bech32(P::HRP, s.items())?)),
            ScriptType::OpReturn(_) => Err(EncodingError::NullDataScript),
            ScriptType::NonStandard => Err(EncodingError::UnknownScriptType),
        }
    }

    fn decode_address(addr: &Address) -> ScriptPubkey {
        match &addr {
            Address::Pkh(s) => {
                let mut v: Vec<u8> = vec![0x76, 0xa9, 0x14]; // DUP, HASH160, PUSH_20
                v.extend(&decode_base58(P::PKH_VERSION, s).unwrap());
                v.extend(&[0x88, 0xac]); // EQUALVERIFY, CHECKSIG
                v.into()
            }
            Address::Sh(s) => {
                let mut v: Vec<u8> = vec![0xa9, 0x14]; // HASH160, PUSH_20
                v.extend(&decode_base58(P::SH_VERSION, s).unwrap());
                v.extend(&[0x87]); // EUQAL
                v.into()
            }
            Address::Wpkh(s) | Address::Wsh(s) => decode_bech32(P::HRP, s).unwrap().into(),
        }
    }

    fn string_to_address(string: &str) -> EncodingResult<Address> {
        let s = string.to_owned();
        if s.starts_with(P::HRP) {
            let result = decode_bech32(P::HRP, &s)?;
            match result.len() {
                22 => Ok(Address::Wpkh(s)),
                34 => Ok(Address::Wsh(s)),
                _ => Err(EncodingError::UnknownScriptType),
            }
        } else if decode_base58(P::PKH_VERSION, &s).is_ok() {
            Ok(Address::Pkh(s))
        } else if decode_base58(P::SH_VERSION, &s).is_ok() {
            Ok(Address::Sh(s))
        } else {
            Err(EncodingError::UnknownScriptType)
        }
    }
}

impl<P: NetworkParams> BitcoinEncoderMarker for BitcoinEncoder<P> {}

/// A param struct for Bitcoin Mainnet
#[derive(Debug, Clone)]
pub struct Main;

impl NetworkParams for Main {
    const HRP: &'static str = "bc";
    const PKH_VERSION: u8 = 0x00;
    const SH_VERSION: u8 = 0x05;
}

/// A param struct for Bitcoin Tesnet
#[derive(Debug, Clone)]
pub struct Test;

impl NetworkParams for Test {
    const HRP: &'static str = "tb";
    const PKH_VERSION: u8 = 0x6f;
    const SH_VERSION: u8 = 0xc4;
}

/// A param struct for Bitcoin Signet
#[derive(Debug, Clone)]
pub struct Sig;

impl NetworkParams for Sig {
    const HRP: &'static str = "sb";
    const PKH_VERSION: u8 = 0x7d;
    const SH_VERSION: u8 = 0x57;
}

/// An encoder for Bitcoin Mainnet
pub type MainnetEncoder = BitcoinEncoder<Main>;

/// An encoder for Bitcoin Tesnet
pub type TestnetEncoder = BitcoinEncoder<Test>;

/// An encoder for Bitcoin Signet
pub type SignetEncoder = BitcoinEncoder<Sig>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_wraps_address_strings() {
        let cases = [
            (
                "bc1qza7dfgl2q83cf68fqkkdd754qx546h4u9vd9tg".to_owned(),
                Address::Wpkh("bc1qza7dfgl2q83cf68fqkkdd754qx546h4u9vd9tg".to_owned()),
            ),
            (
                "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej".to_owned(),
                Address::Wsh(
                    "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej".to_owned(),
                ),
            ),
            (
                "1AqE7oGF1EUoJviX1uuYrwpRBdEBTuGhES".to_owned(),
                Address::Pkh("1AqE7oGF1EUoJviX1uuYrwpRBdEBTuGhES".to_owned()),
            ),
            (
                "3HXNFmJpxjgTVFN35Y9f6Waje5YFsLEQZ2".to_owned(),
                Address::Sh("3HXNFmJpxjgTVFN35Y9f6Waje5YFsLEQZ2".to_owned()),
            ),
        ];
        for case in cases.iter() {
            assert_eq!(MainnetEncoder::string_to_address(&case.0).unwrap(), case.1);
        }

        let errors = [
            "hello",
            "this isn't a real address",
            "bc10pu8s7rc0pu8s7rc0putt44am", // valid bech32, bad length
        ];
        for case in errors.iter() {
            match MainnetEncoder::string_to_address(case) {
                Err(EncodingError::UnknownScriptType) => {}
                _ => panic!("expected err UnknownScriptType"),
            }
        }
    }

    #[test]
    fn it_encodes_addresses() {
        let cases = [
            (
                ScriptPubkey::new(
                    hex::decode("a914e88869b88866281ab166541ad8aafba8f8aba47a87").unwrap(),
                ),
                Address::Sh("3NtY7BrF3xrcb31JXXaYCKVcz1cH3Azo5y".to_owned()),
            ),
            (
                ScriptPubkey::new(
                    hex::decode("76a9140e5c3c8d420c7f11e88d76f7b860d471e6517a4488ac").unwrap(),
                ),
                Address::Pkh("12JvxPk4mT4PKMVHuHc1aQGBZpotQWQwF6".to_owned()),
            ),
            (
                ScriptPubkey::new(
                    hex::decode(
                        "00201bf8a1831db5443b42a44f30a121d1b616d011ab15df62b588722a845864cc99",
                    )
                    .unwrap(),
                ),
                Address::Wsh(
                    "bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydtzh0k9dvgwg4ggkryejvsy49jvz".to_owned(),
                ),
            ),
            (
                ScriptPubkey::new(
                    hex::decode("00141bf8a1831db5443b42a44f30a121d1b616d011ab").unwrap(),
                ),
                Address::Wpkh("bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydt3wy5yh".to_owned()),
            ),
        ];
        for case in cases.iter() {
            assert_eq!(MainnetEncoder::encode_address(&case.0).unwrap(), case.1);
        }
        let errors = [
            (ScriptPubkey::new(hex::decode("01201bf8a1831db5443b42a44f30a121d1b616d011ab15df62b588722a845864cc99").unwrap())), // wrong witness program version
            (ScriptPubkey::new(hex::decode("a914e88869b88866281ab166541ad8aafba8f8aba47a89").unwrap())), // wrong last byte
            (ScriptPubkey::new(hex::decode("aa14e88869b88866281ab166541ad8aafba8f8aba47a87").unwrap())), // wrong first byte
            (ScriptPubkey::new(hex::decode("76a9140e5c3c8d420c7f11e88d76f7b860d471e6517a4488ad").unwrap())), // wrong last byte
            (ScriptPubkey::new(hex::decode("77a9140e5c3c8d420c7f11e88d76f7b860d471e6517a4488ac").unwrap())), // wrong first byte
            (ScriptPubkey::new(hex::decode("01141bf8a1831db5443b42a44f30a121d1b616d011ab").unwrap())), // wrong witness program version
            (ScriptPubkey::new(hex::decode("0011223344").unwrap())), // junk
            (ScriptPubkey::new(hex::decode("deadbeefdeadbeefdeadbeefdeadbeef").unwrap())), // junk
            (ScriptPubkey::new(hex::decode("02031bf8a1831db5443b42a44f30a121d1b616d011ab15df62b588722a845864cc99041bf8a1831db5443b42a44f30a121d1b616d011ab15df62b588722a845864cc9902af").unwrap())), // Raw msig
        ];
        for case in errors.iter() {
            match MainnetEncoder::encode_address(case) {
                Err(EncodingError::UnknownScriptType) => {}
                _ => panic!("expected err UnknownScriptType"),
            }
        }
    }

    #[test]
    fn it_allows_you_to_unwrap_strings_from_addresses() {
        let cases = [
            (
                "3NtY7BrF3xrcb31JXXaYCKVcz1cH3Azo5y".to_owned(),
                Address::Sh("3NtY7BrF3xrcb31JXXaYCKVcz1cH3Azo5y".to_owned()),
            ),
            (
                "12JvxPk4mT4PKMVHuHc1aQGBZpotQWQwF6".to_owned(),
                Address::Pkh("12JvxPk4mT4PKMVHuHc1aQGBZpotQWQwF6".to_owned()),
            ),
            (
                "bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydtzh0k9dvgwg4ggkryejvsy49jvz".to_owned(),
                Address::Wsh(
                    "bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydtzh0k9dvgwg4ggkryejvsy49jvz".to_owned(),
                ),
            ),
            (
                "bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydt3wy5yh".to_owned(),
                Address::Wpkh("bc1qr0u2rqcak4zrks4yfuc2zgw3kctdqydt3wy5yh".to_owned()),
            ),
        ];
        for case in cases.iter() {
            assert_eq!(case.1.as_string(), case.0);
        }
    }
}
