# Wymagania
- [Node.js](https://nodejs.org/en)
- [Zależności systemowe](https://v2.tauri.app/start/prerequisites/#system-dependencies)
- [Rust](https://rust-lang.org/)

## Tauri CLI
```bash
# Instalacja Tauri CLI w wersji 2.x
cargo install tauri-cli --version "^2.0.0" --locked

# Instalacja Trunk
cargo install trunk

# Dodanie celu kompilacji dla WebAssembly
rustup target add wasm32-unknown-unknown
```

## Uruchomienie aplikacji
```bash
cd przepisy-main/przepisy
cargo tauri dev
```
