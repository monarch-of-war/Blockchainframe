use warp::Filter;
use crate::handler::RpcHandler
use blockchain_core::transaction::Transaction;
use std::sync::Arc;


pub RpcServer{
    pub handler: Arc<RpcHandler>,
    pub port: u16,
}



impl RcpServer{
    pub fn new(handler: Arc<RpcHandler>, port: u16) -> Self {
        Self { handler, port }
    }

    pub async fn start(&self) {
        let handler_filter = warp::any().map({
            let handler = self.handler.clone();
            move || handler.clone()
        });


    // GET /block/ latest

    let latest_block = warp::path!("block" / "latest")
    .and(handler_filter.clone())
    and_then(|handler: Arc<Rpcandler>|async move {
        match handler.get_latest_block().await {
            Ok(block) => Ok(warp::reply::json(&block)),
            Err(_) => Err(warp::reject::not_found()),
        }

    });

    let block_by_height = warp::path!("block" / u64)
    .and(handler_filter.clone())
    .and_then(|height: u64, handler: Arc<Rpcandler>| async move {
        match handler.get_block_by_height(height).await {
            Ok(block) => Ok(warp::reply::json(&block)),
            Err(_) => Err(warp::reject::not_found()),
        });


    // POST /transaction
    let submit_tx = warp::path("transaction")
    .and(warp::post())
    .and(warp::body::json())
    .and(handler_filter.clone())
    .and_then(|tx: Transaction, handler: Arc<Rpcandler>| async move {
       handler.submit_tx(tx).await.map_err(|_| warp::reject())
       .map(|_| warp::reply::with_status("Transaction submitted", warp::http::StatusCode::OK))

    });


    let routes = latest_block.or(block_by_height).or(submit_tx);
    println!("RPC server listening on port {}", self.port);
    warp::serve(routes).run(([127,0,0,1], self.port)).await;


}

}