
use crate::block::hash_pair;

pub type Hash = [u8; 32];


#[derive(debug, Clone)]


// recursive structure of a node in merkle tree
pub enum MerkleNode {

    // leaf is the base block with the data--
    // stores the data and the hash of the data
    Leaf{
        hash: Hash,
        data: Vec<u8>,
    },

    Internal{
        hash: Hash,
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
    },
}

#[derive(Debug)]
pub struct MerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<MerkleNode>,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf: Hash,
    pub proof_path: Vec<ProofElement>,
    pub root: Hash,
}

#[derive(Debug, Clone)]
pub struct ProofElement {
    pub hash: Hash,
    pub is_right_sibling: bool,
}


impl MerkleTree{
    pub fn new(transactions: Vec<Vec<u8>>) -> Self{
        // if no transactions are provided then return
        // an empty tree--None root, no leaves
        if transactions.is_empty(){
            return MerkleTree{root: None, leaves: Vec::new()};

        }

        let mut nodes: Vec<MerkleNode> = transactions
        .iter()
        // each transaction is hashed
        .map(|tx| {
            let hash = hash_data(tx);
            // a MerkleNode::Leat is created for each transaction
            // storing the original data and the hash
            MerkleTree::Leaf{
                hash,
                data: tx.clone()
            }
        })
        .collect();
    // This now creates a Vec<MerkleNode> where each node is a leaf

    // Extract just the hash from the nodes
    // Stored in Merkle.leaves for later lookup or proof of generation
    let leaves: Vec<Hash> = nodes.iter().map(|n| n.get_hash());

    while nodes.len() > 1{
        let mut next_level = Vec::new();

        // pair up two nodes at a time
        for chunk in nodes.chunks(2) {
            match chunk{
                // for each pair, hash combined hashes and
                // create a new MerkleNode::Internal with that hash
                [left, right] => {
                    let combined_hash = hash_pair(&left.get_hash(), &right.get_hash());
                    next_level.push(MerkleNode::Internal{
                        hash: combined_hash,
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone),
                    });

                }
                // for odd number of nodes, the last node is duplicated
                [single] =>{
                    let hash = single.get_hash();
                    let duplicated_hash = hash_pair(&hash, &hash);
                    next_level.push(MerkleNode::Internal{
                        hash: duplicated_hash,
                        left: Box::new(single.clone()),
                        right: Box::new(single.clone()),
                    })
                }

                _ => unreachable!(),
            }
        }

        nodes = next_level;
    }

    // after looping, only one node remains, the root
    // which is stored together with the list of leaf hashes in Merkletree
    MerkleTree{
        root: nodes.into_iter.next(),
        leaves,
    }


    }

    pub fn get_root_hash(&self) -> Option<Hash>{
        selr.root.as_ref().map(|node| node.get_hash())
    }
}

impl MerkleNode{
    // a way to extract a node's hash
    fn get_hash(&self)-> Hash{
        match self {
            MerkleNode::Leaf {hasg, ..} => *hash,
            MerkleNode::Internal {hash, ..} => *hash,
        }
    }
}


impl MerkleTree{
    pub fn generate_proof(&self, target_leaf: &Hash) -> Option<MerkleProof>{
        let root = self.rppt.as_ref();
        let mut proof_path = Vec::new();

        if self.find_leaf_and_build_proof(root, target_leaf, &mut proof_path){
            Some(MerkleProof{
                leaf: *target_leaf,
                proof_path,
                root: root.get_hash(),
            })
        
        }
        else{
            None
        }
        
    }

    fn find_leaf_and_build_proof(
        &self,
        node: &MerkleNode,
        target: &Hash,
        proof_path: &mut Vec<ProofElement>,
    ) -> bool{
        match node {
            MerkleNode::Leaf {hash, ..} => {
                *hash == *target
            }

            MerkleNode::Internal {left, right, ..} => {
                if self.find_leaf_and_build_proof(left, target, proof_path){

                    proof_path.push(ProofElement{
                        hash: right.get_hash(),
                        is_right_sibling: true,
                    });
                    return true;
                }

                if self.find_leaf_and_build_proof(right, target, proof_path){
                    proof_path.push(ProofElement{
                        hash: left.get_hash(),
                        is_right_sibling: false,
                    });

                    return true;
                }

                false
            }
        }

    }

}

impl MerkleProof{
    pub fn verify(&self) -> bool{
        let mut current_hash = self.hash;
//  Walk up the tree using the proof path
        for proof_element in &self.proof_path{
            current_hash = if proof_element.is_right_sibling{
                // Sibling is on the right, so current hash goes on left
            hash_pair(&current_hash, &proof_element)
            }else{
                // Sibling is on the left, so current hash goes on right
                hash_pair(&proof_element.hash, current_hash)
            };
        }

        // Final hash should match the root
        current_hash == self.hash
    }
}