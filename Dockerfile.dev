FROM rust AS builder

ARG BUILD_PROFILE=${BUILD_PROFILE:-release}

WORKDIR /src/
RUN rustup default nightly
RUN mkdir dummy
RUN echo 'fn main() {}' > dummy/dummy.rs
COPY config config
COPY data data
COPY tt tt
# Create dummy router project
RUN cargo new --bin router
COPY app/api/Cargo.toml app/api/
RUN echo "[[bin]]\nname = \"dummy\"\npath = \"/src/dummy/dummy.rs\"" >> app/api/Cargo.toml
WORKDIR /src/app/api
RUN cargo build --bin dummy --profile=$BUILD_PROFILE
RUN rm -rf dummy app/api

WORKDIR /src/
COPY app/api app/api
RUN cargo install --path app/api --profile=$BUILD_PROFILE

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y tini

COPY --from=builder /usr/local/cargo/bin/bruss_api /usr/local/bin/bruss_api

WORKDIR /app
ENV PATH=/usr/local/cargo/bin:$PATH
CMD ["tini", "--", "bruss_api"]
