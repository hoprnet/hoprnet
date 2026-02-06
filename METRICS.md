# Metrics

This file is documenting and tracking all the metrics which can be collected
by a Prometheus server. The metrics can be scraped by Prometheus from
the `api/v4/node/metrics` API endpoint.

## Example Prometheus configuration

An example scrape config for Prometheus to collect HOPRd metrics:

```yaml
scrape_configs:
  - job_name: "hoprd"
    scrape_interval: 5s
    static_configs:
      - targets: ["localhost:3001"]
    metrics_path: /api/v4/node/metrics
    basic_auth:
      username: ^MYtoken4testing^
      password: ""
```

## List of metrics

- `hopr_packets_count`: Number of processed packets of different types (sent, received, forwarded), keys: `type`
- `hopr_mixer_queue_size`: Current mixer queue size
- `hopr_mixer_average_packet_delay`: Average mixer packet delay averaged over a packet window
- `hopr_network_health`: Connectivity health indicator
- `hopr_peer_count`: Number of all peers
- `hopr_path_length`: Distribution of number of hops of sent messages, buckets: 0, 1, 2, 3, 4
- `hopr_strategy_closure_auto_finalization_count`: Count of channels where closure finalizing was initiated automatically
- `hopr_strategy_enabled_strategies`: List of enabled strategies, keys: `strategy`
- `hopr_strategy_auto_funding_funding_count`: Count of initiated automatic fundings
- `hopr_strategy_auto_redeem_redeem_count`: Count of initiated automatic redemptions
- `hopr_transport_p2p_active_connection_count`: Count of the currently active p2p connections as observed from the rust-libp2p events
- `hopr_http_api_call_count`: Number of different REST API calls and their statuses, keys: `path`, `method`, `status`
- `hopr_http_api_call_timing_sec`: Timing of different REST API calls in seconds, keys: `path`, `method`, buckets: 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0
- `hopr_http_api_last_used_time`: The unix timestamp in seconds at which any API endpoint was last fetched
- `hopr_start_time`: The unix timestamp in seconds at which the process was started
- `hopr_lib_version`: Executed version of hopr-lib, keys: `version`
- `hopr_node_addresses`: Node on-chain and off-chain addresses, keys: `peerid`, `address`, `safe_address`, `module_address`
- `hopr_session_established_sessions_count`: Number of sessions that were successfully established as an Exit node
- `hopr_session_num_active_sessions`: Number of currently active HOPR sessions
- `hopr_session_received_error_count`: Number of HOPR session errors received from an Exit node, keys: `kind`
- `hopr_session_sent_error_count`: Number of HOPR session errors sent to an Entry node, keys: `kind`
- `hopr_session_initiated_sessions_count`: Number of sessions that were successfully initiated as an Entry node
- `hopr_session_hoprd_clients`: Number of clients connected at this Entry node, keys: `type`
- `hopr_session_hoprd_target_connections`: Number of currently active HOPR session target connections from this Exit node, keys: `type`
- `hopr_session_time_to_finish_frame`: Measures time in milliseconds it takes a frame to be reassembled, buckets: 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0
- `hopr_surb_balancer_target_error_estimate`: Target error estimation by the SURB balancer, keys: `session_id`
- `hopr_surb_balancer_control_output`: Control output of the SURB balancer, keys: `session_id`
- `hopr_surb_balancer_surbs_rate`: Estimation of SURB rate per second (positive is buffer surplus, negative is buffer loss), keys: `session_id`
- `hopr_surb_balancer_current_buffer_estimate`: Estimated number of SURBs in the buffer, keys: `session_id`
- `hopr_surb_balancer_current_buffer_target`: Current target (setpoint) number of SURBs in the buffer, keys: `session_id`
- `hopr_tickets_incoming_statistics`: Ticket statistics with incoming tickets, keys: `statistic`
- `hopr_udp_ingress_packet_len`: UDP packet lengths on ingress, buckets: 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0
- `hopr_udp_egress_packet_len`: UDP packet lengths on egress, buckets: 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0

## CPU Parallelization Metrics (Rayon)

These metrics track the Rayon thread pool used for CPU-intensive cryptographic operations (EC multiplication, ECDSA signing/verification, packet decoding, ticket verification).

### Task Lifecycle Metrics

- `hopr_rayon_tasks_submitted_total`: Total number of tasks submitted to the Rayon thread pool
- `hopr_rayon_tasks_completed_total`: Total number of tasks that completed successfully and delivered results to a live receiver
- `hopr_rayon_tasks_cancelled_total`: Total number of tasks skipped via cooperative cancellation (receiver dropped while task was queued)
- `hopr_rayon_tasks_orphaned_total`: Total number of tasks whose results were discarded after completion (receiver dropped during execution)
- `hopr_rayon_tasks_rejected_total`: Total number of tasks rejected due to queue being full (only occurs when `HOPR_CPU_TASK_QUEUE_LIMIT` is set)

### Performance Metrics

- `hopr_rayon_queue_wait_seconds`: Histogram of time tasks spend waiting in queue before execution starts
  - Buckets: 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0 seconds
  - High P95/P99 values indicate queue depth issues

- `hopr_rayon_execution_seconds`: Histogram of task execution duration, labeled by operation type
  - Keys: `operation` (values: `peerid_lookup`, `packet_decode`, `ticket_verify`, `ack_verify`)
  - Buckets: 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0 seconds
  - Tracks actual CPU time spent in crypto operations

### Queue State Metrics

- `hopr_rayon_outstanding_tasks`: Current number of tasks either queued or running in the Rayon pool (gauge)
- `hopr_rayon_queue_limit`: Configured maximum outstanding tasks from `HOPR_CPU_TASK_QUEUE_LIMIT` environment variable (0 if unlimited)

### Troubleshooting Rayon Pool Saturation

**Symptom**: `hopr_packet_decode_timeouts_total` is increasing

**Diagnosis Steps**:

1. **Check queue wait time**: Look at `hopr_rayon_queue_wait_seconds` P95/P99
   - If > 100ms: Queue is backed up, tasks waiting too long
   - Indicates insufficient CPU capacity or too many concurrent packets

2. **Check execution time**: Review `hopr_rayon_execution_seconds{operation="packet_decode"}` P95
   - If > 50ms: Individual crypto operations are slow
   - May indicate CPU throttling or resource contention

3. **Check task cancellations**: Rising `hopr_rayon_tasks_cancelled_total`
   - High rate confirms tasks timing out before execution
   - Indicates decode timeout (150ms) is too aggressive for current load

4. **Check queue depth**: Monitor `hopr_rayon_outstanding_tasks`
   - Sustained high values (>100) indicate backlog
   - Consider setting `HOPR_CPU_TASK_QUEUE_LIMIT` for backpressure

**Mitigation**:

- Set `HOPR_CPU_TASK_QUEUE_LIMIT` (e.g., 1000) to apply backpressure upstream
- Reduce `HOPR_TRANSPORT_MAX_CONCURRENT_PACKETS` to limit incoming load
- Increase CPU allocation to the hoprd process

## Unacknowledged Ticket Cache Metrics

These metrics help diagnose "unknown ticket" acknowledgement failures by tracking the in-memory cache of tickets awaiting acknowledgement.

### Cache State Metrics

- `hopr_tickets_unack_peers_total`: Number of unique peers with unacknowledged tickets in cache (gauge)
- `hopr_tickets_unack_tickets_total`: Total number of unacknowledged tickets across all peers in cache (gauge)

### Cache Operations Metrics

- `hopr_tickets_unack_insertions_total`: Total number of tickets inserted into the unacknowledged cache (counter)
- `hopr_tickets_unack_lookups_total`: Total number of acknowledgement lookups attempted (counter)
- `hopr_tickets_unack_lookup_misses_total`: Total number of lookup failures - "unknown ticket" errors (counter)
- `hopr_tickets_unack_evictions_total`: Total number of tickets evicted from cache due to TTL or capacity limits (counter)

### Optional Per-Peer Metrics

- `hopr_tickets_unack_tickets_per_peer`: Number of unacknowledged tickets per peer
  - Keys: `peer` (PeerID of the peer)
  - **DISABLED BY DEFAULT** to avoid Prometheus cardinality explosion
  - Enable with `HOPR_METRICS_UNACK_PER_PEER=1` environment variable
  - **Warning**: Only enable for debugging specific peers, not in production with many peers

### Troubleshooting Unknown Ticket Errors

**Symptom**: `hopr_tickets_unack_lookup_misses_total` is increasing

**Diagnosis Steps**:

1. **Check eviction rate**: Compare `hopr_tickets_unack_evictions_total` vs `hopr_tickets_unack_insertions_total`
   - High eviction rate suggests TTL too short or cache capacity too small
   - Tickets may be evicted before acknowledgements arrive

2. **Check cache size**: Monitor `hopr_tickets_unack_tickets_total` and `hopr_tickets_unack_peers_total`
   - Low values during high traffic suggest aggressive eviction
   - May need to increase cache capacity

3. **Identify problem peers**: Enable per-peer metrics temporarily
   - Set `HOPR_METRICS_UNACK_PER_PEER=1`
   - Check `hopr_tickets_unack_tickets_per_peer` to identify peers with many unacked tickets
   - **Remember to disable** after debugging to avoid cardinality issues

**Mitigation**:

- Review cache TTL and capacity settings in ticket processing code
- Investigate network latency to specific peers (acknowledgements arriving late)
- Check if peer is malicious (not sending acknowledgements intentionally)

## Protocol Pipeline Metrics

- `hopr_packet_decode_timeouts_total`: Number of incoming packets dropped due to decode timeout (150ms threshold)
  - A sustained non-zero rate indicates Rayon pool saturation
  - See "Troubleshooting Rayon Pool Saturation" section above for diagnosis
