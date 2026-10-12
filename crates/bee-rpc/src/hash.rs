//! Hash algorithms and hash types (`utils.py`, section "Hash algorithms").
//!
//! A hash type is the digest of the algorithm over the empty input. A pointer
//! carries it next to each hash value, so a reader knows which algorithm made it.

use blake2::digest::consts::U32;
use blake2::{Blake2b, Blake2b512};
use sha2::{Digest, Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

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
        self.hash_type_bytes().to_vec()
    }

    /// The algorithm whose hash type is `hash_type`, if it is one of [`Self::ALL`].
    pub fn from_hash_type(hash_type: &[u8]) -> Option<HashAlgorithm> {
        HashAlgorithm::ALL
            .into_iter()
            .find(|alg| alg.hash_type_bytes() == hash_type)
    }

    /// The hash type, from the constants below. Each one is the digest of the
    /// empty input, so there is nothing to compute on each call.
    fn hash_type_bytes(self) -> &'static [u8] {
        match self {
            HashAlgorithm::Sha3_256 => SHA3_256,
            HashAlgorithm::Sha3_512 => SHA3_512,
            HashAlgorithm::Sha256 => SHA256,
            HashAlgorithm::Sha512 => SHA512,
            HashAlgorithm::Blake2b256 => BLAKE2B_256,
            HashAlgorithm::Blake2b512 => BLAKE2B_512,
        }
    }

    /// A new incremental hasher.
    pub fn hasher(self) -> Hasher {
        let state = match self {
            HashAlgorithm::Sha3_256 => State::Sha3_256(Sha3_256::new()),
            HashAlgorithm::Sha3_512 => State::Sha3_512(Sha3_512::new()),
            HashAlgorithm::Sha256 => State::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => State::Sha512(Sha512::new()),
            HashAlgorithm::Blake2b256 => State::Blake2b256(Blake2b::<U32>::new()),
            HashAlgorithm::Blake2b512 => State::Blake2b512(Blake2b512::new()),
        };
        Hasher { state }
    }

    /// The digest of `data`.
    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        let mut h = self.hasher();
        h.update(data);
        h.finalize()
    }
}

/// The hash type of SHA3-256.
const SHA3_256: &[u8] = &[
    0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62,
    0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a,
];

/// The hash type of SHA3-512.
const SHA3_512: &[u8] = &[
    0xa6, 0x9f, 0x73, 0xcc, 0xa2, 0x3a, 0x9a, 0xc5, 0xc8, 0xb5, 0x67, 0xdc, 0x18, 0x5a, 0x75, 0x6e,
    0x97, 0xc9, 0x82, 0x16, 0x4f, 0xe2, 0x58, 0x59, 0xe0, 0xd1, 0xdc, 0xc1, 0x47, 0x5c, 0x80, 0xa6,
    0x15, 0xb2, 0x12, 0x3a, 0xf1, 0xf5, 0xf9, 0x4c, 0x11, 0xe3, 0xe9, 0x40, 0x2c, 0x3a, 0xc5, 0x58,
    0xf5, 0x00, 0x19, 0x9d, 0x95, 0xb6, 0xd3, 0xe3, 0x01, 0x75, 0x85, 0x86, 0x28, 0x1d, 0xcd, 0x26,
];

/// The hash type of SHA-256.
const SHA256: &[u8] = &[
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
];

/// The hash type of SHA-512.
const SHA512: &[u8] = &[
    0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd, 0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d, 0x80, 0x07,
    0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc, 0x83, 0xf4, 0xa9, 0x21, 0xd3, 0x6c, 0xe9, 0xce,
    0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0, 0xff, 0x83, 0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f,
    0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81, 0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e,
];

/// The hash type of BLAKE2b-256.
const BLAKE2B_256: &[u8] = &[
    0x0e, 0x57, 0x51, 0xc0, 0x26, 0xe5, 0x43, 0xb2, 0xe8, 0xab, 0x2e, 0xb0, 0x60, 0x99, 0xda, 0xa1,
    0xd1, 0xe5, 0xdf, 0x47, 0x77, 0x8f, 0x77, 0x87, 0xfa, 0xab, 0x45, 0xcd, 0xf1, 0x2f, 0xe3, 0xa8,
];

/// The hash type of BLAKE2b-512.
const BLAKE2B_512: &[u8] = &[
    0x78, 0x6a, 0x02, 0xf7, 0x42, 0x01, 0x59, 0x03, 0xc6, 0xc6, 0xfd, 0x85, 0x25, 0x52, 0xd2, 0x72,
    0x91, 0x2f, 0x47, 0x40, 0xe1, 0x58, 0x47, 0x61, 0x8a, 0x86, 0xe2, 0x17, 0xf7, 0x1f, 0x54, 0x19,
    0xd2, 0x5e, 0x10, 0x31, 0xaf, 0xee, 0x58, 0x53, 0x13, 0x89, 0x64, 0x44, 0x93, 0x4e, 0xb0, 0x4b,
    0x90, 0x3a, 0x68, 0x5b, 0x14, 0x48, 0xb7, 0x55, 0xd5, 0x6f, 0x70, 0x1a, 0xfe, 0x9b, 0xe2, 0xce,
];

/// An incremental hasher for one [`HashAlgorithm`].
pub struct Hasher {
    state: State,
}

enum State {
    Sha3_256(Sha3_256),
    Sha3_512(Sha3_512),
    Sha256(Sha256),
    Sha512(Sha512),
    Blake2b256(Blake2b<U32>),
    Blake2b512(Blake2b512),
}

impl Hasher {
    /// Add bytes.
    pub fn update(&mut self, data: &[u8]) {
        match &mut self.state {
            State::Sha3_256(h) => h.update(data),
            State::Sha3_512(h) => h.update(data),
            State::Sha256(h) => h.update(data),
            State::Sha512(h) => h.update(data),
            State::Blake2b256(h) => h.update(data),
            State::Blake2b512(h) => h.update(data),
        }
    }

    /// The digest of all the bytes added.
    pub fn finalize(self) -> Vec<u8> {
        match self.state {
            State::Sha3_256(h) => h.finalize().to_vec(),
            State::Sha3_512(h) => h.finalize().to_vec(),
            State::Sha256(h) => h.finalize().to_vec(),
            State::Sha512(h) => h.finalize().to_vec(),
            State::Blake2b256(h) => h.finalize().to_vec(),
            State::Blake2b512(h) => h.finalize().to_vec(),
        }
    }
}
