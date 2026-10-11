//! Block pointers and hash-type inheritance (`utils.py`, section "Hash types").
//!
//! A pointer is a `buffer.Buffer.Block`: a list of (type, value) hashes. On the
//! wire every hash carries its type. In storage a hash can omit it and inherit
//! the type at the same index from the nearest ancestor that has one.

use crate::buffer::buffer::Block;
use crate::hash::HashAlgorithm;
use crate::Error;

/// The pointer for `block_id` under `algorithm` (`utils.block_pointer`).
///
/// One hash. With `omit_types`, the hash has no type (storage only, never on
/// the wire, never at the top of a stored tree).
pub fn block_pointer(block_id: &[u8], algorithm: HashAlgorithm, omit_types: bool) -> Block {
    let _ = (block_id, algorithm, omit_types);
    todo!("step 2: pointers")
}

/// The type of each hash of `block`, with `inherited` where a hash has none
/// (`utils.resolve_hash_types`). An error if a type cannot be deduced.
pub fn resolve_hash_types(
    block: &Block,
    inherited: Option<&[Vec<u8>]>,
) -> Result<Vec<Vec<u8>>, Error> {
    let _ = (block, inherited);
    todo!("step 2: pointers")
}

/// The types that apply inside the block that `resolved` points to
/// (`utils.inherit_hash_types`): `resolved`, then the rest of `inherited`.
pub fn inherit_hash_types(resolved: &[Vec<u8>], inherited: Option<&[Vec<u8>]>) -> Vec<Vec<u8>> {
    let _ = (resolved, inherited);
    todo!("step 2: pointers")
}

/// The id that `block` names under `algorithm`, or `None`
/// (`utils.block_id_from_pointer`). `None` also when a type cannot be deduced.
pub fn block_id_from_pointer(
    block: &Block,
    inherited: Option<&[Vec<u8>]>,
    algorithm: HashAlgorithm,
) -> Option<Vec<u8>> {
    let _ = (block, inherited, algorithm);
    todo!("step 2: pointers")
}
