FROM rustlang/rust:nightly as builder
WORKDIR /usr/local/src/
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/monolith /usr/local/bin/monolith

CMD ["monolith"]
