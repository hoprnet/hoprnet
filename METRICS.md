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

### connect

| Name                                       | Type    | Description                                                   | Note                       |
| ------------------------------------------ | ------- | ------------------------------------------------------------- | :------------------------- |
| `connect_counter_successful_direct_dials`  | counter | Number of successful direct dials.                            |                            |
| `connect_counter_failed_direct_dials`      | counter | Number of failed direct dials                                 |                            |
| `connect_counter_successful_relayed_dials` | counter | Number of successful relayed dials                            |                            |
| `connect_counter_failed_relayed_dials`     | counter | Number of failed relayed dials                                |                            |
| `connect_gauge_used_relays`                | gauge   | Number of used relays                                         |                            |
| `connect_gauge_conns_to_relays`            | gauge   | Number of connections to relays                               |                            |
| `connect_gauge_relayed_conns`              | gauge   | Number of currently relayed connections                       |                            |
| `connect_counter_successful_relay_reqs`    | counter | Number of successful incoming relay requests                  |                            |
| `connect_counter_failed_relay_reqs`        | counter | Number of failed incoming relay requests                      |                            |
| `connect_counter_relay_reconnects`         | counter | Number of re-established relayed connections                  |                            |
| `connect_counter_successful_conns`         | counter | Number of successful connection attempts                      |                            |
| `connect_counter_failed_conns`             | counter | Number of failed connection attempts                          |                            |
| `connect_counter_udp_stun_requests`        | counter | Number of UDP STUN requests                                   |                            |
| `connect_counter_tcp_stun_requests`        | counter | Number of TCP STUN requests                                   |                            |
| `connect_gauge_node_is_exposed`            | gauge   | Shows whether a node believes that it runs on an exposed host | 1: exposed, 0: not exposed |
| `connect_counter_server_relayed_packets`   | counter | Number of relayed packets (TURN server)                       |                            |
| `connect_counter_client_relayed_packets`   | counter | Number of relayed packets (TURN client)                       |                            |
| `connect_counter_direct_packets`           | counter | Number of directly sent packets (TCP)                         |                            |
| `connect_counter_webrtc_packets`           | counter | Number of directly sent packets (WebRTC)                      |                            |

### core

| Name                                      | Type        | Description                                                    | Note                            |
| ----------------------------------------- | ----------- | -------------------------------------------------------------- | ------------------------------- |
| `core_gauge_num_outgoing_channels`        | gauge       | Number of outgoing channels                                    |                                 |
| `core_gauge_num_incoming_channels`        | gauge       | Number of incoming channels                                    |                                 |
| `core_counter_sent_messages`              | counter     | Number of sent messages                                        |                                 |
| `core_histogram_path_length`              | histogram   | Distribution of number of hops of sent messages                | buckets: 0-4                    |
| `core_counter_received_successful_acks`   | counter     | Number of received successful message acknowledgements         |                                 |
| `core_counter_received_failed_acks`       | counter     | Number of received failed message acknowledgements             |                                 |
| `core_counter_sent_acks`                  | counter     | Number of sent message acknowledgements                        |                                 |
| `core_counter_winning_tickets`            | counter     | Number of winning tickets                                      |                                 |
| `core_counter_losing_tickets`             | counter     | Number of losing tickets                                       |                                 |
| `core_counter_forwarded_messages`         | counter     | Number of forwarded messages                                   |                                 |
| `core_counter_received_messages`          | counter     | Number of received messages                                    |                                 |
| `core_counter_created_tickets`            | counter     | Number of created tickets                                      |                                 |
| `core_counter_packets`                    | counter     | Number of created packets                                      |                                 |
| `core_counter_nr_rejected_conns`          | counter     | Number of rejected connections due to NR                       |                                 |
| `core_gauge_network_health`               | gauge       | Connectivity health indicator                                  | 0 = UNKNOWN, 4 = GREEN          |
| `core_histogram_heartbeat_time_seconds`   | histogram   | Measures total time it takes to probe other nodes (in seconds) | unit: seconds                   |
| `core_counter_heartbeat_successful_pings` | counter     | Total number of successful pings                               |                                 |
| `core_counter_heartbeat_failed_pings`     | counter     | Total number of failed pings                                   |                                 |
| `core_gauge_num_high_quality_peers`       | gauge       | Number of hiqh quality peers                                   | quality > 0.5                   |
| `core_gauge_num_low_quality_peers`        | gauge       | Number of low quality peers                                    | quality <= 0.5                  |
| `core_gauge_num_peers`                    | gauge       | Number of all peers                                            |                                 |
| `core_histogram_ping_time_seconds`        | histogram   | Measures total time it takes to ping a single node (seconds)   | unit: seconds                   |
| `core_mgauge_channel_balances`            | multi_gauge | Balances on channels with counterparties                       | labels: counterparty, direction |

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

### utils

| Name                                                    | Type    | Description                                        | Note |
| ------------------------------------------------------- | ------- | -------------------------------------------------- | ---- |
| `utils_counter_suppressed_unhandled_promise_rejections` | counter | Counter of suppressed unhandled promise rejections |      |
