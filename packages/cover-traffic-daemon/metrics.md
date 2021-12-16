On every tick of cover-traffic strategy, a few metrics are saved in the persisted state (`packages/cover-traffic-daemon/ct.json`).

Other metrics can be deduced from logs. Note that logs start with `hopr:cover-traffic `

1. At each tick, a log lists the current balance of the CT node, channels to open and to close.
   ```
   strategy tick: ${Date.now()} balance:${balance.toString()} open:${toOpen
           .map((p) => p[0].toPeerId().toB58String())
           .join(',')} close: ${toClose.map((p) => p.toPeerId().toB58String()).join(',')}`.replace('\n', ', ')
   ```
   E.g.
   ```
   hopr:cover-traffic strategy tick: 1635409563298 balance:99999999999999999000 open:16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1 close: 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7
   ```
2. Status of all the CT channels registered in the persisted state:
   ```
   channels ${ctChannels.map((c) => `${c.destination.toB58String()} - ${c.latestQualityOf}, ${c.openFrom}`).join('; ')}
   ```
   E.g.
   ```
   hopr:cover-traffic channels 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1 - 1, 1635409664132; 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7 - 1, 1635409664133
   ```
3. Reasons to close a channel:
   - the destination node has a network quality lower than the `CT_NETWORK_QUALITY_THRESHOLD` threshold (default, `0.15`).
     ```js
     closing channel ${c.destination.toB58String()} with quality < ${CT_NETWORK_QUALITY_THRESHOLD}
     ```
     E.g.
     ```
     closing channel 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7 with quality < 0.15
     ```
   - it does not have enough stake.
     ```js
     closing channel with balance too low ${c.destination.toB58String()}
     ```
     E.g.
     ```
     hopr:cover-traffic closing channel with balance too low 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7
     ```
   - its failed traffic rate reaches the `MESSAGE_FAIL_THRESHOLD` threshold (default, `10`).
     ```js
     closing channel with too many message fails: ${c.destination.toB58String()}
     ```
     E.g.
     ```
     hopr:cover-traffic closing channel with too many message fails: 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7
     ```
   - it stalls at `WAIT_FOR_COMMITMENT` state for too long.
     ```js
     channel is stalled in WAITING_FOR_COMMITMENT, closing ${openChannel.destination.toB58String()}
     ```
     E.g.
     ```
     hopr:cover-traffic channel is stalled in WAITING_FOR_COMMITMENT, closing 16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7
     ```
   - for other (unknown) errors:
     ```js
     Unknown error in sending traffic. Channel is ${channel.status}; openChannel is ${JSON.stringify(openChannel)}
     ```
     E.g.
     ```
     hopr:cover-traffic Unknown error in sending traffic. Channel is PENDING_TO_CLOSE; openChannel is {"destination":"16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7","lastestQualityOf":0.5,"openFrom":1639400526749}
     ```
4. Opening channel
   ```js
   opening ${c.toB58String()}
   ```
   E.g.
   ```
   hopr:cover-traffic opening 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1
   ```
5. Sending traffic:
   - Failure in sending traffic:
     ```sh
     aborting send messages - less channels in network than hops required
     ```
     more specifically CT channels that fails to send traffic:
     ```typescript
     failed to send to ${openChannel.destination.toB58String()} fails: ${this.data.messageFails  (openChannel.destination)}
     ```
     ```
     hopr:cover-traffic failed to send to 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1 fails: 3
     ```
   - Success in sending traffic:
     ```
     message send phase complete
     ```
6. As CT node is also a HOPR node, it can relay packets and accumulate tickets. However, it does not react upon a winning ticket and simply logs
   ```
   cover traffic ignores winning ticket.
   ```
