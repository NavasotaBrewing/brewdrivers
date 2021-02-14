FROM rustembedded/cross:arm-unknown-linux-gnueabihf-0.2.1
ENV DEBIAN_FRONTEND=noninteractive
ENV PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig
ENV RPI_TOOLS=/rpi_tools
ENV MACHINE=armv6
ENV ARCH=armv6
ENV CC=gcc
ENV OPENSSL_DIR=/openssl
ENV CROSSCOMP_DIR=/rpi_tools/arm-bcm2708/arm-rpi-4.9.3-linux-gnueabihf/bin
ENV INSTALL_DIR=/tmp/openssl

RUN dpkg --add-architecture armhf
RUN apt-get update &&\
    apt-get install -y wget gcc-arm-linux-gnueabihf openssl:armhf libssl-dev:armhf pkg-config libudev-dev:armhf

# Get Raspberry Pi cross-compiler tools
RUN git -C "/" clone -q --depth=1 https://github.com/raspberrypi/tools.git "${RPI_TOOLS}"


# Manually cross-compile OpenSSL to link with

# 1) Download OpenSSL 1.1.0
RUN mkdir -p $OPENSSL_DIR
RUN cd /tmp &&\
    wget --no-check-certificate https://www.openssl.org/source/openssl-1.1.0h.tar.gz &&\
    tar xzf openssl-1.1.0h.tar.gz

# 2) Compile
RUN cd /tmp/openssl-1.1.0h &&\
    mkdir $INSTALL_DIR && \
    ./Configure linux-generic32 shared\
      --prefix=$INSTALL_DIR --openssldir=$OPENSSL_DIR/openssl\
      --cross-compile-prefix=$CROSSCOMP_DIR/arm-linux-gnueabihf- &&\
      make depend && make && make install