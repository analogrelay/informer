FROM golang:1.15 AS build

WORKDIR /app

COPY go.mod go.sum /app/
RUN go mod download

COPY . /app/
RUN CGO_ENABLED=0 GOOS=linux script/build

FROM alpine:3.13

RUN apk --no-cache add ca-certificates

WORKDIR /app
COPY --from=build /app/bin/ /app/bin/

CMD [ "/app/bin/informer" ]
