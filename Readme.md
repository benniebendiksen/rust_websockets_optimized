# Arbitrage processing

## Build

To build the executable, run the command:

```bash
cargo build --release
```

All executable files will be located in `./target/release/` directory.

## Run

Prepare a configuration file based on the example of the `config.json.example`
file

```bash
./arbitrage-processing --config path/to/config.json --amount 10
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

Run python script with `python test_rust.py`.

## Mock tests

You can run `actix-server` in one terminal emulator, change `url` to
`http://localhost:50000` in the `config.json` file, and send requests to the
server.

```bash
# From one terminal emulator
./actix-ws-server

# From another terminal emulator
./arbitrage-processing --config path/to/config.json --amount 100
```

## API implementation

To implement new API endpoints, you should create a request structure, that can
be serialized into a valid query and implements the `BinanceRequest` trait, the
`BinancePayload` trait, if necessary, and a response structure, that can be
deserialized from the corresponded server response and implements the
`BinanceOkResponse` trait.
