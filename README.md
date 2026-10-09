# pico2-ice-rs

Rust board support for the tinyVision pico2-ice (RP2350B + iCE40UP5K)

## Running the host tests

install [just](https://github.com/casey/just) with `cargo install just`
```shell
just test-host
```


Plain cargo test fails with "can't find crate for `test`", because .cargo/config.toml defaults to the RP2350's thumb target. 

`just test-host` passes the host target, and we want to run tests on the host machine (tested on MacOS and Aurora-Fedora)
