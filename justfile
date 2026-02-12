run-local-cluster:
    #!/usr/bin/env bash
    uv sync --no-dev
    uv run --no-sync -m sdk.python.localcluster --config ./sdk/python/localcluster.params.yml --fully_connected --exposed


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

sign-file source_file:
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail
    echo "Signing file: {{source_file}}"
    basename="$(basename {{source_file}})"
    dirname="$(dirname {{source_file}})"

     # Create isolated GPG keyring
    gnupghome="$(mktemp -d)"
    export GNUPGHOME="$gnupghome"
    echo "$GPG_HOPRNET_PRIVATE_KEY" | gpg --batch --import

     # Generate hash and signature
    cd "$dirname"
    shasum -a 256 "$basename" > "$basename.sha256"
    echo "Hash written to $basename.sha256"
    gpg --armor --output "$basename.asc" --detach-sign "$basename"
    echo "Signature written to $basename.asc"

     # Clean up
    rm -rf "$gnupghome"

# list all available docker image targets which can be built
list-docker-images:
    nix flake show --json | jq '.packages | to_entries | .[0].value | to_entries[] | select(.key | endswith("docker")) | .key'


localcluster clustersize:
    docker rm -f anvil_blokli || true 
    docker run --rm --name anvil_blokli --platform linux/amd64 -p 8080:8080 -d europe-west3-docker.pkg.dev/hoprassociation/docker-images/bloklid-anvil:latest
    cargo run -p hoprd-localcluster -- --chain-url http://localhost:8080  --hoprd-bin ./result/bin/hoprd --size {{clustersize}}
