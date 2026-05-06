FROM rust:latest AS build

# create a new empty shell project
RUN USER=root cargo new --bin stravai_oxide
WORKDIR /stravai_oxide

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src
COPY ./templates ./templates
COPY ./migrations ./migrations

# build for release
RUN rm ./target/release/deps/stravai_oxide*
RUN cargo build --release

# our final base
FROM rust:latest

# copy the build artifact from the build stage
COPY --from=build /stravai_oxide/target/release/stravai_oxide .
COPY ./assets ./assets
COPY ./.env ./.env

# set the startup command to run your binary
CMD ["./stravai_oxide"]