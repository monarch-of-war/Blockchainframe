use blockchain_core::block::Block;


pub struct Validator;


impl Validator {
    pub fn validate_block(prev_block: &Block, new_block: &Block) -> bool {
        // Example validation logic: check if the new block's previous hash matches the previous block's hash
        new_block.previous_hash == prev_block.hash &&
        new_block.height == prev_block.height + 1
    }
}