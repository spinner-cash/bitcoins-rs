//! Specifies an abstract `AddressEncoder` that is used to convert `Address` to/from the in-memory
//! representation of the associated spending constraint.

use crate::{
    bitcoin::script::{Script},  // TODO: REFACTOR THIS OUT AND GENERALIZE
};

/// An AddressEncoder encodes and decodes addresses. This struct is used by the Builder to decode
/// addresses, and is associated with a Network object. It handles converting addresses to
/// scripts and vice versa. It also contains a function that wraps a string in the appropriate
/// address type.
///
/// The associated `Address` type defines what the encoder considers to be an "address."
///
/// A Bitcoin encoder can be found in the `bitcoin` module.
pub trait AddressEncoder {
    /// A type representing the encoded address
    type Address;
    /// An error type that will be returned in case of encoding errors
    type Error;

    /// Encode a script as an address.
    fn encode_address(s: Script) -> Result<Self::Address, Self::Error>;

    /// Decode a script from an address.
    fn decode_address(addr: Self::Address) -> Result<Script, Self::Error>;

    /// Convert a string into an address.
    fn wrap_string(s: String) -> Result<Self::Address, Self::Error>;
}