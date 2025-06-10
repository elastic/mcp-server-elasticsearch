FROM docker.elastic.co/wolfi/chainguard-base:latest@sha256:3d19648819612728a676ab4061edfb3283bd7117a22c6c4479ee1c1d51831832

RUN apk --no-cache add nodejs npm

WORKDIR /app
COPY . ./
RUN npm install && npm run build

ENTRYPOINT ["npm", "start"]
