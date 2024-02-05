# Metrics

This file is documenting and tracking all the metrics which can be collected
by a Prometheus server. The metrics can be scraped by Prometheus from
the `api/v2/node/metrics` API endpoint.

## Example Prometheus configuration

An example scrape config for Prometheus to collect HOPRd metrics:

```yaml
scrape_configs:
  - job_name: 'hoprd'
    scrape_interval: 5s
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: /api/v2/node/metrics
    basic_auth:
      username: ^MYtoken4testing^
      password: ''
```

## List of metrics
- `hopr_packets_count`: Number of processed packets of different types (sent, received, forwarded), keys: `type`
- `hopr_packets_per_peer_count`: Number of processed packets to/from distinct peers, keys: `peer`, `direction`
- `hopr_created_tickets_count`: Number of created tickets
- `hopr_rejected_tickets_count`: Number of rejected tickets
- `hopr_mixer_queue_size`: Current mixer queue size
- `hopr_mixer_average_packet_delay`: Average mixer packet delay averaged over a packet window
- `hopr_aggregated_tickets_count`: Number of aggregated tickets
- `hopr_aggregations_count`: Number of performed ticket aggregations
- `hopr_received_ack_count`: Number of received acknowledgements, keys: `valid`
- `hopr_sent_acks_count`: Number of sent message acknowledgements
- `hopr_winning_tickets_count`: Number of winning tickets
- `hopr_losing_tickets_count`: Number of losing tickets
- `hopr_ping_time_sec`: Measures total time it takes to ping a single node (seconds), buckets: 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0
- `hopr_heartbeat_pings_count`: Total number of pings by result, keys: `success`
- `hopr_heartbeat_round_time_sec`: Measures total time in seconds it takes to probe all other nodes, buckets: 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0
- `hopr_network_health`: Connectivity health indicator
- `hopr_relayed_packet_processing_time_with_mixing_sec`: Histogram of measured processing and mixing time for a relayed packet in seconds, buckets: 0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
- `hopr_peers_by_quality`: Number different peer types by quality
- `hopr_peer_count`: Number of all peers
- `hopr_time_to_green_sec`: Total number of pings by result, keys: `success`
- `hopr_channels_count`: Number of channels per direction, keys: `direction`
- `hopr_channel_balance`: Balances on channels per counterparty, keys: `counterparty`, `direction`
- `hopr_path_length`: Distribution of number of hops of sent messages, buckets: 0, 1, 2, 3, 4
- `hopr_strategy_closure_auto_finalization_count`: Count of channels where closure finalizing was initiated automatically
- `hopr_strategy_enabled_strategies`: List of enabled strategies, keys: `strategy`
- `hopr_strategy_auto_funding_funding_count`: Count of initiated automatic fundings
- `hopr_strategy_promiscuous_opened_channels_count`: Count of open channel decisions
- `hopr_strategy_promiscuous_closed_channels_count`: Count of close channel decisions
- `hopr_strategy_promiscuous_max_auto_channels`: Count of maximum number of channels managed by the strategy
- `hopr_strategy_auto_redeem_redeem_count`: Count of initiated automatic redemptions
- `hopr_strategy_aggregating_aggregation_count`: Count of initiated automatic aggregations
- `hopr_http_api_call_count`: Number of different REST API calls and their statuses, keys: `endpoint`, `method`, `status`
- `hopr_http_api_call_timing_sec`: Timing of different REST API calls in seconds, keys: `endpoint`, `method`, buckets: 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0
- `hopr_message_latency_sec`: Histogram of measured received message latencies in seconds, buckets: 0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0
- `hopr_failed_send_message_count`: Number of sent messages failures
- `hopr_up`: The unix timestamp in seconds at which the process was started
- `hopr_lib_version`: Executed version of hopr-lib, keys: `version`
- `hopr_node_addresses`: Node on-chain and off-chain addresses, keys: `peerid`, `address`, `safe_address`, `module_address`
- `hopr_rpc_call_count`: Number of Ethereum RPC calls over HTTP and their result, key: `call`, `result`
- `hopr_rpc_call_time_sec`: Timing of RPC calls over HTTP in seconds, keys: `call`, buckets: 0.1, 0.5, 1.0, 2.0, 5.0, 7.0, 10.0
- `hopr_retries_per_rpc_call`: Number of retries per RPC call, keys: `call`, buckets: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
- `hopr_chain_head_block_number`: Current block number of chain head
- `hopr_indexer_block_number`: Current last processed block number by the indexer
- `hopr_indexer_sync_progress`: Sync progress of the historical data by the indexer
- `hopr_chain_actions_count`: Number of different chain actions and their results, keys: `action`, `result`
