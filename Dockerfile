FROM docker.io/rust:1-slim-bullseye AS build

## cargo package name: customize here or provide via --build-arg
ARG pkg=eventsphere-be
ARG DATABASE_URL_ARG # Define ARG for database URL

WORKDIR /build

COPY . .

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install sqlx-cli --version="0.7.4" --locked # Pin version and use --locked

ENV DATABASE_URL=${DATABASE_URL_ARG}
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/.sqlx \
    sh -c ' \
    set -eux; \
    if [ -z "$DATABASE_URL" ]; then \
        echo "Warning: DATABASE_URL build argument is not set. '\''cargo sqlx prepare'\'' might fail if the .sqlx cache needs to be generated or updated."; \
    fi; \
    echo "Running cargo sqlx prepare..."; \
    cargo sqlx prepare; \
    echo "Running cargo build --release..."; \
    cargo build --release; \
    echo "Running objcopy..."; \
    objcopy --compress-debug-sections target/release/$pkg ./main \
    '

################################################################################

FROM docker.io/debian:bullseye-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /build/main ./

COPY --from=build /build/Rocket.tom[l] ./static
COPY --from=build /build/stati[c] ./static
COPY --from=build /build/template[s] ./templates

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80

EXPOSE 80

CMD ./main

## docker build --build-arg DATABASE_URL_ARG="postgresql://postgres.adlpwvpelbuntzrwrwvj:jQZSfNlXRNRtVHzF@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres" -t eventsphere .