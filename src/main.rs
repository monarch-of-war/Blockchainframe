fn main() {
    println!("Hello, world!");

    let data = String::from("Morris");
    let hashed = hash_sha256(&data);

    println!("{}", hashed);
    
}

use sha2::{Sha256, Digest};
use blake2::{Blake2b512, Digest};




fn hash_blake(data: &[u8])->String{
    let mut hasher = Blake2b512::new();

    hasher.update(data);

    let result = hasher.finalize();

    hex::encode(result)
}
