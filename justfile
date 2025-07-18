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

package packager arch:
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail
    RELEASE_VERSION=$(./scripts/get-current-version.sh)
    case "{{arch}}" in
        x86_64-linux)
            ARCHITECTURE="amd64"
            ;;
        aarch64-linux)
            ARCHITECTURE="arm64"
            ;;
        *)
            echo "Unsupported architecture: {{arch}}"
            exit 1
            ;;
    esac
    export RELEASE_VERSION ARCHITECTURE
    envsubst < ./deploy/nfpm/nfpm.yaml > ./deploy/nfpm/nfpm.generated.yaml
    ./scripts/generate-changelog.sh "${RELEASE_VERSION}" "{{packager}}" true > ./deploy/nfpm/changelog
    help2man --name="HOPR node executable" --no-info --output ./deploy/nfpm/hoprd.1 ./dist/bin/hoprd
    gzip -9n ./deploy/nfpm/hoprd.1
    help2man --name="HOPR CLI helper tool" --no-info --output ./deploy/nfpm/hopli.1 ./dist/bin/hopli
    gzip -9n ./deploy/nfpm/hopli.1
    [[ "{{packager}}" == "deb" ]] && cat ./deploy/nfpm/changelog | gzip -9 > ./deploy/nfpm/changelog.gz
    [[ "{{packager}}" == "deb" ]] && sed -i.backup '/^license:.*/d' deploy/nfpm/nfpm.generated.yaml && rm deploy/nfpm/nfpm.generated.yaml.backup
    mkdir -p dist/packages
    nfpm package --config deploy/nfpm/nfpm.generated.yaml --packager "{{packager}}" --target "dist/packages/hoprd-{{arch}}.{{packager}}"

test-package packager arch:
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail
    if [ "{{packager}}" == "archlinux" ] && [ "{{arch}}" == "aarch64-linux" ]; then
        echo "Skipping test for archlinux aarch64-linux as it is not supported yet in GCP images."
        exit 0
    fi
    trap 'deploy/nfpm/test-package-tool.sh delete {{packager}} {{arch}} 2>&1 | tee deploy/nfpm/test-package-{{packager}}-{{arch}}.log' EXIT
    deploy/nfpm/test-package-tool.sh create {{packager}} {{arch}} 2>&1 | tee deploy/nfpm/test-package-{{packager}}-{{arch}}.log
    deploy/nfpm/test-package-tool.sh copy {{packager}} {{arch}} 2>&1 | tee -a deploy/nfpm/test-package-{{packager}}-{{arch}}.log
    deploy/nfpm/test-package-tool.sh install {{packager}} {{arch}} 2>&1 | tee -a deploy/nfpm/test-package-{{packager}}-{{arch}}.log

# list all available docker image targets which can be built
list-docker-images:
    nix flake show --json | jq '.packages | to_entries | .[0].value | to_entries[] | select(.key | endswith("docker")) | .key'
