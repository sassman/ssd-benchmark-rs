FROM ekidd/rust-musl-builder:latest as builder

ADD --chown=rust:rust . ./
RUN cargo build --release

# final image
FROM alpine:latest

LABEL name="ssd-benchmark"
LABEL org.opencontainers.image.source="https://github.com/sassman/ssd-benchmark-rs"
LABEL repository="https://github.com/sassman/ssd-benchmark-rs"
LABEL homepage="https://github.com/sassman/ssd-benchmark-rs"
LABEL maintainer="Sven Assmann"

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/ssd-benchmark \
                    /usr/local/bin/
CMD /usr/local/bin/ssd-benchmark