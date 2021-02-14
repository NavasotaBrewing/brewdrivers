# Cross compile
This is a log of me trying to get cross compilation to work with the `cross` rust tool and Docker.

## Links
* https://www.reddit.com/r/rust/comments/g86tc1/help_crosscompiling_for_raspberry_pi_zero_w/
* https://www.acmesystems.it/arm9_toolchain
* https://github.com/rust-lang/rust/issues/28924

The Dockerfile is almost 100% from this [reddit post](https://www.reddit.com/r/rust/comments/g86tc1/help_crosscompiling_for_raspberry_pi_zero_w/)

Here's the current Dockerfile, it seems to be compiling ok:
```Dockerfile
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
```

And here's my current `Cross.toml`
```toml
[build.env]
passthrough = [
    # "PKG_CONFIG_PATH",
    "RUST_BACKTRACE"
]

[target.armv7-unknown-linux-gnueabihf]
image = "navasotabrewing/rtu-compiler:0.2.1"
# linker = "arm-none-linux-gnueabihf-gcc"
linker = "arm-linux-gnueabihf-gcc"
```

`Cargo.toml` is standard, except I added OpenSSL as a vendored dependency
```
openssl = { version = "0.10.29", features = ["vendored"] }
```



I added the following in `.cargo/config` (in the project directory, not home)
```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```
Not sure if that did anything.


It finally compiled, i `scp`ed it to the pi, and it won't run. Fuck this, i give up. I've lost 2 days to this.