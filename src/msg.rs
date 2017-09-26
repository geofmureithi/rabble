use cluster::ClusterStatus;
use correlation_id::CorrelationId;
use metrics::Metric;

type Name = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg<T> {
    User(T),
    ClusterStatus(ClusterStatus),
    StartTimer(usize), // time in ms
    CancelTimer(Option<CorrelationId>),
    Timeout,
    Shutdown,
    GetMetrics,
    Metrics(Vec<(Name, Metric)>)
}
