# SM64 Press Start Head Data Dumper
A tool to dump the various data structures used in the Mario head screen in Super Mario 64.

## Usage
```
USAGE:
    facevtxdump [FLAGS] [OPTIONS] <input> <addr> [output]

FLAGS:
    -a, --animation    Look for animation data (linked in NodeGroup)
    -f, --face         Look for face data (linked in PlaneGroup)
    -h, --help         Prints help information
    -V, --version      Prints version information
    -v, --vertex       Look for vertex data (linked in NodeGroup)

OPTIONS:
    -r, --ram <ram>    RAM address of start of file
    -w <width>         Min padding between values in output array

ARGS:
    <input>     Input binary
    <addr>      Address of Face or Vtx data info struct
    <output>    Output file, or stdout if not present
```