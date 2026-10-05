# Throwaway TLS fixtures

`tests/native_roots.rs` runs an HTTPS server on `127.0.0.1` and checks that
Intern's clients trust a private certificate authority the operating system
offers them (through `SSL_CERT_FILE`, which rustls-native-certs reads instead
of the platform store) and refuse the same server without it.

- `ca.pem` - a self-signed CA certificate that exists only for that test.
- `server.pem` - a certificate for `localhost` and `127.0.0.1`, signed by it.
- `server.key` - the server certificate's private key.

None of these protects anything. The CA's own key was deleted after signing,
so nothing else can ever be issued under it, and no machine trusts it unless a
test points `SSL_CERT_FILE` at it. All three are valid for 100 years.

To regenerate them (OpenSSL 3), from this directory:

```sh
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes \
  -keyout ca.key -out ca.pem -days 36500 \
  -subj "/CN=Intern throwaway test CA" \
  -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,keyCertSign,cRLSign"
openssl req -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes \
  -keyout server.key -out server.csr -subj "/CN=localhost"
printf '%s\n' \
  'subjectAltName=DNS:localhost,IP:127.0.0.1' \
  'basicConstraints=critical,CA:FALSE' \
  'keyUsage=critical,digitalSignature' \
  'extendedKeyUsage=serverAuth' > server.ext
openssl x509 -req -in server.csr -CA ca.pem -CAkey ca.key -set_serial 2 \
  -days 36500 -sha256 -extfile server.ext -out server.pem
rm ca.key server.csr server.ext
```
