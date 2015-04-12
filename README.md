# Small logger

- reads stdout and stdin of your process
- adds a timestamps
- spits the result in a file that is composed of single JSON lines for later parsing
- try never to crash

# How to use ?

```
small-logger your-program-name and its arguments
```

# TODO

- make log folder location a parameter
- add MARK syslog like to indicate process is still alive when idle for a long time.
