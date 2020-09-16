FROM node:alpine

ARG HOPR_ENVIRONMENT

WORKDIR /usr/app
COPY package.json .
RUN yarn install
COPY . .

VOLUME /appdata
RUN NEXT_PUBLIC_HOPR_ENVIRONMENT=$HOPR_ENVIRONMENT yarn run build

EXPOSE 80
EXPOSE 3000
ENV NODE_ENV production
CMD ["yarn", "start"]
