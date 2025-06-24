# generate smart contract bindings
generate-bindings:
    cd ethereum/contracts; \
    forge bind --offline --bindings-path ./../bindings/src/codegen \
      --module --alloy --overwrite \
      --force --skip-cargo-toml \
      --select '^(HoprAnnouncements|HoprAnnouncementsEvents|HoprCapabilityPermissions|HoprChannels|HoprChannelsEvents|HoprCrypto|HoprDummyProxyForNetworkRegistry|HoprBoost|HoprToken|HoprLedger|HoprLedgerevents|HoprMultisig|HoprNetworkRegistry|HoprNetworkRegistryEvents|HoprNodeManagementModule|HoprNodeSafeRegistry|HoprNodeSafeRegistryEvents|HoprNodeStakeFactory|HoprNodeStakeFactoryEvents|HoprSafeProxyForNetworkRegistry|HoprStakingProxyForNetworkRegistry|HoprTicketPriceOracle|HoprTicketPriceOracleEvents|HoprWinningProbabilityOracle|HoprWinningProbabilityOracleEvents)$'

# smart contract tests
# we must run the tests in separate groups to avoid IO race conditions
# Remove `--no-match-test` when https://github.com/foundry-rs/foundry/issues/10586 is fixed
smart-contract-test:
    forge test --gas-report --root ./ethereum/contracts --match-path "./test/scripts/DeployAll.t.sol" && \
    forge test --gas-report --root ./ethereum/contracts --match-path "./test/scripts/DeployNodeManagement.t.sol" && \
    forge test --gas-report --root ./ethereum/contracts --no-match-path "./test/scripts/Deploy*.t.sol" --no-match-test "test.*_.*DomainSeparator"


# run all smoke tests
run-smoke-test-all:
    nix develop .#citest -c uv run --frozen -m pytest tests/

# run a single smoke test
run-smoke-test TEST:
    nix develop .#citest -c uv run --frozen -m pytest tests/test_{{TEST}}.py
