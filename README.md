# Alf

Alf is a program, written in Rust, that reads Apache log data from standard
input and writes it to standard output, formatted differently.

## Usage

When used with no options, it reads Apache log data, expects it to be in the
"combined" log format, splits it into useful chunks, and writes the chunks out
to standard output separated by tab characters with pretty colors for easy
reading.

See [the Apache documentation](https://httpd.apache.org/docs/2.4/mod/mod_log_config.html#examples) for info on a few common log formats.

Pass `-h` or `--help` to `alf` to get a list of all options.

Alf is best used in combination with other tools. Some examples:

```bash
# Get the top 10 browsers
cat access.log | alf -f useragent | sort | uniq -c | sort -n | tail
# Get a list of IP addresses who visited your blog
cat access.log | alf -f ip request -d '|' | grep "blog" | cut -d'|' -f1 | sort | uniq
```

Under the hood, it gets most of its performance from its assumption that Apache
log data is in ASCII format, and it should, but may not, work with UTF-8 data
correctly. Having said that UTF-8 encoded input seems to be escaped by Apache,
although I have not been able to verify this.

## Fields

Depending on the log format, different fields may be available. List them with
Alf's `-l` flag, for example:

```bash
alf -l
alf --list-fields --format common
```

The `request` field is a special case. It is a field that consists of three
other fields, namely `method`, `uri`, and `http`. If you don't list any fields
with the `-f`/`--fields` option, then `request` is not listed and instead the
three "sub-fields" are listed.

If you do use the `-f`/`--fields` option to list fields, then the `request`
field as well of its subfields are available for use. 
