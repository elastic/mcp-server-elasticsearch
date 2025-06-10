FROM cgr.dev/chainguard/wolfi-base:latest

RUN apk --no-cache add nodejs npm

WORKDIR /app
COPY . ./
RUN npm install && npm run build

ENTRYPOINT ["npm", "start"]
