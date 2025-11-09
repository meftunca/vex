# Vex Stdlib Planning - 11: Encoding and Serialization

**Priority:** 11
**Status:** Partial (encoding exists, others missing)
**Dependencies:** builtin, io, reflect

## 📦 Packages in This Category

### 11.1 encoding/json
**Status:** ❌ Missing (critical for data exchange)
**Description:** JSON encoding and decoding

#### Required Types
```vex
struct Decoder {
    r: io.Reader,
    buf: []u8,
    d: decodeState,
    scanp: usize,
    scanned: usize,
    err: Error,
    token_state: int,
    token_stack: []int,
}

struct Encoder {
    w: io.Writer,
    err: Error,
    escape_html: bool,
    indent_prefix: str,
    indent_value: str,
}

trait Marshaler {
    fn marshal_json(self: &Self): Result<[]u8, Error>
}

trait Unmarshaler {
    fn unmarshal_json(self: &mut Self, data: []u8) -> Result<(), Error>
}

struct InvalidUnmarshalError {
    typ: reflect.Type,
}

struct MarshalerError {
    typ: reflect.Type,
    err: Error,
    source: reflect.Type,
}

struct SyntaxError {
    msg: str,
    offset: i64,
}

struct UnmarshalTypeError {
    value: str,
    typ: reflect.Type,
    offset: i64,
    struct_: str,
    field: str,
}
```

#### Required Functions
```vex
// Encoding
fn marshal(v: any): Result<[]u8, Error>
fn marshal_indent(v: any, prefix: str, indent: str): Result<[]u8, Error>
fn new_encoder(w: io.Writer): *Encoder
fn encode(e: *Encoder, v: any): Result<(), Error>

// Decoding
fn unmarshal(data: []u8, v: any): Result<(), Error>
fn new_decoder(r: io.Reader): *Decoder
fn decode(d: *Decoder, v: any): Result<(), Error>

// Streaming
fn new_decoder(r: io.Reader): *Decoder
fn token(d: *Decoder): Result<Token, Error>
fn more(d: *Decoder): bool
fn input_offset(d: *Decoder): i64

// Utilities
fn valid(data: []u8): bool
fn compact(dst: *bytes.Buffer, src: []u8): Result<(), Error>
fn indent(dst: *bytes.Buffer, src: []u8, prefix: str, indent: str): Result<(), Error>
fn html_escape(dst: *bytes.Buffer, src: []u8)
```

#### Dependencies
- builtin
- io
- reflect
- bytes

### 11.2 encoding/xml
**Status:** ❌ Missing (important for web services)
**Description:** XML encoding and decoding

#### Required Types
```vex
struct Decoder {
    r: io.Reader,
    buf: []u8,
    stuck: bool,
    avail: usize,
    next_byte: int,
    ns: Map<str, str>,
    err: Error,
    strict: bool,
    entity: Map<str, str>,
    auto_close: []str,
    nstags: []TagPath,
    attr: []xml.Attr,
}

struct Encoder {
    p: printer,
}

trait Marshaler {
    fn marshal_xml(self: &Self, e: *Encoder, start: StartElement) -> Result<(), Error>
}

trait Unmarshaler {
    fn unmarshal_xml(self: &mut Self, d: *Decoder, start: StartElement) -> Result<(), Error>
}

struct StartElement {
    name: Name,
    attr: []Attr,
}

struct EndElement {
    name: Name,
}

struct CharData {
    data: []u8,
}

struct Comment {
    data: []u8,
}

struct ProcInst {
    target: str,
    inst: []u8,
}

struct Directive {
    data: []u8,
}

struct Name {
    space: str,
    local: str,
}

struct Attr {
    name: Name,
    value: str,
}
```

#### Required Functions
```vex
// Encoding
fn marshal(v: any): Result<[]u8, Error>
fn marshal_indent(v: any, prefix: str, indent: str): Result<[]u8, Error>
fn new_encoder(w: io.Writer): *Encoder
fn encode(e: *Encoder, v: any): Result<(), Error>
fn encode_element(e: *Encoder, v: any, start: StartElement) -> Result<(), Error>

// Decoding
fn unmarshal(data: []u8, v: any): Result<(), Error>
fn new_decoder(r: io.Reader): *Decoder
fn decode(d: *Decoder, v: any): Result<(), Error>
fn decode_element(d: *Decoder, v: any, start: StartElement) -> Result<(), Error>

// Token-based parsing
fn token(d: *Decoder): Result<Token, Error>
fn skip(d: *Decoder): Result<(), Error>
fn input_pos(d: *Decoder): i64
```

#### Dependencies
- builtin
- io
- reflect
- bytes

### 11.3 encoding/base64
**Status:** ❌ Missing (essential for data encoding)
**Description:** Base64 encoding and decoding

#### Required Types
```vex
type Encoding = *encoding

struct encoding {
    encode: [64]u8,
    decode_map: [256]u8,
    pad_char: u8,
    strict: bool,
}
```

#### Required Functions
```vex
// Standard encodings
static std_encoding: *Encoding
static url_encoding: *Encoding
static raw_std_encoding: *Encoding
static raw_url_encoding: *Encoding

// Encoding operations
fn new_encoding(encoder: str): *Encoding
fn encoded_len(enc: *Encoding, n: usize): usize
fn encode(enc: *Encoding, dst: []u8, src: []u8): usize
fn encode_to_string(enc: *Encoding, src: []u8): str
fn decode(enc: *Encoding, dst: []u8, src: []u8): Result<usize, Error>
fn decode_string(enc: *Encoding, s: str): Result<[]u8, Error>

// Corrupt input error
fn corrupt_input_error(n: usize): Error
```

#### Dependencies
- builtin
- strings

### 11.4 encoding/hex
**Status:** ❌ Missing (useful for binary data)
**Description:** Hexadecimal encoding

#### Required Functions
```vex
fn encode(dst: []u8, src: []u8): usize
fn encode_to_string(src: []u8): str
fn decode(dst: []u8, src: []u8): Result<usize, Error>
fn decode_string(s: str): Result<[]u8, Error>
fn decoded_len(x: usize): usize
fn dump(data: []u8): str
fn dumper(w: io.Writer): io.Writer
```

#### Dependencies
- builtin
- io

### 11.5 encoding/asn1
**Status:** ❌ Missing (needed for crypto certificates)
**Description:** ASN.1 encoding

#### Required Types
```vex
enum Class {
    Universal,
    Application,
    ContextSpecific,
    Private,
}

struct RawValue {
    tag: int,
    class: Class,
    is_compound: bool,
    bytes: []u8,
    full_bytes: []u8,
}

struct ObjectIdentifier {
    data: []int,
}
```

#### Required Functions
```vex
fn marshal(val: any): Result<[]u8, Error>
fn unmarshal(b: []u8, val: any): Result<(), Error>
fn unmarshal_with_params(b: []u8, val: any, params: str): Result<(), Error>
```

#### Dependencies
- builtin
- reflect
- big

### 11.6 encoding/binary
**Status:** ❌ Missing (important for network protocols)
**Description:** Binary encoding (big-endian/little-endian)

#### Required Types
```vex
struct ByteOrder {
    // interface for byte ordering
}

static big_endian: ByteOrder
static little_endian: ByteOrder
```

#### Required Functions
```vex
// Basic operations
fn put_uvarint(buf: []u8, x: u64): usize
fn uvarint(buf: []u8): (u64, usize)
fn put_varint(buf: []u8, x: i64): usize
fn varint(buf: []u8): (i64, usize)

// Byte order operations
fn uint16(b: []u8, order: ByteOrder): u16
fn put_uint16(b: []u8, v: u16, order: ByteOrder)
fn uint32(b: []u8, order: ByteOrder): u32
fn put_uint32(b: []u8, v: u32, order: ByteOrder)
fn uint64(b: []u8, order: ByteOrder): u64
fn put_uint64(b: []u8, v: u64, order: ByteOrder)

// Read/write operations
fn read(r: io.Reader, order: ByteOrder, data: any): Result<(), Error>
fn write(w: io.Writer, order: ByteOrder, data: any): Result<(), Error>
fn size(v: any): usize
```

#### Dependencies
- builtin
- io
- reflect

### 11.7 encoding/gob
**Status:** ❌ Missing (Go-specific serialization)
**Description:** Go binary serialization

#### Required Types
```vex
struct Encoder {
    w: io.Writer,
    sent: Map<reflect.Type, *encOp>,
    count_state: *encoder_state,
    err: Error,
}

struct Decoder {
    r: io.Reader,
    buf: []u8,
    // internal state
}
```

#### Required Functions
```vex
fn new_encoder(w: io.Writer): *Encoder
fn encode(e: *Encoder, value: any): Result<(), Error>
fn new_decoder(r: io.Reader): *Decoder
fn decode(d: *Decoder, value: any): Result<(), Error>
fn register(value: any)
fn register_name(name: str, value: any)
```

#### Dependencies
- builtin
- io
- reflect

## 🎯 Implementation Priority

1. **encoding/json** - JSON encoding/decoding
2. **encoding/base64** - Base64 encoding
3. **encoding/hex** - Hexadecimal encoding
4. **encoding/binary** - Binary serialization
5. **encoding/xml** - XML processing
6. **encoding/asn1** - ASN.1 for crypto
7. **encoding/gob** - Go-specific binary format

## ⚠️ Language Feature Issues

- **Reflection:** All encoding packages need runtime type information
- **Interface Types:** Complex interface hierarchies
- **Any Type:** Generic encoding/decoding requires `any`

## 📋 Missing Critical Dependencies

- **Reflection System:** Runtime type inspection
- **Type Registration:** For serialization formats
- **Streaming Parsers:** For large data handling

## 🚀 Next Steps

1. Implement JSON encoding/decoding
2. Add Base64 and hex encoding
3. Create binary serialization
4. Implement XML support
5. Add ASN.1 for certificates
6. Create Go-specific gob format