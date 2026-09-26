# addressline

Most address data I run into is a single line of free text pulled out of a
spreadsheet or a form submission: `12 Elm St, Springfield, IL 62704`. Before
you can do anything useful with it — dedupe it, validate it, ship it to a
mailing API — you need it split into street, city, state, and zip, with the
state code checked against the real USPS list and the zip checked for shape.

`addressline` is a small Rust library that does that one job, plus a CLI
that runs it over a file or a stream.

## Library

```rust
use addressline::parse_line;

let addr = parse_line("1 Infinite Loop, Cupertino, CA 95014-2083")?;
assert_eq!(addr.city, "Cupertino");
assert_eq!(addr.state, "CA");
```

`parse_line` returns `Result<Address, ParseError>`. It accepts the
`STREET, CITY, STATE ZIP` shape — three comma-separated fields, with the
state and zip as the last two whitespace-separated tokens — or a four-field
`STREET, UNIT, CITY, STATE ZIP` shape when an apartment, unit, or suite is
broken out on its own. An apartment/unit folded directly into the street
text (`12 Elm St Apt 4`) is left alone and stays part of `street`. It
normalizes the state code to upper case but leaves street, unit, and city
casing alone.

## CLI

Build it with `cargo build --release`; the binary is `addressline`.

Read from a file:

```
$ addressline addresses.txt
12 Elm St, Springfield, IL 62704
1 Infinite Loop, Cupertino, CA 95014-2083
```

Read from stdin, same as giving no file at all:

```
$ cat addresses.txt | addressline
$ addressline < addresses.txt
```

Mix files and stdin by using `-` for the stream:

```
$ cat more.txt | addressline addresses.txt -
```

Bad lines are reported to stderr with the source and line number and don't
stop the rest of the run:

```
$ printf '12 Elm St, Springfield, IL 62704\nnot an address\n' | addressline
12 Elm St, Springfield, IL 62704
addressline: stdin:2: expected "street, city, state zip" (three comma-separated fields)
```

The process exits non-zero if any line failed to parse.

## Status

Early. Handles the plain `STREET, CITY, STATE ZIP` shape and an optional
unit field — see the open questions in the issue tracker for what's next
(CSV output, recipient name lines, non-US addresses).
