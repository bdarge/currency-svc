# STAGE1: Build the binary
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache build-base musl-dev unzip curl \
    && PROTOC_ZIP=protoc-23.4-linux-aarch_64.zip \
    && curl -OL https://github.com/google/protobuf/releases/download/v23.4/$PROTOC_ZIP \
    && unzip -o $PROTOC_ZIP -d /usr/local bin/protoc \
    && rm -f $PROTOC_ZIP

WORKDIR /app

# copy source files
COPY . .

RUN cargo build --release

# STAGE2: create a slim image with the compiled binary
FROM alpine AS runner

# Copy the binary from the builder stage
WORKDIR /app
COPY --from=builder /app/target/release/currency_svc app

CMD ["./app"]
