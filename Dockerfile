# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0
FROM cgr.dev/chainguard/wolfi-base:latest

RUN apk --no-cache add nodejs npm

WORKDIR /app

# Install dependencies (Docker build cache friendly)
COPY package.json package-lock.json tsconfig.json ./
RUN touch index.ts
RUN npm install

COPY *.ts ./
RUN npm run build

COPY run-docker.sh .

# Future-proof the CLI and require the "stdio" argument
ENV RUNNING_IN_CONTAINER="true"

ENTRYPOINT ["./run-docker.sh"]
