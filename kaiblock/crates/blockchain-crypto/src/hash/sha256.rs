use sha2::{Sha256, Digest};
use crate::hash::merkle::Hash;


// take in a slice of bytes &[u8], computes the sha-256 hash of the data
// and returns the hash as a [u8; 32] array
// into() works as finalize returns GenericArray<u8, U32>
// thus giving us array [u8, 32]
fn hash_sha256(data: &[u8]) ->String{
    let mut hasher = Sha256::new();
    hasher.update(data);

    hasher.finalize().into()

}


// Merkle tree implementation.
// parent = hash(left_child_hash || right_child_hash)
fn hash_pair(left: &Hash, right: &Hash) -> Hash{
	// pre-allocate a 64 byte vec
	let mut combined = Vec::with_capacity(64);
	// append both the left and rignt(msee documenting ni kaa naandika novel sitaai publish: that is publicly) hashes and hash the combo.
	combined.extend_from_slice(left);
	combined.extend_from_slice(right);
	hash_sha256(&combined)
}