//! Hash algorithms and hash types (`utils.py`, section "Hash algorithms").
//!
//! A hash type is the digest of the algorithm over the empty input. A pointer
//! carries it next to each hash value, so a reader knows which algorithm made it.

/// The algorithms that the reference registers by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// SHA3-256. The default of the reference (`Enviroment.hash_factory`).
    #[default]
    Sha3_256,
    /// SHA3-512.
    Sha3_512,
    /// SHA-256.
    Sha256,
    /// SHA-512.
    Sha512,
    /// BLAKE2b with a 32-byte digest.
    Blake2b256,
    /// BLAKE2b with a 64-byte digest.
    Blake2b512,
}

impl HashAlgorithm {
    /// All algorithms, in the order above.
    pub const ALL: [HashAlgorithm; 6] = [
        HashAlgorithm::Sha3_256,
        HashAlgorithm::Sha3_512,
        HashAlgorithm::Sha256,
        HashAlgorithm::Sha512,
        HashAlgorithm::Blake2b256,
        HashAlgorithm::Blake2b512,
    ];

    /// A short name: `sha3_256`, `blake2b_256`, ... (the names of the reference tests).
    pub fn name(self) -> &'static str {
        match self {
            HashAlgorithm::Sha3_256 => "sha3_256",
            HashAlgorithm::Sha3_512 => "sha3_512",
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Sha512 => "sha512",
            HashAlgorithm::Blake2b256 => "blake2b_256",
            HashAlgorithm::Blake2b512 => "blake2b_512",
        }
    }

    /// The hash type: the digest of this algorithm over the empty input.
    pub fn hash_type(self) -> Vec<u8> {
        todo!("step 2: hash types")
    }

    /// The algorithm whose hash type is `hash_type`, if it is one of [`Self::ALL`].
    pub fn from_hash_type(hash_type: &[u8]) -> Option<HashAlgorithm> {
        let _ = hash_type;
        todo!("step 2: hash types")
    }

    /// A new incremental hasher.
    pub fn hasher(self) -> Hasher {
        todo!("step 2: hash types")
    }

    /// The digest of `data`.
    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        let mut h = self.hasher();
        h.update(data);
        h.finalize()
    }
}

/// An incremental hasher for one [`HashAlgorithm`].
pub struct Hasher {
    // Add the state you need. The field is private, so you can change it.
    _private: (),
}

impl Hasher {
    /// Add bytes.
    pub fn update(&mut self, data: &[u8]) {
        let _ = data;
        todo!("step 2: hash types")
    }

    /// The digest of all the bytes added.
    pub fn finalize(self) -> Vec<u8> {
        todo!("step 2: hash types")
    }
}
