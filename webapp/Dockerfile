FROM rust:1.38

COPY . /usr/src/app

# Sometimes target causes error
RUN rm -rf /usr/src/app/target

VOLUME /usr/src/app
WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update
RUN apt-get install -y mecab libmecab-dev mecab-ipadic-utf8 git curl xz-utils file

# User setup
ARG UID
ARG GID
ARG UNAME

ENV UID $UID
ENV GID $GID
ENV UNAME $UNAME
ENV DATADIR /mnt

RUN groupadd -g $GID $UNAME
RUN useradd -u $UID -g $UNAME -m $UNAME

RUN chown $UID:$GID $DATADIR

USER $UNAME

ENV HOME /home/$UNAME
ENV CARGO_HOME /$HOME/.cargo
ENV PATH $PATH:${CARGO_HOME}/bin

# Mecab setup; make and install user dictionary
RUN sh /usr/src/app/userdic.sh

# Cargo setup

# Following command will be permission error
# as it tries to create target directory
# but the copied directory is owned by root,
# RUN cargo install --path .
# So I copy it and install cargo dependencies as workaround.
RUN cp -a /usr/src/app $HOME
RUN cd $HOME/app && cargo install --path .


# Start application server
# RUN systemfd --no-pid -s http::0.0.0.0:8088 -- cargo watch -x 'run app'
#RUN cargo install systemfd cargo-watch
#CMD systemfd --no-pid -s http::0.0.0.0:8088 -- cargo watch -x 'run app'