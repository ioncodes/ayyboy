FROM rustlang/rust:nightly

RUN dpkg --add-architecture amd64
RUN apt update && apt upgrade -y
RUN apt install -y g++-mingw-w64-x86-64
RUN apt install -y g++-x86-64-linux-gnu
RUN apt install -y libasound2-dev:amd64 pkg-config
RUN apt install -y libdbus-1-dev:amd64

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install stable-x86_64-pc-windows-gnu
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN export PKG_CONFIG_PATH=$(pkg-config --variable pc_path pkg-config | tr ':' '\n' | grep -E 'x86_64-linux-gnu' | head -n 1) && echo $PKG_CONFIG_PATH
ENV PKG_CONFIG_PATH /usr/lib/x86_64-linux-gnu/pkgconfig:/usr/local/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig

WORKDIR /app

CMD ["/bin/bash", "-c", "cargo build --release --target x86_64-pc-windows-gnu && cargo build --release --target x86_64-unknown-linux-gnu"]