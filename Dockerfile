FROM rustlang/rust:nightly

RUN apt update && apt upgrade -y
RUN apt install -y g++-mingw-w64-x86-64
RUN apt install -y g++-x86-64-linux-gnu

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install stable-x86_64-pc-windows-gnu
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc

WORKDIR /app

CMD ["/bin/bash", "-c", "cargo build --release --target x86_64-pc-windows-gnu && cargo build --release --target x86_64-unknown-linux-gnu"]