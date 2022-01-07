FROM docker:20.10.12-alpine3.15

ARG HOPR_RELEASE=latest
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh", "$HOPR_RELEASE"]
