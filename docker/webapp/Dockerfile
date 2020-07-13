FROM rust:1.44

ENV DICDIR /var/lib/mecab/dic/mydic
ENV CARGO_TARGET_DIR /target

VOLUME /var/lib/mecab/dic/mydic
VOLUME /usr/src

# For build
WORKDIR /usr/src

# Install dependencies
RUN apt-get update && apt-get install -y \
  mecab libmecab-dev

# Mecab setup; use mydic
RUN grep -v dicdir /etc/mecabrc > ~/.mecabrc
RUN echo "dicdir = $DICDIR" >> ~/.mecabrc

ENTRYPOINT ["/target/release/onsen-compose", "app"]
