FROM rust:1.38

VOLUME /usr/src/app

WORKDIR /usr/src/app
COPY . /usr/src/app

RUN apt-get update
RUN apt-get install -y mecab libmecab-dev mecab-ipadic-utf8 git curl xz-utils file

RUN cargo install --path .
RUN cargo install systemfd cargo-watch
