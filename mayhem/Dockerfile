FROM --platform=linux/amd64 rustlang/rust:nightly as builder

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update 
RUN apt-get install -y cmake clang
RUN cargo install cargo-fuzz

ADD . /charabia/

WORKDIR /charabia/fuzz/

RUN cargo +nightly fuzz build

FROM --platform=linux/amd64 rustlang/rust:nightly

COPY --from=builder /charabia/fuzz/target/x86_64-unknown-linux-gnu/release/tokenize /

COPY --from=builder /charabia/fuzz/target/x86_64-unknown-linux-gnu/release/segment /