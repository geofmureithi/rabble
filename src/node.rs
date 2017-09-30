use std;
use std::sync::mpsc::{Sender, SendError};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use amy;
use slog;
use node_id::NodeId;
use cluster::ClusterMsg;
use pid::Pid;
use correlation_id::CorrelationId;
use process::Process;
use envelope::Envelope;
use errors::*;
use processes::Processes;

macro_rules! send {
    ($s:ident.$t:ident, $msg:expr, $pid:expr, $errmsg:expr) => {
        if let Err(_) = $s.$t.send($msg) {
            return Err(ErrorKind::SendError($errmsg, $pid).into())
        } else {
            return Ok(());
        }
    }
}

/// A Node represents a way for services to interact with rabble internals.
///
/// The Node api is used by services and their handlers to send messages, get status, join
/// nodes into a cluster, etc...
#[derive(Clone)]
pub struct Node<T> {
    pub id: NodeId,
    pub logger: slog::Logger,
    processes: Processes<T>,
    cluster_tx: Sender<ClusterMsg<T>>
}

impl<'de, T: Serialize + Deserialize<'de> + Debug + Clone> Node<T> {
    /// Create a new node. This function should not be called by the user directly. It is called by
    /// by the user call to `rabble::rouse(..)` that initializes a rabble system for a single node.
    pub fn new(id: NodeId,
               processes: Processes<T>,
               cluster_tx: Sender<ClusterMsg<T>>,
               logger: slog::Logger) -> Node<T> {
        Node {
            id: id,
            processes: processes,
            cluster_tx: cluster_tx,
            logger: logger
        }
    }

    /// Join 1 node to another to form a cluster.
    ///
    /// Node joins are transitive such that if `Node A` joins `Node B` which is already joined with
    /// `Node C`, then `Node A` will become connected to both `Node B` and `Node C`.
    ///
    /// Join's are not immediate. The local member state is updated and the joining node will
    /// continuously try to connect to the remote node so that they can exchange membership
    /// information and participate in peer operations.
    pub fn join(&self, node_id: &NodeId) -> Result<()> {
        send!(self.cluster_tx,
              ClusterMsg::Join(node_id.clone()),
              None,
              format!("ClusterMsg::Join({:?})", *node_id))
    }

    pub fn leave(&self, node_id: &NodeId) -> Result<()> {
        send!(self.cluster_tx,
              ClusterMsg::Leave(node_id.clone()),
              None,
              format!("ClusterMsg::Leave({:?})", *node_id))
    }

    /// Add a process to the processes map that can be sent Envelopes addressed to its pid
    pub fn spawn(&self, pid: Pid, process: Box<Process<T>>) -> Result<()> {
        self.processes.spawn(pid, process).map_err(|_| "Failed to spawn process".into())
    }

    /// Remove a process from the processes map
    pub fn stop(&self, pid: &Pid) -> Result<()> {
        self.processes.kill(pid);
        Ok(())
    }

    /// Register a Service's sender with the processes map so that it can be sent messages addressed to
    /// its pid
    pub fn register_service(&self, pid: Pid, tx: amy::Sender<Envelope<T>>) -> Result<()>
    {
        self.processes.register_service(pid, tx).map_err(|_| "Failed to register service".into())
    }

    /// Send an envelope to the proper receiver
    ///
    /// Send an envelope to the processes map so it gets routed to the appropriate process or
    /// service if the envelope is local. Otherwise send it to the cluster_server so it gtets
    /// forwarded to the correct node.
    ///
    /// Return the envelope if the send fails.
    pub fn send(&mut self, envelope: Envelope<T>) -> std::result::Result<(), Envelope<T>> {
        if envelope.to.node == self.id {
            self.processes.send(envelope)
        } else {
            self.cluster_tx.send(ClusterMsg::Envelope(envelope)).map_err(|SendError(cluster_msg)| {
                if let ClusterMsg::Envelope(envelope) = cluster_msg {
                    return envelope;
                }
                unreachable!();
            })
        }
    }

    /// Get the status of the cluster server
    pub fn cluster_status(&self, correlation_id: CorrelationId) -> Result<()> {
        let to = correlation_id.pid.clone();
        send!(self.cluster_tx,
              ClusterMsg::GetStatus(correlation_id),
              Some(to),
              "ClusterMsg::GetStatus".to_string())
    }

    /// Shutdown the node
    pub fn shutdown(&self) {
        self.cluster_tx.send(ClusterMsg::Shutdown).unwrap();
        self.processes.shutdown();
    }
}
