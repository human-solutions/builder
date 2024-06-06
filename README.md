# CSS Stacker

Simple program to stack css|scss|sass files into a single css file

**Usage:** `css-stacker [OPTIONS]`

## Installation

```bash
cargo install --locked --git https://github.com/anotherempty/css-stacker
```

## Options

* `-p, --path <PATH>`      Path to the directory containing the styles *[default: ./]*
* `-o, --output <OUTPUT>`  Name with path of the output css file without the extension *[default: ./style (which creates ./style.css)]*
* `-f, --format <FORMAT>`  Format of the output file *[possible values: minified, pretty]*, if none is provided, both format will be created
* `-h, --help`             Print help
* `-V, --version`          Print version

## Example

```bash
css-stacker -p ./src/styles -o ./dist/style -f minified
```

 Will output `.dist/style.min.css`
