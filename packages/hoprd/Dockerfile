ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hopr-toolchain:latest}

# Creates a Docker container that encapsulates a horpd node

FROM --platform=linux/amd64 ${HOPR_TOOLCHAIN_IMAGE} as build

WORKDIR /app/hoprnet

RUN mv /app/hoprnet-toolchain/.cargo .
COPY .cargo/config.toml .cargo/
COPY vendor vendor/
COPY packages packages/
COPY misc misc/
COPY Makefile Cargo.toml Cargo.lock rust-toolchain.toml ./

RUN make build-solidity-types \
    && cargo build --release \
    && cargo install --root ./artifacts --path packages/hoprd/crates/hoprd-hoprd/

# Debian 11.7
# https://hub.docker.com/layers/library/debian/11.7-slim/images/sha256-b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3?context=explore
FROM --platform=linux/amd64 debian:11.7-slim@sha256:b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3 as runtime

LABEL description="Launches a hoprd node"

WORKDIR /app/hoprnet

RUN apt-get update && \
    apt-get install -y bash curl jq tini ca-certificates tar xz-utils && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false && \
    # create directory which is later used for the database, so that it inherits
    # permissions when mapped to a volume
    mkdir -p hoprd-db

COPY --from=build /app/hoprnet/artifacts/bin/ /usr/local/bin/

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprnet/hoprd-db"]

# set data directory to user-mountable directory
ENV HOPRD_DATA=/app/hoprnet/hoprd-db

WORKDIR /app/hoprnet/packages/hoprd

# making sure some standard environment variables are set for production use
ENV RUST_LOG "debug"
ENV RUST_BACKTRACE 1

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080
# p2p UDP + TCP
EXPOSE 9091/udp
EXPOSE 9091/tcp

ENTRYPOINT ["/usr/bin/tini", "--", "hoprd"]
