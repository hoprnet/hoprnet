---
name: New Chain Epic Template
about: Things to do when switching to a new chain.
title: ''
labels: 'epic'
assignees: ''
---

<!--- Please DO NOT remove the automatically added 'new issue' label -->
<!--- Provide a general summary of the issue in the Title above -->

<!--
  Provide a clear and concise description of what this epic achieves.
-->
### Description

Tasks required to be completed for moving to a new chain.

<!--
  Provide a list of issues, it's okay if the issues are not yet turned into github issues but they are just text.
-->
### Relevant issues

- [ ] Ensure that the `gasPrice` set in hopr is high enough to resist the volatility of `gasPrice` of the chain.
- [ ] Update `CONFIRMATIONS` in [constants](../../packages/core-ethereum/src/constants.ts) using the chain's uncle block rate.
- [ ] Update chain native token symbol.
- [ ] Update chain HOPR token symbol.
- [ ] Ensure E2E tests pass (step_time in [integration-test.sh](https://github.com/hoprnet/hoprnet/blob/3b25d9bc2c55f87cf23a0aa84620933eda6c2e39/test/integration-test.sh#L52) and [full_interconnected_cluster.sh](https://github.com/hoprnet/hoprnet/blob/3b25d9bc2c55f87cf23a0aa84620933eda6c2e39/scripts/topologies/full_interconnected_cluster.sh#L59) is affected by changing `CONFIRMATIONS`).

<!--
  How can a team member know this epic was completed.
-->
### Definition of DONE

All tasks are completed and tested under a [HOPR team testing](../../.processes/release.md#testing-phases) session.