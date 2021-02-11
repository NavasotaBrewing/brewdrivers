FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

RUN dpkg --add-architecture arm64
RUN apt-get update
RUN apt-get install pkg-config -y
RUN apt-get install libudev-dev -y
