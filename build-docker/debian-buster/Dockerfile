FROM rust:slim-buster

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /opt/kime

RUN apt-get update
RUN apt-get install -y build-essential git gcc clang libclang-dev cmake extra-cmake-modules pkg-config
RUN apt-get install -y libpango1.0-dev libcairo2-dev libgtk2.0-dev libgtk-3-dev libglib2.0 libxcb1 libappindicator3-dev
RUN apt-get install -y qt5-default qtbase5-dev qtbase5-private-dev libqt5gui5
RUN mkdir -pv /opt/kime-out

COPY src ./src
COPY .git ./.git

COPY Cargo.toml .
COPY Cargo.lock .
COPY LICENSE .

RUN cargo fetch

COPY res ./res
COPY ci ./ci
COPY docs ./docs
COPY scripts ./scripts

ENTRYPOINT [ "ci/build_deb.sh" ]
