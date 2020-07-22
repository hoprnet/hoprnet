
# -- BASE STAGE --------------------------------

FROM node:12.9.1-alpine AS base
WORKDIR /src

# use yarn 1.19.2
ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION

COPY package*.json ./
COPY yarn.lock ./

# Install OS libs necessary by some packages during `npm i` (e.g.: node-canvas)
RUN apk add --update \
&& apk add --no-cache alsa-lib ffmpeg opus pixman cairo pango giflib ca-certificates \
&& apk add --no-cache --virtual .build-deps git curl build-base jpeg-dev pixman-dev \
cairo-dev pango-dev pangomm-dev gcompat libjpeg-turbo-dev giflib-dev freetype-dev python g++ make \
\
&& yarn install --build-from-source --frozen-lockfile \
\
&& apk del .build-deps

RUN apk add libc6-compat
RUN ln -s /lib/libc.musl-x86_64.so.1 /lib/ld-linux-x86-64.so.2

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
FROM node:12.9.1-alpine AS runtime

ENV NODE_ENV 'production'
RUN apk --no-cache add curl
WORKDIR /app

COPY --from=build /src/node_modules /app/node_modules
COPY --from=build /src/package.json /app/package.json
COPY --from=build /src/tsconfig.json /app/tsconfig.json
COPY --from=build /src/dist /app/dist

EXPOSE 9091
EXPOSE 50051

VOLUME ["/app/db"]

# Needed by HOPR
RUN apk add libc6-compat
RUN ln -s /lib/libc.musl-x86_64.so.1 /lib/ld-linux-x86-64.so.2

ENTRYPOINT ["node", "./dist/main.js"]