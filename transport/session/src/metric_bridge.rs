use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, Weak},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::{
    AckSnapshot, FrameBufferSnapshot, SessionAckMode, SessionId, SessionLifecycleState, SessionLifetimeSnapshot,
    SessionStatsSnapshot, SessionTelemetry, SurbSnapshot, TransportSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionMetricKind {
    U64Gauge,
    U64Counter,
    F64Gauge,
}

impl SessionMetricKind {
    fn from_name(metric_name: &str) -> Self {
        if metric_name.ends_with("_total") {
            SessionMetricKind::U64Counter
        } else {
            SessionMetricKind::U64Gauge
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionMetricDefinition {
    pub name: String,
    pub kind: SessionMetricKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionMetricValue {
    U64(u64),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionMetricSample {
    pub name: String,
    pub value: SessionMetricValue,
}

pub fn serialize_system_time_millis<S>(timestamp: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let timestamp_millis = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    serializer.serialize_u64(timestamp_millis)
}

pub fn serialize_duration_millis<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_millis() as u64)
}

fn flatten_snapshot_value(value: &serde_json::Value, path: &mut Vec<String>, output: &mut Vec<SessionMetricSample>) {
    match value {
        serde_json::Value::Object(fields) => {
            for (field_name, field_value) in fields {
                if path.is_empty() && field_name == "session_id" {
                    continue;
                }
                path.push(field_name.clone());
                flatten_snapshot_value(field_value, path, output);
                path.pop();
            }
        }
        serde_json::Value::Number(number) => {
            let metric_name = format!("hopr_session_{}", path.join("_"));
            if let Some(value) = number.as_u64() {
                output.push(SessionMetricSample {
                    name: metric_name,
                    value: SessionMetricValue::U64(value),
                });
            } else if let Some(value) = number.as_i64() {
                output.push(SessionMetricSample {
                    name: metric_name,
                    value: SessionMetricValue::U64(value.max(0) as u64),
                });
            } else if let Some(value) = number.as_f64() {
                output.push(SessionMetricSample {
                    name: metric_name,
                    value: SessionMetricValue::F64(value),
                });
            }
        }
        serde_json::Value::Bool(value) => {
            let metric_name = format!("hopr_session_{}", path.join("_"));
            output.push(SessionMetricSample {
                name: metric_name,
                value: SessionMetricValue::U64(u64::from(*value)),
            });
        }
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::String(_) => {}
    }
}

fn collect_snapshot_metrics(snapshot: &SessionStatsSnapshot) -> Vec<SessionMetricSample> {
    let mut output = Vec::new();
    let mut path = Vec::new();

    if let Ok(serialized) = serde_json::to_value(snapshot) {
        flatten_snapshot_value(&serialized, &mut path, &mut output);
    }

    output
}

#[derive(serde::Serialize)]
struct SessionMetricSchemaSnapshot {
    #[serde(rename = "snapshot_at_ms")]
    #[serde(serialize_with = "serialize_system_time_millis")]
    snapshot_at: SystemTime,
    lifetime: SessionLifetimeSnapshot,
    #[serde(rename = "frame")]
    frame_buffer: FrameBufferSnapshot,
    ack: AckSnapshot,
    surb: SurbSnapshot,
    transport: TransportSnapshot,
}

fn metric_schema_snapshot() -> SessionMetricSchemaSnapshot {
    SessionMetricSchemaSnapshot {
        snapshot_at: UNIX_EPOCH,
        lifetime: SessionLifetimeSnapshot {
            created_at: UNIX_EPOCH,
            last_activity_at: UNIX_EPOCH,
            uptime: Duration::from_millis(0),
            idle: Duration::from_millis(0),
            state: SessionLifecycleState::Active,
            pipeline_errors: 0,
        },
        frame_buffer: FrameBufferSnapshot {
            frame_mtu: 0,
            frame_timeout: Duration::from_millis(0),
            frame_capacity: 0,
            frames_being_assembled: 0,
            frames_completed: 0,
            frames_emitted: 0,
            frames_discarded: 0,
        },
        ack: AckSnapshot {
            mode: SessionAckMode::None,
            incoming_segments: 0,
            incoming_retransmission_requests: 0,
            incoming_acknowledged_frames: 0,
            outgoing_segments: 0,
            outgoing_retransmission_requests: 0,
            outgoing_acknowledged_frames: 0,
        },
        surb: SurbSnapshot {
            produced_total: 0,
            consumed_total: 0,
            buffer_estimate: 0,
            target_buffer: Some(0),
            rate_per_sec: 0.0,
            refill_in_flight: false,
        },
        transport: TransportSnapshot {
            bytes_in: 0,
            bytes_out: 0,
            packets_in: 0,
            packets_out: 0,
        },
    }
}

pub fn session_snapshot_metric_definitions() -> Vec<SessionMetricDefinition> {
    let mut definitions = Vec::new();
    let mut path = Vec::new();

    if let Ok(serialized) = serde_json::to_value(metric_schema_snapshot()) {
        let mut samples = Vec::new();
        flatten_snapshot_value(&serialized, &mut path, &mut samples);
        for sample in samples {
            let kind = match sample.value {
                SessionMetricValue::U64(_) => SessionMetricKind::from_name(&sample.name),
                SessionMetricValue::F64(_) => SessionMetricKind::F64Gauge,
            };
            definitions.push(SessionMetricDefinition {
                name: sample.name,
                kind,
            });
        }
    }

    definitions
}

pub fn session_snapshot_metric_value(snapshot: &SessionStatsSnapshot, metric_name: &str) -> Option<SessionMetricValue> {
    collect_snapshot_metrics(snapshot)
        .into_iter()
        .find(|sample| sample.name == metric_name)
        .map(|sample| sample.value)
}

pub fn session_snapshot_metric_samples(snapshot: &SessionStatsSnapshot) -> Vec<SessionMetricSample> {
    collect_snapshot_metrics(snapshot)
}

fn session_telemetry_registry() -> &'static parking_lot::Mutex<HashMap<SessionId, Weak<SessionTelemetry>>> {
    static REGISTRY: OnceLock<parking_lot::Mutex<HashMap<SessionId, Weak<SessionTelemetry>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| parking_lot::Mutex::new(HashMap::new()))
}

pub(crate) fn register_session_telemetry(session_telemetry: &Arc<SessionTelemetry>) {
    session_telemetry_registry()
        .lock()
        .insert(*session_telemetry.session_id(), Arc::downgrade(session_telemetry));
}

pub(crate) fn unregister_session_telemetry(session_id: &SessionId) {
    session_telemetry_registry().lock().remove(session_id);
}

pub fn session_telemetry_snapshots() -> Vec<SessionStatsSnapshot> {
    let mut registry = session_telemetry_registry().lock();
    let mut stale_sessions = Vec::new();
    let mut snapshots = Vec::with_capacity(registry.len());

    for (session_id, session_telemetry) in registry.iter() {
        if let Some(session_telemetry) = session_telemetry.upgrade() {
            snapshots.push(session_telemetry.snapshot());
        } else {
            stale_sessions.push(*session_id);
        }
    }

    for session_id in stale_sessions {
        registry.remove(&session_id);
    }

    snapshots.sort_by(|left, right| left.session_id.as_str().cmp(right.session_id.as_str()));
    snapshots
}
