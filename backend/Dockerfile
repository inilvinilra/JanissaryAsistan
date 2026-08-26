FROM rust:1.97-bookworm AS builder
WORKDIR /workspace
# Dependencies are built from the manifests alone first, so editing a source
# file rebuilds only this crate instead of every dependency again.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY src ./src
# Cargo skips a rebuild when the mtime is older than the artefact it produced
# for the placeholder above.
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates clamav tesseract-ocr tesseract-ocr-eng tesseract-ocr-tur \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/backend /usr/local/bin/jury-backend
COPY samples ./samples

# The service handles files from the public internet — uploaded reports are
# parsed, and scanned by a subprocess — so it does not run as root. The uploads
# directory is created here because the volume mounted over it at runtime
# inherits this ownership.
RUN useradd --system --uid 10001 --create-home --home-dir /home/jury jury \
    && mkdir -p /app/uploads \
    && chown -R jury:jury /app
USER jury

ENV APP_ENV=production
EXPOSE 3000
CMD ["jury-backend"]
