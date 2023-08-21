ARG HOPRD_IMAGE=${HOPRD_IMAGE:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:latest}
ARG HOPLI_IMAGE=${HOPLI_IMAGE:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hopli:latest}
ARG ANVIL_IMAGE=${ANVIL_IMAGE:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hopr-anvil:latest}

# import hopli image to copy in files later
FROM ${HOPLI_IMAGE} as hopli

# import hoprd image to copy in files later
FROM ${HOPRD_IMAGE} as hoprd

# the anvil image is our base runtime image since it includes most relevant
# tools already
FROM ${ANVIL_IMAGE} as runtime

LABEL description="Docker image running a HOPR-enabled anvil chain with 5 hoprd nodes using it and being fully interconnected."

# install tools required within our scripts
RUN apt-get update \
 && apt-get install -y \
      bash curl jq lsof \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

# Get hopli binary
COPY --from=hopli /bin/hopli /bin/

WORKDIR /app/hoprnet

COPY --from=hoprd /app/hoprnet/ ./

COPY packages/ethereum/contracts packages/ethereum/contracts
COPY scripts/ scripts
COPY Makefile .

# anvil RPC port
EXPOSE 8545
# hoprd Rest API ports
EXPOSE 13301
EXPOSE 13302
EXPOSE 13303
EXPOSE 13304
EXPOSE 13305
# hoprd p2p ports
EXPOSE 19091
EXPOSE 19092
EXPOSE 19093
EXPOSE 19094
EXPOSE 19095
# hoprd healthcheck ports
EXPOSE 18081
EXPOSE 18082
EXPOSE 18083
EXPOSE 18084
EXPOSE 18085

# set default token, but keep it configurable for users
ARG HOPRD_API_TOKEN
ENV HOPRD_API_TOKEN=${HOPRD_API_TOKEN:-%th1s-IS-a-S3CR3T-ap1-PUSHING-b1ts-TO-you%}
ENV NODE_OPTIONS="--experimental-wasm-modules"
ENTRYPOINT /app/hoprnet/scripts/setup-local-cluster.sh -p -t "${HOPRD_API_TOKEN}" -i /app/hoprnet/scripts/topologies/full_interconnected_cluster.sh --hoprd-command "node packages/hoprd/lib/main.cjs" -l "0.0.0.0"
