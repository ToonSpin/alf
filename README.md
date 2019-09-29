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

Depending on the log format, the following fields may be available. In the
default format, called `combined`, all of these are available except `vhost`.

* `vhost`
* `ip`
* `rfc1413`
* `username`
* `time`
* `method`
* `request`
* `http`
* `status`
* `responsesize`
* `referer`
* `useragent`
