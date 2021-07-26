FROM node:16-buster-slim@sha256:028cd08984d88964f206077d56756ac594043cbd32b029e167ca23f824981df7 AS base
RUN yarn hardhat node --config packages/ethereum/hardhat.config.ts > "/tmp/$DATAFILE-rpc.txt" 2>&1 &
