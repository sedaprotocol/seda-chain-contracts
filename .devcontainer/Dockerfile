# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bullseye"
FROM rust:1-${VARIANT}

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
	# Remove imagemagick due to https://security-tracker.debian.org/tracker/CVE-2019-10131
	&& apt-get purge -y imagemagick imagemagick-6-common

# Tools
RUN apt-get -y install --no-install-recommends vim git docker.io libc6 build-essential

# WASM dependencies + Rustup
RUN apt-get -y install --no-install-recommends binaryen
RUN rustup target add wasm32-wasi
RUN rustup target add wasm32-unknown-unknown

# Rustfmt with nightly toolchain
RUN rustup install nightly
RUN rustup component add rustfmt --toolchain nightly

# Dependencies
RUN apt-get -y install --no-install-recommends protobuf-compiler

COPY ./download-seda.sh ./download-seda.sh
RUN bash ./download-seda.sh
