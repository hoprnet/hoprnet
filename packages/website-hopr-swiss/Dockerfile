### Install stage
FROM node:12.9.1-alpine AS base

ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION

# Install app dependencies
COPY . .
RUN yarn install --frozen-lockfile

### Build stage

FROM base as build

COPY . .
RUN yarn build
RUN npm prune --production --no-audit
RUN yarn cache clean

### Run stage

FROM nginx:1.17.8-alpine
COPY --from=build /build /usr/share/nginx/html
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx/nginx.conf /etc/nginx/conf.d
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"] 