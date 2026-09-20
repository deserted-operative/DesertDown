FROM mcr.microsoft.com/devcontainers/rust:latest

ARG CMARK_VERSION=0.31.2

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        cmake \
        git \
        linux-perf \
        lldb \
        ninja-build \
        pkg-config \
        python3 \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install flamegraph --locked

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        llvm-dev \
        libclang-dev \
        lld \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-afl