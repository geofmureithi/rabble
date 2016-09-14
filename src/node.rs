use std::sync::mpsc::Sender;
use rustc_serialize::{Encodable, Decodable};
use node_id::NodeId;
use executor_msg::ExecutorMsg;
use cluster_msg::ClusterMsg;

pub struct Node<T: Encodable + Decodable, U> {
    pub id: NodeId,
    executor_tx: Sender<ExecutorMsg<T, U>>,
    cluster_tx: Sender<ClusterMsg<T>>
}

impl<T: Encodable + Decodable, U> Node<T, U> {
    pub fn new(id: NodeId,
               executor_tx: Sender<ExecutorMsg<T, U>>,
               cluster_tx: Sender<ClusterMsg<T>>) -> Node<T, U> {
        Node {
            id: id,
            executor_tx: executor_tx,
            cluster_tx: cluster_tx
        }
    }
}
