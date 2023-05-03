FROM rust:1.69 as build

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim as run

RUN apt update && apt install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=build /app/target/release/project-overwatch-scanner .

CMD [ "./project-overwatch-scanner" ]


