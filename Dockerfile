FROM ubuntu:20.04 as build-env
RUN apt-get update && apt-get -y install sudo
RUN sudo apt install -y git clang curl libssl-dev llvm libudev-dev build-essential
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo --version
RUN rustup toolchain add nightly-2022-01-16
RUN rustup default nightly-2022-01-16
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2022-01-16
RUN cargo install cargo-chef
RUN rustup show

# FROM build-env AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM build-env as cacher
# RUN cargo install cargo-chef
# COPY --from=planner /recipe.json recipe.json
# RUN cargo chef cook --release --recipe-path recipe.json

FROM build-env as builder
COPY . .
# COPY --from=cacher /target target
RUN cargo test --all
RUN cargo clean
RUN cargo build --release
