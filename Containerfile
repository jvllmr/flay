FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:x86_64-musl AS builder-amd64

FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:i686-musl AS builder-386

FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:aarch64-musl AS builder-aarch64

FROM --platform=$BUILDPLATFORM	ghcr.io/rust-cross/rust-musl-cross:armv7-musleabi AS builder-armv7

FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:s390x-musl AS builder-s390x

FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:powerpc64le-musl AS builder-ppc64le

FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/rust-musl-cross:riscv64 AS builder-riscv64

ARG BUILDARCH

FROM builder-$BUILDARCH AS builder

RUN apt update

RUN apt install python3 python3-dev python3-pip -y
RUN --mount=type=cache,target=/root/.cache/pip pip install pdm maturin
WORKDIR /app
COPY README.md Cargo.toml Cargo.lock pyproject.toml pdm.lock ./
COPY rust ./rust
COPY src ./src
RUN --mount=type=cache,target=/root/.cache/pip \
    pdm install --frozen-lock --no-self
RUN --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    maturin develop --release --locked
RUN pdm run flay bundle flay
FROM docker.io/python:3.13-alpine AS runner
WORKDIR /app
COPY --from=builder /app/flayed/flay ./flay
