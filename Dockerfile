#
# Run HOPRd and and Envoy proxy within a single container.
#
# NB. This is not a long term solution, so you probably don't want to rely on 
# this, our ultimate goal is a single HOPRd process, but this is a quick solution
# to expedite our HOPR node work.
#

# -- BASE STAGE --------------------------------

FROM node:12.9.1-buster AS base
WORKDIR /hoprd

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    ca-certificates \
    fuse \
    gcc \
    cmake \
    wget

ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION
COPY package*.json ./
COPY yarn.lock ./
RUN yarn install --build-from-source --frozen-lockfile

# -- BUILD STAGE --------------------------------

FROM base as build
COPY . .
RUN yarn build
RUN npm prune --production --no-audit
RUN yarn cache clean

# -- RUNTIME STAGE ------------------------------\

FROM node:12.9.1-buster AS runtime

# install envoy
RUN apt-get update && \
    apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg-agent \
    software-properties-common \
    gettext

RUN curl -sL 'https://getenvoy.io/gpg' | apt-key add -

RUN apt-key fingerprint 6FF974DB

RUN add-apt-repository \
  "deb [arch=amd64] https://dl.bintray.com/tetrate/getenvoy-deb $(lsb_release -cs) stable"

RUN apt-get update && apt-get install -y getenvoy-envoy

# install yarn
RUN yarn global add pm2

ENV NODE_ENV 'production'
WORKDIR /app


# Server
COPY --from=build /hoprd/lib/server.js /app/server.js

# Envoy
COPY --from=build /hoprd/envoy/envoy.yaml.tmpl /app/envoy/envoy.yaml.tmpl
COPY --from=build /hoprd/envoy/docker-entrypoint.sh /app/envoy/envoy.sh

# PM2
COPY --from=build /hoprd/process.yaml /app/process.yaml


RUN rm -rf db # If we somehow added a database entry, kill it or we share a peer id
#VOLUME ["/app/db"]


EXPOSE 9091
EXPOSE 50051
EXPOSE 8080
EXPOSE 8081
EXPOSE 3000
CMD ["pm2-runtime", "process.yaml"]

