# Small logger

- reads stdout and stdin of your process
- adds a timestamps
- spits the result in a file that is composed of single JSON lines for later parsing
- file is in ./logs/%Y/%m/%d/process-name-%T.ajson
- try to never crash

# How to use ?

```
small-logger your-program-name and its arguments
```

# Now includes a simple log viewer

```
small-logger -d
```

Then connect to http://localhost:5001/viewer .

# TODO

- make log folder location a parameter
- add MARK syslog like to indicate process is still alive when idle for a long time.
- Authentication in http endpoint
