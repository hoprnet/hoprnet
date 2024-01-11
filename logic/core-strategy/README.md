# core-strategy

This crate contains all the Strategies for HOPRd.

- passive strategy
- promiscuous strategy
- auto funding strategy
- auto redeeming strategy
- aggregating strategy
- multi strategy

HOPRd can be configured to use any of the above strategies. See the Configuration section below.

## Passive Strategy

This strategy does nothing.

## Promiscuous Strategy

This strategy opens or closes automatically channels based the following rules:

- if node quality is below or equal to a threshold `network_quality_threshold` and we have a channel opened to it, the strategy will close it
  - if node quality is above `network_quality_threshold` and no channel is opened yet, it will try to open channel to it (with initial stake `new_channel_stake`).
    However, the channel is opened only if the following is both true:
  - the total node balance does not drop below `minimum_node_balance`
  - the number of channels opened by this strategy does not exceed `max_channels`

Also, the candidates for opening (quality > `network_quality_threshold`), are sorted by best quality first.
So that means if some nodes cannot have channel opened to them, because we hit `minimum_node_balance` or `max_channels`,
the better quality ones were taking precedence.

The sorting algorithm is intentionally unstable, so that the nodes which have the same quality get random order.
The constant `k` can be also set to a value > 1, which will make the strategy to open more channels for smaller networks,
but it would keep the same asymptotic properties.
Per default `k` = 1.

The strategy starts acting only after at least `min_network_size_samples` network size samples were gathered, which means
it does not start opening/closing channels earlier than `min_network_size_samples` number of minutes after the node has started.

### Default parameters

- `network_quality_threshold`: 0.5
- `new_channel_stake`: 10 HOPR
- `minimum_node_balance`: 10 HOPR
- `min_network_size_samples`: 10
- `max_channels:` None (defaults to square root of network size)
- `enforce_max_channels`: true (when set, indicates whether the `max_channels` limit is enforced)

## Auto Funding Strategy

This strategy listens for channel state change events to check whether a channel has dropped below `min_stake_threshold` HOPR.
If this happens, the strategy issues a **fund channel** transaction to re-stake the channel with `funding_amount` HOPR.

### Default parameters

- `min_stake_threshold`: 1 HOPR
- `funding_amount`: 10 HOPR

## Auto Redeeming Strategy

Strategy which listens for newly added acknowledged tickets and automatically issues a redeem transaction on that ticket.
It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far less on-chain transactions being issued).

### Default parameters

- `redeem_only_aggregated`: false

## Aggregating Strategy

This strategy automates ticket aggregation on different channel/ticket events.
Note that the aggregating strategy can be combined with the Auto Redeem Strategy above.

Ticket aggregation is an interactive process and requires cooperation of the ticket issuer, the aggregation
will fail if the aggregation takes more than `aggregation_timeout` (in seconds). This does not affect runtime of the
strategy, since the ticket aggregation and awaiting it is performed on a separate thread.

This strategy listens for two distinct channel events and triggers the interactive aggregation based on different criteria:

### 1) New winning acknowledged ticket event

This strategy listens to newly added acknowledged winning tickets and once the amount of tickets in a certain channel reaches
an `aggregation_threshold`, the strategy will initiate ticket aggregation in that channel.
The strategy can independently also check if the unrealized balance (current balance _minus_ total unredeemed unaggregated tickets value) in a certain channel
has not gone over `unrelalized_balance_ratio` percent of the current balance in that channel. If that happens, the strategy will also initiate
ticket aggregation.

### 2) Channel transition from `Open` to `PendingToClose` event

If the `aggregate_on_channel_close` flag is set, the aggregation will be triggered once a channel transitions from `Open` to `PendingToClose` state.
This behavior does not have any additional criteria, unlike in the previous event.

The aggregation on channel closure slightly differs in what happens on failure behaviour.
If the aggregation on channel closure fails, the unaggregated tickets in that channel are automatically send for redeeming.
When this strategy is triggered from the

### Default parameters

- `aggregation_threshold`: 100 tickets
- `unrealized_balance_ratio`: 0.9 (= 90%)
- `aggregation_timeout`: 60 seconds
- `aggregate_on_channel_close`: true

## Multi Strategy

This strategy can stack multiple above strategies (called sub-strategies in this context) into one.
Once a strategy event is triggered, it is executed sequentially on the sub-strategies one by one.
The strategy can be configured to not call the next sub-strategy event if the sub-strategy currently being executed failed,
which is done by setting the `on_fail_continue` flag.

Hence, the sub-strategy chain then can behave as a logical AND (`on_fail_continue` = `false`) execution chain
or logical OR (`on_fail_continue` = `true`) execution chain.

A Multi Strategy can also contain another Multi Strategy as a sub-strategy if `allow_recursive` flag is set.
However, this recursion is always allowed up to 2 levels only.
Along with the `on_fail_continue` value, the recursive feature allows constructing more complex logical strategy chains.

The MultiStrategy can also observe channels being `PendingToClose` and running out of closure grace period,
and if this happens, it will issue automatically the final close transaction, which transitions the state to `Closed`.
This can be controlled by the `finalize_channel_closure` parameter.

### Default parameters

- `on_fail_continue`: true
- `allow_recursive`: true
- `finalize_channel_closure`: false
- `strategies`: none (the Multi strategy behaves as Passive strategy per default)

## Configuring strategies in HOPRd

There are two ways of configuring strategies in HOPRd: via CLI and via a YAML config file.

The configuration through CLI allows only fairly primitive single-strategy setting, through the `defaultStrategy`
parameter. It can be set to any of the above strategies, however the strategy parameters are not further
configurable via the CLI and will always have their default values.
In addition, if `disableTicketAutoRedeem` CLI argument is `false`, the default Auto Redeem strategy is added to the
strategy configured via the `defaultStrategy` argument (they execute together as Multi strategy).

For more complex strategy configurations, the YAML configuration method is recommended via the `strategy` YAML section.
In this case, the top-most strategy is always assumed to be Multi strategy:

```yaml
strategy:
  on_fail_continue: true
  allow_recursive: true
  strategies:
    - !Promiscuous
      max_channels: 50
      new_channel_stake: 20
    - !AutoFunding
      funding_amount: 20
    - !Aggregating:
      aggregation_threshold: 1000
```
