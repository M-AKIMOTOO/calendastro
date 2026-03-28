# calendastro

`calendastro` is a desktop calendar application written in Rust with `eframe`/`egui`.

The app displays:

- the current JST and UT clocks
- a month calendar
- the day-of-year for each date
- the Modified Julian Date (MJD) for each date

## Requirements

- Rust toolchain
- a Linux desktop environment capable of running an `eframe` window

## Run

```bash
cargo run
```

## Build Release

```bash
cargo build --release
```

## Controls

- `<< Prev`: move to the previous month
- `Today`: jump to the current month
- `Next >>`: move to the next month
- `Year` / `Month` + `Go`: jump to a specific month
