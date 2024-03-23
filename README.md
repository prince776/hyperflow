# HyperFlow

A basic implementation of HTTP2 in rust. This implementation receives http2 requests and
provides them to a handler function in http 1.1 semantics. Response returned by the
handler in http 1.1 semantics is then converted to http2 and sent.

This implementation has been tested with a basic GET and POST request via `h2load`.

```
❯ h2load http://localhost:8080 -d Cargo.toml -v
starting benchmark...
spawning thread #0: 1 total client(s). 1 total requests
Application protocol: h2c
[stream_id=1] :status: 200
[stream_id=1] content-length: 18
progress: 100% done

finished in 2.31ms, 432.53 req/s, 21.54KB/s
requests: 1 total, 1 started, 1 done, 1 succeeded, 0 failed, 0 errored, 0 timeout
status codes: 1 2xx, 0 3xx, 0 4xx, 0 5xx
traffic: 51B (51) total, 6B (6) headers (space savings 76.92%), 18B (18) data
                     min         max         mean         sd        +/- sd
time for request:     1.87ms      1.87ms      1.87ms         0us   100.00%
time for connect:      308us       308us       308us         0us   100.00%
time to 1st byte:     2.21ms      2.21ms      2.21ms         0us   100.00%
req/s           :     446.74      446.74      446.74        0.00   100.00%

```

See the h2load succesfully received response from our rust impl of h2.

```
Application protocol: h2c
[stream_id=1] :status: 200
[stream_id=1] content-length: 18
progress: 100% done

finished in 2.31ms, 432.53 req/s, 21.54KB/s
requests: 1 total, 1 started, 1 done, 1 succeeded, 0 failed, 0 errored, 0 timeout
```

This implementation is NOT http2 compliant, however it can be extended to do so.
I however doubt I'll continue this project as this project was mostly for learning about http2
by reading the rfc and practicing some rust.

## Development

```
$ cargo run # to run the server
$ h2load http://localhost:8080 -v# to make http2 req
$ sudo tshark -i any -f "tcp port 8080" -Y "http2" -V # to monitor http2 requests
$ ~/Downloads/h2spec -p 8080 http2 # for checking compliance, you might have it somewhere else.
```
