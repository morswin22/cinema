# Use the latest stable Rust image
FROM rust:1.82 as builder

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

# Create project directory
WORKDIR /app

# Copy your project files here (optional if building a project)
COPY . .

# Final image (you can make it slimmer with multi-stage builds if needed)
CMD ["/app/start.sh"]
