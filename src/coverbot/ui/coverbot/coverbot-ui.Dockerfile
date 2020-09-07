FROM node:alpine

WORKDIR /usr/app
COPY package.json .
RUN yarn install
COPY . .

VOLUME /appdata
RUN STATS_FILE=/appdata/stats.json yarn run build

EXPOSE 80
EXPOSE 3000
ENV NODE_ENV production
CMD ["yarn", "start"]
