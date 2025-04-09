FROM node:23.11-slim AS node_base
WORKDIR /app

FROM node_base AS builder
RUN npm install -g typescript shx

COPY package.json yarn.lock tsconfig.json ./
RUN --mount=type=cache,target=/usr/local/share/.cache/yarn yarn install --ignore-scripts --omit-dev

COPY . ./
RUN --mount=type=cache,target=/usr/local/share/.cache/yarn tsc && shx chmod +x dist/*.js

FROM node_base
COPY package.json yarn.lock ./
COPY --from=builder /app/dist ./dist
ENV NODE_ENV=production
RUN --mount=type=cache,target=/usr/local/share/.cache/yarn yarn install --ignore-scripts --omit-dev
ENTRYPOINT ["node", "/app/dist/index.js"]