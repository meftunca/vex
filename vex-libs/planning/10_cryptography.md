# Vex Stdlib Planning - 10: Cryptography

**Priority:** 10
**Status:** Partial (crypto exists, others missing)
**Dependencies:** builtin, io, big

## 📦 Packages in This Category

### 10.1 crypto (extend existing)
**Status:** ✅ Exists (extend with algorithms)
**Description:** Cryptographic primitives

#### Current Implementation
- Basic crypto functions exist

#### Required Extensions
```vex
// Hash interfaces
trait Hash {
    fn write(self: &mut Self, p: []u8): Result<usize, Error>
    fn sum(self: &Self, b: []u8): []u8
    fn reset(self: &mut Self)
    fn size(self: &Self): usize
    fn block_size(self: &Self): usize
}

// Hash functions
fn md4_new(): Hash
fn md5_new(): Hash
fn sha1_new(): Hash
fn sha224_new(): Hash
fn sha256_new(): Hash
fn sha384_new(): Hash
fn sha512_new(): Hash
fn sha512_224_new(): Hash
fn sha512_256_new(): Hash
fn sha3_224_new(): Hash
fn sha3_256_new(): Hash
fn sha3_384_new(): Hash
fn sha3_512_new(): Hash
fn shake128_new(): ShakeHash
fn shake256_new(): ShakeHash

// HMAC
fn hmac_new(h: fn(): Hash, key: []u8): Hash

// Utility functions
fn md4_sum(data: []u8): [md4.Size]u8
fn md5_sum(data: []u8): [md5.Size]u8
fn sha1_sum(data: []u8): [sha1.Size]u8
fn sha256_sum(data: []u8): [sha256.Size]u8
fn sha512_sum(data: []u8): [sha512.Size]u8
```

#### Dependencies
- builtin
- io

### 10.2 crypto/rand
**Status:** ❌ Missing (important for crypto)
**Description:** Cryptographically secure random numbers

#### Required Functions
```vex
fn int(rand: io.Reader, max: *big.Int): Result<*big.Int, Error>
fn prime(rand: io.Reader, bits: int): Result<*big.Int, Error>
fn read(b: []u8): Result<usize, Error>
```

#### Dependencies
- crypto
- big
- io

### 10.3 crypto/rsa
**Status:** ❌ Missing (asymmetric crypto)
**Description:** RSA encryption and signatures

#### Required Types
```vex
struct PublicKey {
    n: *big.Int,
    e: int,
}

struct PrivateKey {
    public_key: PublicKey,
    d: *big.Int,
    primes: []*big.Int,
    precomputed: PrecomputedValues,
}

struct PSSOptions {
    salt_length: int,
    hash: crypto.Hash,
}

struct OAEPOptions {
    hash: crypto.Hash,
    label: []u8,
}
```

#### Required Functions
```vex
// Key generation
fn generate_key(random: io.Reader, bits: int): Result<PrivateKey, Error>
fn generate_multi_prime_key(random: io.Reader, nprimes: int, bits: int): Result<PrivateKey, Error>

// Encryption
fn encrypt_oaep(hash: crypto.Hash, random: io.Reader, pub: &PublicKey, msg: []u8, label: []u8): Result<[]u8, Error>
fn decrypt_oaep(hash: crypto.Hash, random: io.Reader, priv: &PrivateKey, ciphertext: []u8, label: []u8): Result<[]u8, Error>

// Signing
fn sign_pkcs1v15(random: io.Reader, priv: &PrivateKey, hash: crypto.Hash, hashed: []u8): Result<[]u8, Error>
fn verify_pkcs1v15(pub: &PublicKey, hash: crypto.Hash, hashed: []u8, sig: []u8): Result<(), Error>
fn sign_pss(random: io.Reader, priv: &PrivateKey, hash: crypto.Hash, hashed: []u8, opts: *PSSOptions): Result<[]u8, Error>
fn verify_pss(pub: &PublicKey, hash: crypto.Hash, hashed: []u8, sig: []u8, opts: *PSSOptions): Result<(), Error>
```

#### Dependencies
- crypto
- big
- io

### 10.4 crypto/ecdsa
**Status:** ❌ Missing (elliptic curve crypto)
**Description:** ECDSA signatures

#### Required Types
```vex
struct PublicKey {
    curve: elliptic.Curve,
    x: *big.Int,
    y: *big.Int,
}

struct PrivateKey {
    public_key: PublicKey,
    d: *big.Int,
}
```

#### Required Functions
```vex
fn generate_key(c: elliptic.Curve, rand: io.Reader): Result<PrivateKey, Error>
fn sign(rand: io.Reader, priv: &PrivateKey, hash: []u8): Result<[]u8, Error>
fn verify(pub: &PublicKey, hash: []u8, r: *big.Int, s: *big.Int): bool
```

#### Dependencies
- crypto
- elliptic
- big
- io

### 10.5 crypto/aes
**Status:** ❌ Missing (symmetric crypto)
**Description:** AES encryption

#### Required Types
```vex
type KeySizeError = usize
type KeySize = usize
```

#### Required Functions
```vex
fn new_cipher(key: []u8): Result<cipher.Block, Error>
fn new_cipher_with_iv(b: cipher.Block, iv: []u8): cipher.Stream
```

#### Dependencies
- crypto
- cipher

### 10.6 crypto/tls
**Status:** ❌ Missing (critical for HTTPS)
**Description:** TLS protocol implementation

#### Required Types
```vex
struct Config {
    rand: io.Reader,
    time: fn(): time.Time,
    certificates: []Certificate,
    name_to_certificate: Map<str, *Certificate>,
    root_cas: *x509.CertPool,
    next_protos: []str,
    server_name: str,
    client_auth: ClientAuthType,
    client_cas: *x509.CertPool,
    insecure_skip_verify: bool,
    cipher_suites: []uint16,
    prefer_server_cipher_suites: bool,
    session_tickets_disabled: bool,
    session_ticket_key: [32]u8,
    client_session_cache: ClientSessionCache,
    min_version: uint16,
    max_version: uint16,
    curve_preferences: []CurveID,
    dynamic_record_sizing_disabled: bool,
    renegotiation: RenegotiationSupport,
    key_log_writer: io.Writer,
}

struct ConnectionState {
    version: uint16,
    handshake_complete: bool,
    did_resume: bool,
    cipher_suite: uint16,
    negotiated_protocol: str,
    negotiated_protocol_is_mutual: bool,
    server_name: str,
    peer_certificates: []*x509.Certificate,
    verified_chains: [][]*x509.Certificate,
    signed_certificate_timestamps: [][]u8,
    ocsp_response: []u8,
    tls_unique: []u8,
}
```

#### Required Functions
```vex
// Client connections
fn client(net_conn: net.Conn, config: *Config): Result<*Conn, Error>
fn dial(network: str, addr: str, config: *Config): Result<*Conn, Error>
fn dial_with_dialer(dialer: *net.Dialer, network: str, addr: str, config: *Config): Result<*Conn, Error>

// Server connections
fn server(net_conn: net.Conn, config: *Config): Result<*Conn, Error>
fn listen(network: str, laddr: str, config: *Config): Result<net.Listener, Error>
fn new_listener(inner: net.Listener, config: *Config): net.Listener

// Certificate loading
fn load_x509_key_pair(cert_file: str, key_file: str): Result<Certificate, Error>
fn x509_key_pair(cert_pem_block: []u8, key_pem_block: []u8): Result<Certificate, Error>
```

#### Dependencies
- crypto
- net
- x509
- io
- time

### 10.7 crypto/x509
**Status:** ❌ Missing (certificate handling)
**Description:** X.509 certificate parsing

#### Required Types
```vex
struct Certificate {
    raw: []u8,
    raw_tbscertificate: []u8,
    raw_subject_public_key_info: []u8,
    raw_subject: []u8,
    raw_issuer: []u8,
    signature: []u8,
    signature_algorithm: SignatureAlgorithm,
    public_key_algorithm: PublicKeyAlgorithm,
    public_key: any,
    version: int,
    serial_number: *big.Int,
    issuer: pkix.Name,
    subject: pkix.Name,
    not_before: time.Time,
    not_after: time.Time,
    key_usage: KeyUsage,
    extensions: []pkix.Extension,
    extra_extensions: []pkix.Extension,
    unrecognized_extensions: []pkix.Extension,
    subject_key_id: []u8,
    authority_key_id: []u8,
    ocsp_server: []str,
    issuing_certificate_url: []str,
    dns_names: []str,
    email_addresses: []str,
    ip_addresses: []net.IP,
    uris: []*url.URL,
    permitted_dns_domains: []str,
    excluded_dns_domains: []str,
    permitted_ip_ranges: []*net.IPNet,
    excluded_ip_ranges: []*net.IPNet,
    permitted_email_addresses: []str,
    excluded_email_addresses: []str,
    permitted_uri_domains: []str,
    excluded_uri_domains: []str,
    crl_distribution_points: []str,
    policy_identifiers: []asn1.ObjectIdentifier,
}
```

#### Required Functions
```vex
fn parse_certificate(asn1_data: []u8): Result<*Certificate, Error>
fn parse_certificates(asn1_data: []u8): Result<[]*Certificate, Error>
fn marshal_pkix_public_key(pub: any): Result<[]u8, Error>
fn parse_pkix_public_key(der_bytes: []u8): Result<any, Error>
fn create_certificate(rand: io.Reader, template: *Certificate, parent: *Certificate, pub: any, priv: any): Result<[]u8, Error>
```

#### Dependencies
- crypto
- big
- net
- time
- asn1

## 🎯 Implementation Priority

1. **crypto extensions** - Complete hash functions
2. **crypto/rand** - Cryptographic random numbers
3. **crypto/rsa** - RSA encryption/signatures
4. **crypto/aes** - Symmetric encryption
5. **crypto/ecdsa** - Elliptic curve signatures
6. **crypto/x509** - Certificate parsing
7. **crypto/tls** - TLS protocol

## ⚠️ Language Feature Issues

- **Big Integers:** RSA/ECDSA need arbitrary precision arithmetic
- **Complex Types:** Certificate structures are very complex
- **FFI Integration:** Extensive C library bindings needed

## 📋 Missing Critical Dependencies

- **ASN.1 Support:** For certificate parsing
- **Big Integer Math:** For cryptographic operations
- **Elliptic Curves:** Built-in curve implementations

## 🚀 Next Steps

1. Extend crypto with additional hash functions
2. Implement crypto/rand
3. Add RSA support
4. Implement AES encryption
5. Add ECDSA signatures
6. Create X.509 certificate handling
7. Implement TLS protocol