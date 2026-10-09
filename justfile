host := `rustc -vV | sed -n 's/host: //p'`

test-host:
    cargo test -p pico2-ice-core --target {{host}}