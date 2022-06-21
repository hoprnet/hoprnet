ARG HOPRD_VERSION=${HOPRD_VERSION}
ARG HARDHAT_VERSION=${HARDHAT_VERSION}

FROM gcr.io/hoprassociation/hoprd:${HOPRD_VERSION} as hoprd

FROM gcr.io/hoprassociation/hopr-hardhat:${HARDHAT_VERSION} as runtime

RUN apk add --no-cache supervisor

WORKDIR /app

COPY --from=hoprd /app .

COPY supervisord.conf /etc/supervisord.conf

ENTRYPOINT ["supervisord", "--nodaemon", "-c", "/etc/supervisord.conf"]
