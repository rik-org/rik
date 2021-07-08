# This Dockerfile aims to run the project on a distribution which is not
# supported by Runc, Skopeo and Umoci like macOS.
# This is only for development purposes.
FROM alpine:latest AS build
RUN apk add --no-cache runc skopeo umoci rust cargo protoc

WORKDIR /build

COPY ./src ./src
COPY ./Cargo.* ./

RUN cargo build

FROM alpine:latest
COPY --from=build /build/target/debug/riklet .
ENTRYPOINT ["riklet", "--master-ip", "172.20.0.2:4995"]