ARG HOPRD_IMAGE=${HOPRD_IMAGE}
ARG HARDHAT_IMAGE=${HARDHAT_IMAGE}

# simply import hoprd image to copy in files later
FROM ${HOPRD_IMAGE} as hoprd

FROM ${HARDHAT_IMAGE} as runtime

LABEL description="Docker image running a HOPR-enabled hardhat network with 5 hoprd nodes using it and being fully interconnected."

# install tools required within our scripts
RUN apk add --no-cache bash lsof curl jq

WORKDIR /app

COPY --from=hoprd /app .

# copy deployment files to make them available to hoprd as well
RUN mkdir /hardhat/deployments/hardhat-localhost \
 && ln -s /hardhat/deployments/hardhat-localhost ./node_modules/@hoprnet/hopr-ethereum/deployments/

# hardhat RPC port
EXPOSE 8545
# hoprd Rest API ports
EXPOSE 13301
EXPOSE 13302
EXPOSE 13303
EXPOSE 13304
EXPOSE 13305
# hoprd p2p ports
EXPOSE 18081
EXPOSE 18082
EXPOSE 18083
EXPOSE 18084
EXPOSE 18085
# hoprd Admin UI ports
EXPOSE 19091
EXPOSE 19092
EXPOSE 19093
EXPOSE 19094
EXPOSE 19095
# hoprd healthcheck ports
EXPOSE 19501
EXPOSE 19502
EXPOSE 19503
EXPOSE 19504
EXPOSE 19505

# set default token, but keep it configurable for users
ARG HOPRD_API_TOKEN
ENV HOPRD_API_TOKEN=${HOPRD_API_TOKEN:-%th1s-IS-a-S3CR3T-ap1-PUSHING-b1ts-TO-you%}

ENTRYPOINT /hardhat/scripts/setup-local-cluster.sh -p -t "${HOPRD_API_TOKEN}" -i /hardhat/scripts/topologies/full_interconnected_cluster.sh --hoprd-command "npx hoprd" --hardhat-basedir "/hardhat" -l "0.0.0.0"
