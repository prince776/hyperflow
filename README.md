# HyperFlow

An HTTP2 impl in rust.

## Development

```
$ cargo run # to run the server
$ h2load http://localhost:8080 # to make http2 req
$ sudo tshark -i any -f "tcp port 8080" -Y "http2" -V # to monitor http2 requests
```
