# Finetime: Accurate, flexible, and efficient time keeping
`finetime` is a Rust library for accurate, flexible, and efficient timekeeping, designed for applications where precision and performance are critical.
- **Accurate**: Supports exact arithmetic with attosecond-level precision over arbitrary time ranges, without sacrificing correctness or performance.
- **Flexible**: Built on Rust generics, `finetime` allows durations and time points to be expressed as integers or floats of any native Rust bitwidth, in any SI time unit, and using any time scale.
- **Efficient**: Represents time values as tick counts since an epoch, enabling compact storage and fast processing without conversion overhead.
- **Verified**: Key correctness properties have been formally proven using the [`Kani` model checker](https://model-checking.github.io/kani/), ensuring a high degree of reliability.
- **Portable**: The `finetime` library is fully `no_std`, such that it may be used even in bare metal environments.

With this fine degree of control and precision, `finetime` is suitable for all types of applications, from nanoseconds in embedded systems to femtoseconds in scientific computing, or picoseconds for precise orbit determination.

## Getting started
`finetime` requires the `cargo` build system for the Rust programming language to be present on your system. The library may be added as dependency for your Rust project by running `cargo add finetime`.