# Rust as the base image
FROM rust:1.70 as build

ENV APPNAME=golem

# Create a new empty shell project
RUN USER=root cargo new --bin golem
WORKDIR /golem

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build
RUN rm ./target/debug/deps/golem*
RUN cargo build

# The final base image
FROM debian:bullseye-slim

# Copy from the previous build
COPY --from=build /golem/target/debug/golem /usr/app/golem

# Copy ./templates and ./public
COPY ./templates /templates
COPY ./public /public

EXPOSE 7878

# Run the binary
CMD "/usr/app/golem"
