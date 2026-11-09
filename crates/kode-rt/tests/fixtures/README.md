# Test certificates

`localhost.der` and `localhost-key.der` are a self-signed certificate and its
PKCS#8 private key for `DNS:localhost`, valid until 2036. They are used only by
the TLS tests in `src/tls.rs` and `src/http_client.rs`; no compiled program
contains them.

Regenerate them (they are throwaway, so the key is committed on purpose):

```sh
openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes \
  -keyout localhost.key -out localhost.crt \
  -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost"
openssl x509 -in localhost.crt -outform der -out localhost.der
openssl pkcs8 -topk8 -nocrypt -in localhost.key -outform der -out localhost-key.der
```
