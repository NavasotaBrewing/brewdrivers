FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

ENV PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig"
ENV OPENSSL_DIR="/usr/lib/ssl"
ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl" 
ENV OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu" 
ENV PKG_CONFIG_ALLOW_CROSS=1
# ENV CC=arm-linux-gnueabihf-gcc

RUN dpkg --add-architecture arm64
RUN apt-get update
RUN apt-get install pkg-config -y
RUN apt-get install libudev-dev:arm64 -y
RUN apt-get install libssl-dev:arm64 -y 