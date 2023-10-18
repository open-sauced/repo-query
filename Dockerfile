# syntax=docker/dockerfile:1

# This dockerfile is a rust binary builder for the repo-query engine.
#
# Note: running "cargo build" will also fetch the onnx runtime ".so" file into
# cargo's target/release directory which is needed to start the server.

# This also includes libssl dependencies which are required by reqwest within
# the cargo dependency tree which uses "native-tls"
# TODO (jpmcb): It may be advantageous for us to explore replacing native-tls
# deep in the dependencies with rustls to avoid some of the nasty openssl
# remote code execution vulnerabilities.
#
# Uses the bullseye-slim debian image per the rust recommendation.
FROM --platform=$TARGETPLATFORM rust:1.73-slim-bullseye AS builder

ARG TARGETPLATFORM

# Install g++ and other build essentials for compiling openssl/tls dependencies
RUN apt update
RUN apt install -y build-essential

# Install other openssl / native tls dependencies
RUN apt-get update
RUN apt-get install -y \
  pkg-config \
  libssl-dev

# Clean up some unnecessary apt artifacts taht are not necessary
RUN rm -rf /var/lib/apt/lists/*

# Build the repo-query binary
WORKDIR /repo-query-build
COPY . .
RUN cargo build --release --all

ENV ORT_DYLIB_PATH=./target/release/libonnxruntime.so
CMD ["./target/release/open-sauced-repo-query"]
