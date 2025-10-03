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
- `hopr_replayed_packet_count`: The total count of replayed packets during the packet processing pipeline run
- `hopr_mixer_queue_size`: Current mixer queue size
- `hopr_mixer_average_packet_delay`: Average mixer packet delay averaged over a packet window
- `hopr_aggregated_tickets_count`: Number of aggregated tickets
- `hopr_aggregations_count`: Number of performed ticket aggregations
- `hopr_received_ack_count`: Number of received acknowledgements, keys: `valid`
- `hopr_sent_acks_count`: Number of sent message acknowledgements
- `hopr_tickets_count`: Number of tickets (winning, losing, rejected), keys: `type`
- `hopr_ping_time_sec`: Measures total time it takes to ping a single node (seconds), buckets: 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0
- `hopr_probe_count`: Total number of probes by result, keys: `success`
- `hopr_heartbeat_round_time_sec`: Measures total time in seconds it takes to probe all other nodes, buckets: 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0
- `hopr_network_health`: Connectivity health indicator
- `hopr_relayed_packet_processing_time_with_mixing_sec`: Histogram of measured processing and mixing time for a relayed packet in seconds, buckets: 0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
- `hopr_peers_by_quality`: Number different peer types by quality
- `hopr_peer_count`: Number of all peers
- `hopr_time_to_green_sec`: Total number of pings by result, keys: `success`
- `hopr_channels_count`: Number of channels per direction, keys: `direction`
- `hopr_path_length`: Distribution of number of hops of sent messages, buckets: 0, 1, 2, 3, 4
- `hopr_strategy_closure_auto_finalization_count`: Count of channels where closure finalizing was initiated automatically
- `hopr_strategy_enabled_strategies`: List of enabled strategies, keys: `strategy`
- `hopr_strategy_auto_funding_funding_count`: Count of initiated automatic fundings
- `hopr_strategy_promiscuous_opened_channels_count`: Count of open channel decisions
- `hopr_strategy_promiscuous_closed_channels_count`: Count of close channel decisions
- `hopr_strategy_promiscuous_max_auto_channels`: Count of maximum number of channels managed by the strategy
- `hopr_strategy_auto_redeem_redeem_count`: Count of initiated automatic redemptions
- `hopr_strategy_aggregating_aggregation_count`: Count of initiated automatic aggregations
- `hopr_transport_p2p_opened_connection_count`: Count of the currently active p2p connections as observed from the rust-libp2p events
- `hopr_http_api_call_count`: Number of different REST API calls and their statuses, keys: `path`, `method`, `status`
- `hopr_http_api_call_timing_sec`: Timing of different REST API calls in seconds, keys: `path`, `method`, buckets: 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0
- `hopr_http_api_last_used_time`: The unix timestamp in seconds at which any API endpoint was last fetched
- `hopr_start_time`: The unix timestamp in seconds at which the process was started
- `hopr_lib_version`: Executed version of hopr-lib, keys: `version`
- `hopr_node_addresses`: Node on-chain and off-chain addresses, keys: `peerid`, `address`, `safe_address`, `module_address`
- `hopr_rpc_call_count`: Number of Ethereum RPC calls over HTTP and their result, key: `call`, `result`
- `hopr_rpc_call_time_sec`: Timing of RPC calls over HTTP in seconds, keys: `call`, buckets: 0.1, 0.5, 1.0, 2.0, 5.0, 7.0, 10.0
- `hopr_retries_per_rpc_call`: Number of retries per RPC call, keys: `call`, buckets: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
- `hopr_chain_head_block_number`: Current block number of chain head
- `hopr_indexer_block_number`: Current last processed block number by the indexer
- `hopr_indexer_sync_progress`: Sync progress of the historical data by the indexer
- `hopr_indexer_checksum`: Contains an unsigned integer that represents the low 32-bits of the Indexer checksum.
- `hopr_indexer_data_source`: Current data source of the Indexer, keys: `source`
- `hopr_chain_actions_count`: Number of different chain actions and their results, keys: `action`, `result`
- `hopr_indexer_contract_log_count`: Counts of different HOPR contract logs processed by the Indexer, keys: `contract`
- `hopr_tickets_incoming_statistics`: Ticket statistics with incoming tickets, keys: `statistic`
- `hopr_session_num_active_sessions`: Number of currently active HOPR sessions
- `hopr_session_received_error_count`: Number of HOPR session errors received from an Exit node, keys: `kind`
- `hopr_session_sent_error_count`: Number of HOPR session errors sent to an Entry node, keys: `kind`
- `hopr_session_established_sessions_count`: Number of sessions that were successfully established as an Exit node
- `hopr_session_initiated_sessions_count`: Number of sessions that were successfully initiated as an Entry node
- `hopr_session_hoprd_clients`: Number of clients connected at this Entry node, keys: `type`
- `hopr_session_hoprd_target_connections`: Number of currently active HOPR session target connections from this Exit node, keys: `type`
- `hopr_tickets_incoming_win_probability`: Observes the winning probabilities on incoming tickets, buckets: 0.0, 0.0001, 0.001, 0.01, 0.05, 0.1, 0.15, 0.25, 0.3, 0.5
- `hopr_session_time_to_finish_frame`: Measures time in milliseconds it takes a frame to be reassembled, buckets: 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0
- `hopr_udp_ingress_packet_len`: UDP packet lengths on ingress, buckets: 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0
- `hopr_udp_egress_packet_len`: UDP packet lengths on egress, buckets: 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0
- `hopr_session_inner_sizes`: Sizes of data chunks fed from inner session to HOPR protocol, keys: `session_id`, buckets: 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0
- `hopr_surb_balancer_target_error_estimate`: Target error estimation by the SURB balancer, keys: `session_id`
- `hopr_surb_balancer_control_output`: Control output of the SURB balancer, keys: `session_id`
- `hopr_surb_balancer_surbs_rate`: Estimation of SURB rate per second (positive is buffer surplus, negative is buffer loss), keys: `session_id`
- `hopr_surb_balancer_current_buffer_estimate`: Estimated number of SURBs in the buffer, keys: `session_id`
- `hopr_surb_balancer_current_buffer_target`: Current target (setpoint) number of SURBs in the buffer, keys: `session_id`
