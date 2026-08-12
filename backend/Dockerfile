FROM rust:1.97-bookworm AS builder
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates clamav tesseract-ocr tesseract-ocr-eng tesseract-ocr-tur && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/backend /usr/local/bin/jury-backend
COPY samples ./samples
ENV APP_ENV=production
EXPOSE 3000
CMD ["jury-backend"]
