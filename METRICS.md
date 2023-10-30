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

## Package metrics

The following sections document the metrics per package:

### core

| Name                                                | Type        | Description                                                            | Note                                                                      |
| --------------------------------------------------- | ----------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `core_gauge_num_outgoing_channels`                  | gauge       | Number of outgoing channels                                            |                                                                           |
| `core_gauge_num_incoming_channels`                  | gauge       | Number of incoming channels                                            |                                                                           |
| `core_counter_sent_messages`                        | counter     | Number of sent messages                                                |                                                                           |
| `core_counter_failed_send_messages`                 | counter     | Number of send message failures                                        |                                                                           |
| `core_histogram_path_length`                        | histogram   | Distribution of number of hops of sent messages                        | buckets: 0-4                                                              |
| `core_counter_received_successful_acks`             | counter     | Number of received successful message acknowledgements                 |                                                                           |
| `core_counter_received_failed_acks`                 | counter     | Number of received failed message acknowledgements                     |                                                                           |
| `core_counter_sent_acks`                            | counter     | Number of sent message acknowledgements                                |                                                                           |
| `core_counter_winning_tickets`                      | counter     | Number of winning tickets                                              |                                                                           |
| `core_counter_losing_tickets`                       | counter     | Number of losing tickets                                               |                                                                           |
| `core_counter_forwarded_messages`                   | counter     | Number of forwarded messages                                           |                                                                           |
| `core_counter_received_messages`                    | counter     | Number of received messages                                            |                                                                           |
| `core_counter_created_tickets`                      | counter     | Number of created tickets                                              |                                                                           |
| `core_counter_packets`                              | counter     | Number of created packets                                              |                                                                           |
| `core_counter_nr_rejected_conns`                    | counter     | Number of rejected connections due to NR                               |                                                                           |
| `core_gauge_network_health`                         | gauge       | Connectivity health indicator                                          | 0 = UNKNOWN, 4 = GREEN                                                    |
| `core_histogram_heartbeat_time_seconds`             | histogram   | Measures total time it takes to probe other nodes (in seconds)         | unit: seconds                                                             |
| `core_counter_heartbeat_successful_pings`           | counter     | Total number of successful pings                                       |                                                                           |
| `core_counter_heartbeat_failed_pings`               | counter     | Total number of failed pings                                           |                                                                           |
| `core_gauge_num_high_quality_peers`                 | gauge       | Number of high quality peers                                           | quality > 0.5                                                             |
| `core_gauge_num_low_quality_peers`                  | gauge       | Number of low quality peers                                            | quality <= 0.5                                                            |
| `core_gauge_num_peers`                              | gauge       | Number of all peers                                                    |                                                                           |
| `core_histogram_ping_time_seconds`                  | histogram   | Measures total time it takes to ping a single node (seconds)           | unit: seconds                                                             |
| `core_mgauge_channel_balances`                      | multi_gauge | Balances on channels with counterparties                               | labels: counterparty, direction                                           |
| `core_counter_strategy_ticks`                       | counter     | Number of strategy decisions (ticks)                                   |                                                                           |
| `core_gauge_strategy_last_opened_channels`          | gauge       | Number of opened channels in the last strategy tick                    |                                                                           |
| `core_gauge_strategy_last_closed_channels`          | gauge       | Number of closed channels in the last strategy tick                    |                                                                           |
| `core_gauge_strategy_max_auto_channels`             | gauge       | Maximum number of channels the current strategy can open               |                                                                           |
| `core_counter_aggregated_tickets`                   | counter     | Number of aggregated tickets                                           |                                                                           |
| `core_counter_aggregations`                         | counter     | Number of performed ticket aggregations                                |                                                                           |
| `core_counter_strategy_aggregating_aggregations`    | counter     | Count of initiated automatic aggregations                              |                                                                           |
| `core_counter_strategy_auto_funding_fundings`       | counter     | Count of initiated automatic fundings                                  |                                                                           |
| `core_counter_strategy_auto_redeem_redeems`         | counter     | Count of initiated automatic redemptions                               |                                                                           |
| `core_counter_strategy_promiscuous_opened_channels` | counter     | Count of open channel decisions                                        |                                                                           |
| `core_counter_strategy_promiscuous_closed_channels` | counter     | Count of close channel decisions                                       |                                                                           |
| `core_counter_strategy_count_closure_finalization`  | counter     | Count of channels where closure finalizing was initiated automatically |                                                                           |
| `core_multi_gauge_strategy_enabled_strategies`      | multi_gauge | List of enabled strategies                                             | labels: promiscuous,aggregating,auto_redeeming,auto_funding,multi,passive |

### core-ethereum

| Name                                                         | Type          | Description                                 | Note                                                                |
| ------------------------------------------------------------ | ------------- | ------------------------------------------- | ------------------------------------------------------------------- |
| `core_ethereum_mcounter_indexer_provider_errors`             | multi counter | Multicounter for provider errors in Indexer |                                                                     |
| `core_ethereum_counter_indexer_processed_unconfirmed_blocks` | counter       | Number of processed unconfirmed blocks      |                                                                     |
| `core_ethereum_counter_indexer_announcements`                | counter       | Number of processed announcements           |                                                                     |
| `core_ethereum_gauge_indexer_block_number`                   | gauge         | Current block number                        |                                                                     |
| `core_ethereum_gauge_indexer_channel_status`                 | multi gauge   | Status of different channels                | 0 = closed, 1 = waiting for commitment, 2 = open, 3 = pending close |
| `core_ethereum_counter_indexer_tickets_redeemed`             | counter       | Number of redeemed tickets                  |                                                                     |
| `core_ethereum_counter_num_send_transactions`                | counter       | The number of sendTransaction calls         |                                                                     |
| `core_ethereum_counter_successful_transactions`              | counter       | Number of successful transactions           |                                                                     |
| `core_ethereum_counter_failed_transactions`                  | counter       | Number of failed transactions               |                                                                     |
| `core_ethereum_counter_timeout_transactions`                 | counter       | Number of timed out transactions            |                                                                     |

### ethereum

### hoprd

| Name                                            | Type        | Description                                                       | Note                                     |
| ----------------------------------------------- | ----------- | ----------------------------------------------------------------- | ---------------------------------------- |
| `hoprd_gauge_startup_unix_time_seconds`         | gauge       | The unix timestamp at which the process was started               | seconds since Epoch                      |
| `hoprd_histogram_startup_time_seconds`          | histogram   | Time it takes for a node to start up                              | unit: seconds                            |
| `hoprd_histogram_time_to_green_seconds`         | histogram   | Time it takes for a node to transition to the GREEN network state | unit: seconds                            |
| `hoprd_histogram_message_latency_ms`            | histogram   | Histogram of measured received message latencies                  | unit: milliseconds                       |
| `hoprd_mgauge_version`                          | multi gauge | Executed version of HOPRd                                         |                                          |
| `hoprd_gauge_nodejs_total_alloc_heap_bytes`     | gauge       | V8 allocated total heap size in bytes                             | unit: bytes                              |
| `hoprd_gauge_nodejs_total_used_heap_bytes`      | gauge       | V8 used heap size in bytes                                        | unit: bytes                              |
| `hoprd_gauge_nodejs_total_available_heap_bytes` | gauge       | V8 total available heap size in bytes                             | unit: bytes                              |
| `hoprd_gauge_nodejs_num_native_contexts`        | gauge       | V8 number of active top-level native contexts                     | unit: bytes, increase indicates mem leak |
| `hoprd_gauge_nodejs_num_detached_contexts`      | gauge       | V8 number of detached contexts which are not GCd                  | unit: bytes, non-zero indicates mem leak |
| `hoprd_counter_api_successful_send_msg`         | counter     | Number of successful API calls to POST message endpoint           |                                          |
| `hoprd_counter_api_failed_send_msg`             | counter     | Number of failed API calls to POST message endpoint               |                                          |

### utils

| Name                                                    | Type    | Description                                        | Note |
| ------------------------------------------------------- | ------- | -------------------------------------------------- | ---- |
| `utils_counter_suppressed_unhandled_promise_rejections` | counter | Counter of suppressed unhandled promise rejections |      |
