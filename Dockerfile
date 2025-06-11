FROM rust:1.86 as builder

RUN apt-get update && apt-get install -y \
    default-libmysqlclient-dev \
    default-mysql-client \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features mysql

RUN cargo new playground

WORKDIR /playground

COPY Cargo.toml /playground/Cargo.toml

RUN cargo build
RUN rm src/*.rs

COPY . .

RUN chmod +x start.sh

RUN cargo build

CMD ["/playground/start.sh"]
