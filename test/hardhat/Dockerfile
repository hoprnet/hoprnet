FROM node:14.15-buster@sha256:74a322375cbf1bfca73e3d41f39d4a04e4869c7defdf83f9663166989d1b553c AS base
RUN yarn hardhat node --config packages/ethereum/hardhat.config.ts > "/tmp/$DATAFILE-rpc.txt" 2>&1 &
