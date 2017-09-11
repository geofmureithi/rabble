use envelope::Envelope;
use correlation_id::CorrelationId;
use pid::Pid;

/// A ConnectionHandler denotes the endpoint of a single connection in a network server
///
/// Implement this for a specific connection handler
pub trait ConnectionHandler: Sized {
    type ClientMsg;

    fn new(pid: Pid, id: u64) -> Self;
    fn handle_envelope(&mut self, Envelope, &mut Vec<ConnectionMsg<Self>>);
    fn handle_network_msg(&mut self, Self::ClientMsg, &mut Vec<ConnectionMsg<Self>>);
}

/// Connection messages are returned from the callback functions for a Connection.
///
/// These messages can be either an envelope as gets used in the rest of the system or a message
/// specific to this service that can be serialized and sent to a client on the other end of the
/// connection.
pub enum ConnectionMsg<C: ConnectionHandler>
{
    Envelope(Envelope),
    Client(C::ClientMsg, CorrelationId)
}
