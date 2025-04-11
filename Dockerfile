FROM docker.io/rust:1-slim-bookworm AS build

## cargo package name from Cargo.toml
ARG pkg=eventsphere-be

WORKDIR /build

COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./main

################################################################################

FROM docker.io/debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

## Create uploads directory for image storage
RUN mkdir -p /app/uploads

## Set required environment variables
ENV UPLOADS_DIR=/app/uploads
ENV ROCKET_ENV=production
ENV MEDIA_BASE_URL=/uploads

## copy the main binary
COPY --from=build /build/main ./

## copy configuration files
COPY --from=build /build/Rocket.toml ./

## Keep port as 8000 to match your Rocket.toml configuration
EXPOSE 8000

CMD ["./main"]