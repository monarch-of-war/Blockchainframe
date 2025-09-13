use serde::{Serialize, Deserialize};
use blockchain_core::block::Block;
use blockchain_core::transaction::Transaction;


pub enum MessageType{
    Block,
    Transaction,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkMessage{
    pub msg_type: MessageType,
    pub payload: Vec<u8>, // Serialized Block or Transaction
}

impl NetworkMessage{
    pub new_block(block: &Block)-> Self{
        Self{
            msg_type: MessageType::Block,
            payload: bincode::serialize(block).unwrap(),
        }
    }

    oub new_transaction(tx: &Transaction) -> Self{
        Self{
            msg_type: MessageType::Transaction,
            payload: bincode::serialize(tx).unwrap(),
        }
    }
}
