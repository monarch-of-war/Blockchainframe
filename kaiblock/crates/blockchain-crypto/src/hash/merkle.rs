use super::{Hash256, hash_combine};
use crate::{CryptoError, Result};
use serde::{Deserialize, Serialize};

/// Merkle tree for efficient verification of large datasets
#[derive(Debug, Clone)]
pub struct MerkleTree {
    leaves: Vec<Hash256>,
    nodes: Vec<Vec<Hash256>>,
    root: Hash256,
}

/// Proof that a leaf exists in the merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub leaf_hash: Hash256,
    pub siblings: Vec<Hash256>,
    pub root: Hash256,
}

impl MerkleTree {
    /// Create a new merkle tree from leaf hashes
    pub fn new(leaves: Vec<Hash256>) -> Result<Self> {
        if leaves.is_empty() {
            return Err(CryptoError::InvalidHash("Cannot create merkle tree with no leaves".to_string()));
        }
        
        let mut tree = MerkleTree {
            leaves: leaves.clone(),
            nodes: Vec::new(),
            root: Hash256::zero(),
        };
        
        tree.build_tree()?;
        Ok(tree)
    }
    
    /// Build the merkle tree
    fn build_tree(&mut self) -> Result<()> {
        let mut current_level = self.leaves.clone();
        self.nodes.push(current_level.clone());
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            // Process pairs of nodes
            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(left); // Duplicate last node if odd
                
                let combined = hash_combine(&[left.as_bytes(), right.as_bytes()]);
                next_level.push(combined);
            }
            
            self.nodes.push(next_level.clone());
            current_level = next_level;
        }
        
        self.root = current_level[0];
        Ok(())
    }
    
    /// Get the merkle root
    pub fn root(&self) -> Hash256 {
        self.root
    }
    
    /// Get all leaf hashes
    pub fn leaves(&self) -> &[Hash256] {
        &self.leaves
    }
    
    /// Generate a merkle proof for a specific leaf
    pub fn generate_proof(&self, leaf_index: usize) -> Result<MerkleProof> {
        if leaf_index >= self.leaves.len() {
            return Err(CryptoError::InvalidMerkleProof);
        }
        
        let mut siblings = Vec::new();
        let mut current_index = leaf_index;
        
        // Traverse from leaf to root, collecting siblings
        for level in 0..(self.nodes.len() - 1) {
            let level_nodes = &self.nodes[level];
            
            // Find sibling
            let sibling_index = if current_index % 2 == 0 {
                // Left node, sibling is right
                if current_index + 1 < level_nodes.len() {
                    current_index + 1
                } else {
                    current_index // Duplicate if no right sibling
                }
            } else {
                // Right node, sibling is left
                current_index - 1
            };
            
            siblings.push(level_nodes[sibling_index]);
            current_index /= 2; // Move to parent index
        }
        
        Ok(MerkleProof {
            leaf_index,
            leaf_hash: self.leaves[leaf_index],
            siblings,
            root: self.root,
        })
    }
    
    /// Verify a merkle proof
    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut current_hash = proof.leaf_hash;
        let mut current_index = proof.leaf_index;
        
        for &sibling in &proof.siblings {
            current_hash = if current_index % 2 == 0 {
                // Current is left, sibling is right
                hash_combine(&[current_hash.as_bytes(), sibling.as_bytes()])
            } else {
                // Current is right, sibling is left
                hash_combine(&[sibling.as_bytes(), current_hash.as_bytes()])
            };
            
            current_index /= 2;
        }
        
        current_hash == proof.root
    }
}

/// Create a merkle tree from raw data (will be hashed)
pub fn merkle_tree_from_data(data: &[&[u8]]) -> Result<MerkleTree> {
    let leaves: Vec<Hash256> = data.iter()
        .map(|d| super::sha256(d))
        .collect();
    
    MerkleTree::new(leaves)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::sha256;

    #[test]
    fn test_merkle_tree_creation() {
        let data = vec![b"data1", b"data2", b"data3", b"data4"];
        let tree = merkle_tree_from_data(&data).unwrap();
        
        assert_eq!(tree.leaves().len(), 4);
        assert!(!tree.root().is_zero());
    }

    #[test]
    fn test_merkle_proof_generation_and_verification() {
        let data = vec![b"apple", b"banana", b"cherry", b"date"];
        let tree = merkle_tree_from_data(&data).unwrap();
        
        // Generate proof for "banana" (index 1)
        let proof = tree.generate_proof(1).unwrap();
        
        assert_eq!(proof.leaf_index, 1);
        assert_eq!(proof.leaf_hash, sha256(b"banana"));
        assert_eq!(proof.root, tree.root());
        
        // Verify the proof
        assert!(MerkleTree::verify_proof(&proof));
    }

    #[test]
    fn test_single_leaf_tree() {
        let leaves = vec![sha256(b"single_leaf")];
        let tree = MerkleTree::new(leaves).unwrap();
        
        let proof = tree.generate_proof(0).unwrap();
        assert!(MerkleTree::verify_proof(&proof));
    }

    #[test]
    fn test_odd_number_of_leaves() {
        let data = vec![b"one", b"two", b"three"];
        let tree = merkle_tree_from_data(&data).unwrap();
        
        // Test proof for each leaf
        for i in 0..3 {
            let proof = tree.generate_proof(i).unwrap();
            assert!(MerkleTree::verify_proof(&proof));
        }
    }

    #[test]
    fn test_invalid_proof() {
        let data = vec![b"test1", b"test2"];
        let tree = merkle_tree_from_data(&data).unwrap();
        
        let mut proof = tree.generate_proof(0).unwrap();
        
        // Tamper with the proof
        proof.leaf_hash = sha256(b"tampered");
        
        // Verification should fail
        assert!(!MerkleTree::verify_proof(&proof));
    }
}