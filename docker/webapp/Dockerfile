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

# Setup templates
/target/rerelease/onsen-compo template add --id default \
  --name "Default template" \
  --path /usr/src/data/templates/default.html

/target/rerelease/onsen-compo template add --id ghost \
  --name "For ghost blog" \
  --path /usr/src/data/templates/ghost.html

ENTRYPOINT ["/target/release/onsen-compose", "app"]
