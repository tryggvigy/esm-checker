# Debugging

## Running the web server with a persistent debug directory

Create a debug directory:

```bash
mkdir debug_dir
```

Run the web server:

```bash
DEBUG_DIR="<abs_path_to_debug_dir>" RUST_LOG="trace" cargo run -p web_server
```

## Running rusty is_esm_ready_yet on a repo and print to console

```bash
cargo run -p is_esm_ready_yet --release -- -p <rel/abs_path_to_package.json> | grep "react"
```

## Running rusty is_esm_ready_yet on a repo and generate report in a file

```bash
cargo run -p is_esm_ready_yet --release -- -p <rel/abs_path_to_package.json> -o out.json
```

---

# Misc
- `RUST_LOG=info,walk_imports=debug cargo run`