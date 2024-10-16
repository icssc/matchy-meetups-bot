FROM rust:alpine
COPY ./ ./
RUN cargo build --release
CMD ["./target/release/matchy_meetups_bot"]
