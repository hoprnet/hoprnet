# -- BASE STAGE --------------------------------

FROM node:12.9.1-buster AS base
WORKDIR /src

ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION

COPY package*.json ./
COPY yarn.lock ./

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    ca-certificates \
    fuse \
    gcc \
    cmake \
    wget

RUN yarn install --build-from-source --frozen-lockfile

# -- CHECK STAGE --------------------------------

# FROM base AS check

# ARG CI
# ENV CI $CI

# COPY . .
# RUN yarn test

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

COPY --from=build /src/node_modules /app/node_modules
COPY --from=build /src/package.json /app/package.json
COPY --from=build /src/tsconfig.json /app/tsconfig.json
COPY --from=build /src/dist /app/dist
COPY --from=build /src/envoy/envoy.yaml.tmpl /app/envoy/envoy.yaml.tmpl
COPY --from=build /src/envoy/envoy.sh /app/envoy/envoy.sh
COPY --from=build /src/process.yaml /app/process.yaml

EXPOSE 9091
EXPOSE 50051
EXPOSE 8080
EXPOSE 8081

VOLUME ["/app/db"]

CMD ["pm2-runtime", "process.yaml"]