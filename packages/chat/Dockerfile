
# -- BASE STAGE --------------------------------

FROM node:12.9.1-buster AS base
WORKDIR /src

# use npm 1.19.2
ENV YARN_VERSION 1.19.2
RUN npm policies set-version $YARN_VERSION

COPY package*.json ./
COPY npm.lock ./

RUN npm install --build-from-source 

# -- CHECK STAGE --------------------------------

FROM base AS check

ARG CI
ENV CI $CI

COPY . .
RUN npm test

# -- BUILD STAGE --------------------------------

FROM base AS build

COPY . .
RUN npm build
RUN npm prune --production --no-audit
RUN npm cache clean

# -- RUNTIME STAGE --------------------------------

FROM node:12.9.1-buster AS runtime

ENV NODE_ENV 'production'
WORKDIR /app

COPY --from=build /src/package.json /app/package.json
COPY --from=build /src/node_modules /app/node_modules
COPY --from=build /src/lib /app/lib

EXPOSE 9091
EXPOSE 9092
EXPOSE 9093
EXPOSE 9094
EXPOSE 9095

VOLUME ["/app/db"]

ENTRYPOINT ["node", "./lib/index.js"]
