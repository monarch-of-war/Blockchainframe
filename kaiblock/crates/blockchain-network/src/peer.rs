use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Peer{
    pub add: SocketAddr,
}


impl Peer{
    pub fn new(addr: SocketAddr) -Self{
        Self{
            add: addr,
        }
    }
}