FROM rust:1.74-slim-buster as build

WORKDIR /app

RUN apt update 
RUN apt install -y pkg-config libssl-dev

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release

FROM rust:1.74-slim-buster 

COPY --from=build /app/target/release/controller .

CMD ["./controller"]
