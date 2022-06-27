ARG HOPRD_IMAGE=${HOPRD_IMAGE}
ARG HARDHAT_IMAGE=${HARDHAT_IMAGE}
ARG HOPRD_API_TOKEN

FROM ${HOPRD_IMAGE} as hoprd

FROM ${HARDHAT_IMAGE} as runtime

RUN apk add --no-cache bash libc6-compat lsof

# symlink required to get WRTC to work
RUN ln -s /lib/libc.musl-x86_64.so.1 /lib/ld-linux-x86-64.so.2

WORKDIR /app

COPY --from=hoprd /app .

#COPY supervisord.conf /etc/supervisord.conf

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

ENV HOPRD_API_TOKEN=${HOPRD_API_TOKEN:-%th1s-IS-a-S3CR3T-ap1-PUSHING-b1ts-TO-you%}

#
#ENTRYPOINT ["supervisord", "-c", "/etc/supervisord.conf"]
ENTRYPOINT /hardhat/scripts/setup-local-cluster.sh -t "${HOPRD_API_TOKEN}" -i /hardhat/scripts/topologies/full_interconnected_cluster.sh --hardhat-command "yarn run network" --hoprd-command "yarn hoprd" --hardhat-basedir "/hardhat"
