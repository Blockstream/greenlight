
# Bindings

for python, swift, kotlin, ruby

# Generate 

```
cargo build
cargo run --bin generate-bindings
```

# Test python

```
cd bindings/python
LD_LIBRARY_PATH=../../../target/debug/ python3

>>> import gl_client
>>> t = gl_client.default_tls_config()
>>> gl_client.print_tls(t)
'[45, 45, 45, 45, 45, 66, 69, 71, ....'

```