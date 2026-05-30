# rust-mouse

[![PyPI](https://img.shields.io/pypi/v/rust-mouse.svg)](https://pypi.org/project/rust-mouse/)
[![Python versions](https://img.shields.io/pypi/pyversions/rust-mouse.svg)](https://pypi.org/project/rust-mouse/)
[![Built with maturin](https://img.shields.io/badge/built%20with-maturin-blue.svg)](https://github.com/PyO3/maturin)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Realistic, human-like mouse movement for Python, powered by Rust.

`rust-mouse` moves the cursor along organic, hard-to-distinguish-from-human paths instead of teleporting or following a straight line.
The motion engine is written in Rust (via [PyO3](https://pyo3.rs) and [enigo](https://github.com/enigo-rs/enigo)) for precise, low-jitter timing, and exposed to Python as a single, simple function.

## Features

- **Bézier curve paths** — movement follows a randomized cubic Bézier curve rather than a straight line.
- **Natural easing** — one of six "ease-out" functions is picked per movement for a realistic acceleration/deceleration profile.
- **Fitts's law timing** — when no duration is given, movement time is derived from distance and target size, the way real pointing behaves.
- **Tremor & micro-jitter** — multi-octave noise adds subtle hand tremor that damps out as the cursor approaches the target.
- **Overshoot & corrections** — optionally overshoots the target and corrects back, like a real hand.
- **Occasional mistakes** — small random deviations and re-corrections appear at a low probability.
- **Mid-movement pauses** — longer movements may pause with a slight wobble, as a human hesitates.
- **Arrival drift** — small settling movements around the target before the cursor comes to rest.
- **High-precision sleeps** — uses [`spin_sleep`](https://crates.io/crates/spin_sleep) for accurate sub-millisecond step timing.

## Installation

```bash
pip install rust-mouse
```

Prebuilt wheels will be published for Linux, macOS, and Windows on CPython and PyPy (Python 3.10+).

## Usage

```python
from rust_mouse import move_mouse_human

# Move to (800, 450) with automatic, distance-based timing.
move_mouse_human(800, 450)

# Move with an explicit duration of 1.2 seconds and a wider curve.
move_mouse_human(1200, 700, duration_ms=1200, deviation=80.0)

# Force an overshoot-and-correct motion.
move_mouse_human(300, 300, overshoot=True)
```

## API

### `move_mouse_human(target_x, target_y, duration_ms=None, deviation=50.0, overshoot=False, allow_pauses=None)`

Moves the cursor from its current position to `(target_x, target_y)` along a
human-like path. Blocks until the movement is complete.

| Parameter     | Type            | Default | Description |
|---------------|-----------------|---------|-------------|
| `target_x`    | `float`         | —       | Destination X coordinate, in absolute screen pixels. |
| `target_y`    | `float`         | —       | Destination Y coordinate, in absolute screen pixels. |
| `duration_ms` | `int \| None`   | `None`  | Total movement time in milliseconds. When `None`, the duration is estimated from distance using Fitts's law. |
| `deviation`   | `float`         | `50.0`  | Maximum sideways curve deviation, in pixels. Larger values produce wider arcs. |
| `overshoot`   | `bool`          | `False` | When `True`, the cursor deliberately overshoots the target and corrects back. |
| `allow_pauses`| `bool \| None`  | `None`  | Enable brief mid-movement pauses. When `None`, pauses are enabled automatically for movements longer than 500 px. |

**Returns:** `None`. **Raises:** `RuntimeError` if the platform input backend cannot be initialized.

> **Note:** Coordinates are absolute and not clamped to your screen bounds — pass
> values that fall within your active display.

## How it works

Each call composes several layers of motion:

1. A short randomized initial delay (50–200 ms) before the movement begins.
2. A cubic Bézier curve is built from the start and end points with two randomized control points, giving the path a natural arc.
3. The path is sampled over time using a randomly selected ease-out function, so the cursor accelerates and decelerates believably.
4. Multi-octave tremor noise is layered on top of each sampled point and damped as the cursor nears the target.
5. Optional overshoot, mistakes, mid-movement pauses, and final arrival drift are applied depending on distance and randomness.

## Building from source

Requires a [Rust toolchain](https://rustup.rs) and [maturin](https://github.com/PyO3/maturin).

```bash
# Install the build tool
pip install maturin

# Build and install into the current virtual environment
maturin develop --release

# Or build a wheel
maturin build --release
```

## Platform notes

`rust-mouse` controls the real system cursor through the OS input APIs, so the
calling process needs permission to do so:

- **Linux** — works under X11 out of the box. Wayland support depends on the compositor.
- **macOS** — grant the terminal/app **Accessibility** permission under *System Settings → Privacy & Security*.
- **Windows** — works without additional setup.

## License

Released under the [MIT License](LICENSE).
