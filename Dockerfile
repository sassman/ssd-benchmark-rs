FROM rust:alpine3.21 as builder

WORKDIR /app

ADD Cargo.toml Cargo.lock src /app/
RUN apk update && \
    apk add musl-dev jq && \
    cargo build --release --message-format=json > build.json
# copy binary to a new location
RUN jq -r '.executable | select(. != null)' build.json | xargs -I '{}' cp '{}' /app

# final image
FROM alpine:latest

LABEL name="ssd-benchmark"
LABEL org.opencontainers.image.source="https://github.com/sassman/ssd-benchmark-rs"
LABEL repository="https://github.com/sassman/ssd-benchmark-rs"
LABEL homepage="https://github.com/sassman/ssd-benchmark-rs"
LABEL maintainer="Sven Kanoldt <sven@d34dl0ck.me>"

COPY --from=builder /app/ssd-benchmark \
    /usr/local/bin/
CMD /usr/local/bin/ssd-benchmark
