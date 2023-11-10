# 10x-weather
A service for getting a weather report over http

## Layout
- `src` Contains the rust project that runs the actual weather server
- `tests` Contains a node test suite that verifies the behavior of the server

## Run it
To run the server and the test suite, run the script at the top level of the repo

```
./weather.sh
```

This will 

1. Build the docker images
2. Start the server and the test suite in a private network
3. Run the test suite
4. Start the server again open to the host on port 3000

