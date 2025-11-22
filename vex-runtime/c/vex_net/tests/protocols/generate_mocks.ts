
import { write } from "bun";

const OUTPUT_DIR = "tests/protocols";
const COUNT = 500;

// --- Helpers ---

function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function randomBytes(len) {
  const buf = Buffer.alloc(len);
  for (let i = 0; i < len; i++) buf[i] = randomInt(0, 255);
  return buf;
}

function randomString(len) {
  const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let res = "";
  for (let i = 0; i < len; i++) res += chars[randomInt(0, chars.length - 1)];
  return res;
}

// --- Generators ---

function generateHttp1() {
  const methods = ["GET", "POST", "PUT", "DELETE", "HEAD"];
  const uris = ["/api/v1/users", "/index.html", "/login", "/static/style.css", "/ws"];
  const headers = [
    "Host: example.com",
    "User-Agent: VexClient/1.0",
    "Accept: */*",
    "Connection: keep-alive",
    "Content-Type: application/json"
  ];

  let output = "";
  for (let i = 0; i < COUNT; i++) {
    const method = methods[randomInt(0, methods.length - 1)];
    const uri = uris[randomInt(0, uris.length - 1)] + "?id=" + randomInt(1, 10000);
    
    let req = `${method} ${uri} HTTP/1.1\r\n`;
    // Add random headers
    const numHeaders = randomInt(2, 5);
    for (let j = 0; j < numHeaders; j++) {
      req += headers[randomInt(0, headers.length - 1)] + "\r\n";
    }
    
    if (method === "POST" || method === "PUT") {
      const body = `{"id":${i},"data":"${randomString(10)}"}`;
      req += `Content-Length: ${body.length}\r\n\r\n${body}`;
    } else {
      req += "\r\n";
    }
    
    // Separator for test runner
    output += `---BEGIN---\n${req}\n---END---\n`;
  }
  return output;
}

function generateHttp2() {
  // Generate Preface + Frame Header
  const preface = Buffer.from("PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
  
  let output = "";
  for (let i = 0; i < COUNT; i++) {
    // Random Frame Header
    const length = randomInt(0, 16384);
    const type = randomInt(0, 9); // DATA, HEADERS, etc.
    const flags = randomInt(0, 255);
    const streamId = randomInt(1, 2147483647);
    
    const frameHeader = Buffer.alloc(9);
    frameHeader.writeUIntBE(length, 0, 3);
    frameHeader.writeUInt8(type, 3);
    frameHeader.writeUInt8(flags, 4);
    frameHeader.writeUInt32BE(streamId, 5);
    
    // Combine preface (sometimes) + frame header
    const data = Buffer.concat([
      i % 2 === 0 ? preface : Buffer.alloc(0),
      frameHeader
    ]);
    
    output += `---BEGIN---\n${data.toString('hex')}\n---END---\n`;
  }
  return output;
}

function generateWebSocket() {
  let output = "";
  for (let i = 0; i < COUNT; i++) {
    const fin = 1;
    const opcode = i % 10 === 0 ? 0x8 : 0x1; // Close or Text
    const masked = 1;
    const payload = randomString(randomInt(5, 100));
    const maskKey = randomBytes(4);
    
    let lenByte = payload.length;
    if (payload.length > 125) lenByte = 126; // Simplified for mock
    
    const buf = Buffer.alloc(2 + (masked ? 4 : 0) + payload.length);
    buf[0] = (fin << 7) | opcode;
    buf[1] = (masked << 7) | lenByte;
    
    let pos = 2;
    if (masked) {
      maskKey.copy(buf, pos);
      pos += 4;
    }
    
    const payloadBuf = Buffer.from(payload);
    if (masked) {
      for (let j = 0; j < payloadBuf.length; j++) {
        payloadBuf[j] ^= maskKey[j % 4];
      }
    }
    payloadBuf.copy(buf, pos);
    
    output += `---BEGIN---\n${buf.toString('hex')}\n---END---\n`;
  }
  return output;
}

function generateDNS() {
  let output = "";
  for (let i = 0; i < COUNT; i++) {
    const id = randomInt(0, 65535);
    const flags = 0x0100; // Standard query
    const qdcount = 1;
    
    const header = Buffer.alloc(12);
    header.writeUInt16BE(id, 0);
    header.writeUInt16BE(flags, 2);
    header.writeUInt16BE(qdcount, 4);
    
    const domain = `test${i}.example.com`;
    const parts = domain.split('.');
    let qnameLen = 0;
    parts.forEach(p => qnameLen += p.length + 1);
    qnameLen += 1; // Null terminator
    
    const qname = Buffer.alloc(qnameLen);
    let pos = 0;
    parts.forEach(p => {
      qname.writeUInt8(p.length, pos++);
      qname.write(p, pos);
      pos += p.length;
    });
    qname.writeUInt8(0, pos);
    
    const qtypeClass = Buffer.alloc(4);
    qtypeClass.writeUInt16BE(1, 0); // A record
    qtypeClass.writeUInt16BE(1, 2); // IN class
    
    const packet = Buffer.concat([header, qname, qtypeClass]);
    output += `---BEGIN---\n${packet.toString('hex')}\n---END---\n`;
  }
  return output;
}

function generateTLS() {
  let output = "";
  for (let i = 0; i < COUNT; i++) {
    // Simplified ClientHello construction
    const version = 0x0303; // TLS 1.2
    const random = randomBytes(32);
    const sessionId = randomBytes(32);
    const cipherSuites = Buffer.alloc(2);
    cipherSuites.writeUInt16BE(0x002f, 0); // AES128-SHA
    
    // SNI Extension
    const serverName = `server${i}.com`;
    const sniLen = serverName.length + 5; // + type(1) + len(2) + list_len(2)
    const sniExt = Buffer.alloc(4 + sniLen);
    sniExt.writeUInt16BE(0x0000, 0); // Type SNI
    sniExt.writeUInt16BE(sniLen, 2); // Length
    sniExt.writeUInt16BE(sniLen - 2, 4); // List Length
    sniExt.writeUInt8(0, 6); // Name Type (Host)
    sniExt.writeUInt16BE(serverName.length, 7);
    sniExt.write(serverName, 9);
    
    const extensions = sniExt;
    const extLen = extensions.length;
    
    const handshakeLen = 2 + 32 + 1 + 32 + 2 + 2 + 1 + 1 + 2 + extLen;
    const handshake = Buffer.alloc(handshakeLen);
    let pos = 0;
    handshake.writeUInt8(1, pos++); // ClientHello
    // Length (24-bit) - skipping for simplicity in mock generator, parser handles it
    pos += 3; 
    handshake.writeUInt16BE(version, pos); pos += 2;
    random.copy(handshake, pos); pos += 32;
    handshake.writeUInt8(32, pos++);
    sessionId.copy(handshake, pos); pos += 32;
    handshake.writeUInt16BE(2, pos); pos += 2; // Cipher len
    cipherSuites.copy(handshake, pos); pos += 2;
    handshake.writeUInt8(1, pos++); // Comp len
    handshake.writeUInt8(0, pos++); // Null comp
    handshake.writeUInt16BE(extLen, pos); pos += 2;
    extensions.copy(handshake, pos);
    
    // Record Layer
    const record = Buffer.alloc(5 + handshakeLen);
    record.writeUInt8(22, 0); // Handshake
    record.writeUInt16BE(0x0301, 1); // TLS 1.0 record
    record.writeUInt16BE(handshakeLen, 3);
    handshake.copy(record, 5);
    
    output += `---BEGIN---\n${record.toString('hex')}\n---END---\n`;
  }
  return output;
}

function generateICMP() {
  let output = "";
  for (let i = 0; i < COUNT; i++) {
    const type = 8; // Echo Request
    const code = 0;
    const id = i % 65535;
    const seq = i % 65535;
    const payload = Buffer.from(`PingData${i}`);
    
    const packet = Buffer.alloc(8 + payload.length);
    packet.writeUInt8(type, 0);
    packet.writeUInt8(code, 1);
    packet.writeUInt16BE(0, 2); // Checksum placeholder
    packet.writeUInt16BE(id, 4);
    packet.writeUInt16BE(seq, 6);
    payload.copy(packet, 8);
    
    // Calculate Checksum
    let sum = 0;
    for (let j = 0; j < packet.length; j += 2) {
      let word = (packet[j] << 8) + (j + 1 < packet.length ? packet[j + 1] : 0);
      sum += word;
    }
    while (sum >> 16) sum = (sum & 0xFFFF) + (sum >> 16);
    sum = ~sum & 0xFFFF;
    
    packet.writeUInt16BE(sum, 2);
    
    output += `---BEGIN---\n${packet.toString('hex')}\n---END---\n`;
  }
  return output;
}

// --- Main ---

console.log(`Generating ${COUNT} mocks per protocol...`);

await write(`${OUTPUT_DIR}/http1.mock.txt`, generateHttp1());
await write(`${OUTPUT_DIR}/http2.mock.txt`, generateHttp2());
await write(`${OUTPUT_DIR}/websocket.mock.txt`, generateWebSocket());
await write(`${OUTPUT_DIR}/dns.mock.txt`, generateDNS());
await write(`${OUTPUT_DIR}/tls.mock.txt`, generateTLS());
await write(`${OUTPUT_DIR}/icmp.mock.txt`, generateICMP());

console.log("Done! Mocks saved to", OUTPUT_DIR);
