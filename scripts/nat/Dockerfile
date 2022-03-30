FROM docker:20.10.14-alpine3.15@sha256:7dad83861f0b28bd6a0b281dc5f72144927b9f8173e388e461c8feba6be20bec

# This build arg is mandatory
ARG HOPRD_RELEASE
RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}
RUN apk add bash
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh"]
