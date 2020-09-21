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

# -- RUNTIME STAGE ------------------------------

FROM node:12.9.1-buster AS runtime

ENV NODE_ENV 'production'
WORKDIR /app

COPY --from=build /src/node_modules /app/node_modules
COPY --from=build /src/package.json /app/package.json
COPY --from=build /src/tsconfig.json /app/tsconfig.json
COPY --from=build /src/dist /app/dist

EXPOSE 9091
EXPOSE 50051

VOLUME ["/app/db"]

ENTRYPOINT ["node", "./dist/main.js"]