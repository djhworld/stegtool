## Steganography tool

Encode data into PNG images!

    stegtool 0.1
    Daniel Harper
    Steganography tool

    USAGE:
        stegtool [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        decode    decodes data from an image (if available!)
        encode    encodes data into a cover image
        help      Prints this message or the help of the given subcommand(s)

### Building

Note building requires the [`steg`](https://github.com/djhworld/steg) library to be available in `../`

```cargo build```
