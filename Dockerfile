# Use the latest stable Rust image
FROM rust:1.86 as builder

# Install system dependencies for MariaDB
RUN apt-get update && apt-get install -y \
    default-libmysqlclient-dev \
    default-mysql-client \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install diesel_cli with MySQL support
RUN cargo install diesel_cli --no-default-features --features mysql

RUN cargo new playground

# Create project directory
WORKDIR /playground

COPY Cargo.toml /playground/Cargo.toml

RUN cargo build
RUN rm src/*.rs

# Copy your project files here (optional if building a project)
COPY . .

RUN cargo build

# Final image (you can make it slimmer with multi-stage builds if needed)
CMD ["/playground/start.sh"]
