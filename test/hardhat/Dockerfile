FROM node:16-buster-slim@sha256:ddb4d0ea63591c5a4ef6a9778e3913c3bfcc70328240bb4f97d31d4843587f9b AS base
RUN yarn hardhat node --config packages/ethereum/hardhat.config.ts > "/tmp/$DATAFILE-rpc.txt" 2>&1 &
