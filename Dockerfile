FROM node:23.11-slim AS builder
WORKDIR /app
RUN npm install -g typescript shx

COPY . ./
RUN --mount=type=cache,target=/root/.npm npm install

ENTRYPOINT ["node", "/app/dist/index.js"]