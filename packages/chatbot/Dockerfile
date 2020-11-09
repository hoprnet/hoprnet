# -- BASE STAGE --------------------------------
FROM node:12.9.1-buster AS base
WORKDIR /src

# use yarn 1.19.2
ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    ca-certificates \
    fuse \
    gcc \
    cmake \
    wget

COPY package*.json ./

# -- INSTALL STAGE ------------------------------
RUN yarn install --build-from-source

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

ENTRYPOINT ["node", "./dist/index.js"]
