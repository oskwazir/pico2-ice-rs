# Notes for Claude

A `no_std` Rust crate for the tinyVision pico2-ice (RP2350B + iCE40UP5K), on `embassy-rp`.
So far it is a scaffold with one example, `rp_blinky`.

The plan and everything known about the board live in another repo, `~/Repos/hardware`.
Read these before answering questions or reviewing work here:

| For | Read, under `~/Repos/hardware/` |
|---|---|
| How the user works, the board's current state, published pages | `CLAUDE.md` |
| What to build next, and each slice's acceptance criteria | `projects/pico2-ice-dev-board/pico2-ice-build-slices.md` |
| The design, the bring-up stages, the open questions | `projects/pico2-ice-dev-board/pico2-ice-rust-crate.md` |
| Any pin, and what has been tested on the board | `projects/pico2-ice-dev-board/pico2-ice-micropython-manual.md`, sections 10 and 12 |
| The vendor C SDK this crate ports | `projects/pico2-ice-dev-board/pico-ice-sdk-reading-guide.md` |

Things that bite here:

- The user makes the commits, and reviews happen slice by slice against the acceptance
  criteria. The user is new to Rust: explain idioms.
- Flashing any example replaces the board's MicroPython firmware. Going back is a reflash
  of `pico2_ice.uf2` in boot-select mode.
- The FPGA's flash currently holds a demo gateware that drives GPIO 20, 24 and 25. Do not
  drive those from the RP2350 until slice 10 replaces it.
- The FPGA clock is GPIO 21, not the 22 the vendor header names.
- Lattice wants the CRAM load clock between 1 MHz and 25 MHz. The bit-banged loader may
  come out slower; slices 4 and 12 carry that risk.
