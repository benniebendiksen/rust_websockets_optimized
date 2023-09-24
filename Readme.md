# Arbitrage processing

## Mock server use

Run the `actix-server` main function in one terminal emulator. It is already set to localhost

```bash
# From one terminal instance
cd mock/actix-ws-server/src
cargo run
```

## Python bindings

Install maturin tool. Create and activate a virtual environment with:

```bash
python -m venv .env
source ./.env/bin/activate
```

Build the pyo3 project:

```bash
cd client
maturin build --release --bindings pyo3
maturin develop --release --bindings pyo3
```

From a second terminal instance, run the python script with `python test_rust.py`.
