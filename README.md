# Small logger

This application allow you to read another app logs (`stdout` and `stderr`) and write them to a log file.


- reads stdout and stdin of your process
- adds a timestamps
- writes the result in a file that is composed of single JSON lines for later parsing

# installing

```
cargo install small-logger
```

# How to use?

```
Usage: small-logger [-d log_folder] [-n process_name] [COMMAND WITH PARAMETERS]

Options:
    -d, --directory DIRECTORY
                        root log folder. Default is ./logs
    -n, --name PROCESS  pretty name for the process. Should be time invariant
                        and filesystem friendly. Default is process
    -c, --change_directory WORKING_DIRECTORY
                        PWD for the process. Default is .
    -h, --help          print this help menu.
```


# TODO

- add MARK syslog like to indicate process is still alive when idle for a long time.
