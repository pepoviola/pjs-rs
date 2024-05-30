import * as location from "ext:deno_web/12_location.js";
globalThis.location = location.locationDescriptor;
import * as webSocket from "ext:deno_websocket/01_websocket.js";
globalThis.WebSocket = webSocket.WebSocket;
import * as crypto from "ext:deno_crypto/00_crypto.js";
globalThis.crypto = crypto.crypto;
globalThis.Crypto = crypto.Crypto;