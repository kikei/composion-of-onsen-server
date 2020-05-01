FROM rust:1.38

ARG UID
ARG GID
ARG UNAME

ENV UID $UID
ENV GID $GID
ENV UNAME $UNAME
ENV DATADIR /mnt
ENV DICDIR=/var/lib/mecab/dic/mydic

COPY . /usr/src/app

# Sometimes target causes error
RUN rm -rf /usr/src/app/target

VOLUME /usr/src/app
VOLUME $DICDIR

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y \
  mecab libmecab-dev

# User setup
RUN groupadd -g $GID $UNAME
RUN useradd -u $UID -g $UNAME -m $UNAME

USER $UNAME

ENV HOME /home/$UNAME
ENV CARGO_HOME /$HOME/.cargo
ENV PATH $PATH:${CARGO_HOME}/bin

# Mecab setup; use mydic
RUN grep -v dicdir /etc/mecabrc > ~/.mecabrc
RUN echo "dicdir = $DICDIR" >> ~/.mecabrc

# Cargo setup

# Following command will be permission error
# as it tries to create target directory
# but the copied directory is owned by root,
# RUN cargo install --path .
# So I copy it and install cargo dependencies as workaround.
RUN cp -a /usr/src/app $HOME
RUN cd $HOME/app && cargo install --path .

ENTRYPOINT ["cargo", "run", "--release"]