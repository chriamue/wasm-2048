FROM rust:1.73

WORKDIR /app

RUN cargo install wasm-pack

COPY . .

RUN wasm-pack build --target web --out-name wasm --out-dir ./static

WORKDIR /app/static

CMD ["python", "-m", "SimpleHTTPServer"]