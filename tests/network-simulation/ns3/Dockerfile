FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV NS3_VERSION=3.43

RUN apt update \
    && apt install git bzip2 g++ python3 cmake make tar wget libc6-dev sqlite3 libsqlite3-dev -y \
    && rm -rf /var/lib/apt/lists/*

RUN cd /usr/local && \
    wget https://www.nsnam.org/releases/ns-allinone-${NS3_VERSION}.tar.bz2 && \
    tar xjf ns-allinone-${NS3_VERSION}.tar.bz2

# Build ns-3
WORKDIR /usr/local/ns-allinone-${NS3_VERSION}/ns-${NS3_VERSION}

RUN ./ns3 configure --enable-tests \
    && ./ns3 build

# Test installation is successful
RUN ./ns3 run test.py
