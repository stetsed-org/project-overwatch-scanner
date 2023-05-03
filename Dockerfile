FROM rust:1.69 as build

WORKDIR /app

COPY . .

RUN cargo build --release

FROM archlinux:latest as run

WORKDIR /app

COPY --from=build /app/target/release/project-overwatch-scanner .

VOLUME /app/data

CMD [ "cd /app/data && ../project-overwatch-scanner" ]


