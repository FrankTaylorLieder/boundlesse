# Boundlesse - A boundless life (esse) simulation

Boundlesse is a Rust-based desktop app for running Conway's Game of Life.

It's goals are:

- No artificial limits
  - The universe can be as large as needed, limited only by memory to store and
  update it.
- As fast as possible
  - Some attention has been paid to performance, but there is probably a long
  way to go!
- Explore life configrations from [LifeWiki](https://conwaylife.com/wiki).
  - You can load RLE encoded patterns to initialise the Universe.

Status:

- This is a very early version...it's UI is somewhat rough.
- There are many possible new features that could be added.


## Usage

### Building

The usual cargo build commands:

- Build: `cargo build --release`
  - Executable: `target/release/boundlesse`
- Test: `cargo test`
- Benchmarks: `cargo bench`

### Running

To run: `boundlesse [pattern]`

If provided, the pattern is loaded into the universe, without one the universe
is blank.

Patterns are RLE encoded files (see: [Run Length
Encoded](https://conwaylife.com/wiki/Run_Length_Encoded)). A set of interesting
patterns (from LifeWiki) is provided in the `patterns/` directory.

The following keys control Boundlesse:

- `<SPC>`: start/stop the simulation.
- `=`, `+`: increase the generations rate.
- `-`, `_`: decrease the generations rate.
- `<Arrow>`: move the viewport a small amount, or large amount if `<Shift>` is held too.
- `c`: center the viewport.
- `<Del>`, `<BS>`: clear the universe.
- `g`: toggle showing the grid.
- `h`: toggle showing the header.
- `a` / `s`: decrease/increase the zoom level.
- `r`: create a grid of random data 

You can control the target Generations Per Second (GPS). The application
attempts to render the UI at 60 Frames Per Second (FPS).

- If you set a GPS higher than 60, then multiple generations will be computed
per frame.
- If the configuration is large (i.e. the time to compute a generation is more
than 1/60s) then the FPS will drop.
- If the FPS is high and time to compute generations is also high then both
the achieved GPS and FPS will reduce.

### Logging

Boundlesse uses [`env_logger`](https://crates.io/crates/env_logger) for
logging. Set the `RUST_LOG` environment variable to control the logging level.

## Development

See `JOURNAL` for a log of the development of Boundlesse. This also includes a
list of possible improvements.
