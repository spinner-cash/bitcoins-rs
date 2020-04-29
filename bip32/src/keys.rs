use crate::{
    Bip32Error,
    model::*,
    curve::{Secp256k1Backend}
};

#[cfg(any(feature = "libsecp", feature = "rust-secp"))]
/// A Private Key using the crate's compiled-in backend.
/// This type is available whenever a compiled-in backend is used.
pub type Privkey<'a> = GenericPrivkey<'a, crate::Secp256k1<'a>>;
#[cfg(any(feature = "libsecp", feature = "rust-secp"))]
/// A Public Key using the crate's compiled-in backend.
/// This type is available whenever a compiled-in backend is used.
pub type Pubkey<'a> = GenericPubkey<'a, crate::Secp256k1<'a>>;

/// A Private key with a reference to its associated backend
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GenericPrivkey<'a, T: Secp256k1Backend<'a>> {
    /// The private key.
    pub key: T::Privkey,
    /// A reference to the backend. Many operations will return errors if this is None.
    pub backend: Option<&'a T>,
}

impl<'a, T: Secp256k1Backend<'a>> HasPrivkey<'a, T> for GenericPrivkey<'a, T> {
    fn privkey(&self) -> &T::Privkey {
        &self.key
    }
}

impl<'a, T: Secp256k1Backend<'a>> HasBackend<'a, T> for GenericPrivkey<'a, T> {
    fn backend(&self) -> Result<&'a T, Bip32Error> {
        self.backend.ok_or(Bip32Error::NoBackend)
    }
}

impl<'a, T: Secp256k1Backend<'a>> SigningKey<'a, T> for GenericPrivkey<'a, T> {
    /// The corresponding verifying key
    type VerifyingKey = GenericPubkey<'a, T>;

    /// Derive the corresponding pubkey
    fn derive_verifying_key(&self) -> Result<Self::VerifyingKey, Bip32Error> {
        Ok(GenericPubkey {
            key: self.derive_pubkey()?,
            backend: self.backend,
        })
    }
}

/// A Public key with a reference to its associated backend
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GenericPubkey<'a, T: Secp256k1Backend<'a>> {
    /// The public key.
    pub key: T::Pubkey,
    /// A reference to the backend. Many operations will return errors if this is None.
    pub backend: Option<&'a T>,
}

impl<'a, T: Secp256k1Backend<'a>> HasPubkey<'a, T> for GenericPubkey<'a, T> {
    fn pubkey(&self) -> &T::Pubkey {
        &self.key
    }
}

impl<'a, T: Secp256k1Backend<'a>> HasBackend<'a, T> for GenericPubkey<'a, T> {
    fn backend(&self) -> Result<&'a T, Bip32Error> {
        self.backend.ok_or(Bip32Error::NoBackend)
    }
}

impl<'a, T: Secp256k1Backend<'a>> VerifyingKey<'a, T> for GenericPubkey<'a, T> {
    type SigningKey = GenericPrivkey<'a, T>;
}