# BUILD STAGE
FROM rust:alpine as builder

COPY . /build
WORKDIR /build

RUN apk add --no-cache musl-dev lld && \
    rustup install nightly && \
    rustup override set nightly && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-musl && \
    rustup component add llvm-tools-preview --toolchain nightly && \
    cargo install --locked bootimage && \
    cargo bootimage

# RUNTIME STAGE
FROM debian:bullseye

WORKDIR /app

COPY --from=builder /build/target/x86_64-ruost/debug/bootimage-ruost.bin /app/

RUN apt-get update && \
    apt-get install -y --no-install-recommends qemu qemu-system-x86 && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 5900

ENTRYPOINT ["qemu-system-x86_64", "-drive", "format=raw,file=bootimage-ruost.bin", "-vnc", ":0", "-display", "none"]
