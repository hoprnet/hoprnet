This Pull requests contains all the required changes needed before releasing a new version.

- [ ] Check that there is an entry in `packages/hoprd/releases.json` for the new release name.
   - If the release will run in its own network ($RELEASENAME == $NETWORK) then a new entry in `packages/core/protocol-config.json` should be created for the network.
   - If the release will run in a multinetwork network like `dufour` then update the file `packages/core/protocol-config.json` for the `dufour` entry to accept the new `version_range` of the new release.
- [ ] Change all occurences of the last release name to the new release name within documentation files and Docker files. Don't touch the `protocol-config.json` and `releases.json` files in this step.
- [ ] Check the [milestone](https://github.com/hoprnet/hoprnet/milestones) contents and that all items are closed. And there is no missing issue or PR to be included.
- [ ] If the release will run in a new network then, check that the entry `networks` in `contracts-addresses.json`, contains its own network

   ```
    "new_network": {
      "boost_contract_address": "",
      "channels_contract_address": "",
      "environment_type": "production",
      "indexer_start_block_number": 0,
      "network_registry_contract_address": "",
      "network_registry_proxy_contract_address": "",
      "stake_contract_address": "",
      "stake_season": 7,
      "token_contract_address": "",
      "xhopr_contract_address": ""
    }
   ```

   NOTE: Don't include the deployment of HoprChannels, because this will be re-deployed anyway by the CD system.
   Changes should be committed locally.

- [ ] Assess that all CI checks have passed successfully and wait for the docker images builds to finish.
- [ ] Get the approval from at least 2 members
