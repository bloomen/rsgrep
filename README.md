This is a simple version of grep implemented in Rust.

Help:
```
rsgrep 0.5.0
Christian Blume
A simple version of grep implemented in Rust

USAGE:
    rsgrep [FLAGS] <string> <path>

FLAGS:
    -c, --color          Output colored strings
    -f, --followlinks    Follow links
    -h, --help           Prints help information
    -i, --insensitive    Case insensitive search
    -l, --location       Print filename and line number
    -r, --recursive      Search directories recursively
    -e, --regex          Interpret the search string as a regular expression
    -t, --relative       Print relative filenames
    -V, --version        Prints version information
    -w, --warnings       Show warnings

ARGS:
    <string>    The string to search for
    <path>      The path to search in (file or directory)
```
