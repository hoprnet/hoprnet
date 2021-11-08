On every tick of cover-traffic strategy, a few metrics are saved in the persisted state (`packages/cover-traffic-daemon/ct.json`).

Other metrics can be deduced from logs. Note that logs start with `hopr:cover-traffic `

1. At each tick, a log lists the current balance of the CT node, channels to open and to close.
   ```sh
   strategy tick: ${Date.now()} balance:${balance.toString()} open:${toOpen
           .map((p) => p[0].toPeerId().toB58String())
           .join(',')} close: ${toClose.map((p) => p.toPeerId().toB58String()).join(',')}`.replace('\n', ', ')
   ```
2. Status of all the CT channels registered in the persisted state:
   ```sh
   'channels',
   ctChannels.map((c) => `${c.destination.toB58String()} - ${c.latestQualityOf}, ${c.openFrom}`).join('; ')
   ```
3. Reasons to close a channel:
   - the destination node has a network quality lower than the `CT_NETWORK_QUALITY_THRESHOLD` threshold (default, `0.15`).
     ```sh
     closing channel ${c.destination.toB58String()} with quality < ${CT_NETWORK_QUALITY_THRESHOLD}
     ```
   - it does not have enough stake.
     ```sh
     closing channel with balance too low ${c.destination.toB58String()}
     ```
   - its failed traffic rate reaches the `MESSAGE_FAIL_THRESHOLD` threshold (default, `10`).
     ```sh
     closing channel with too many message fails: ${c.destination.toB58String()}
     ```
   - it stalls at `WAIT_FOR_COMMITMENT` state for too long.
     ```sh
     channel is stalled in WAITING_FOR_COMMITMENT, closing openChannel.destination.toB58String()
     ```
4. Opening channel
   ```sh
   opening ${c.toB58String()}
   ```
5. Sending traffic:
   - Failure in sending traffic:
     ```sh
     aborting send messages - less channels in network than hops required
     ```
     more specifically CT channels that fails to send traffic:
     ```sh
     failed to send to ${openChannel.destination.toB58String()} fails: ${this.data.messageFails  (openChannel.destination)}
     ```
   - Success in sending traffic:
     ```sh
     message send phase complete
     ```
6. As CT node is also a HOPR node, it can relay packets and accumulate tickets. However, it does not react upon a winning ticket and simply logs
   ```sh
   cover traffic ignores winning ticket.
   ```
