FROM ubuntu:latest

ENV DICDIR=/var/lib/mecab/dic/mydic
VOLUME $DICDIR

RUN apt-get update && apt-get install -y \
  mecab libmecab-dev sudo git make make curl unzip xz-utils file

WORKDIR /usr/src/app

RUN git clone --depth 1 https://github.com/neologd/mecab-unidic-neologd.git
RUN cd mecab-unidic-neologd && \
  ./libexec/install-mecab-unidic.sh && \
  ./bin/install-mecab-unidic-neologd -y -n -p $DICDIR

