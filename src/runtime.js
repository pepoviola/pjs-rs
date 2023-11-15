
import * as url from "ext:deno_url/00_url.js";
import * as urlPattern from "ext:deno_url/01_urlpattern.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as location from "ext:deno_web/12_location.js";

// needed by the runtime
import * as webidl from "ext:deno_webidl/00_webidl.js";
import * as console2 from "ext:deno_console/01_console.js";
import * as infra from "ext:deno_web/00_infra.js";
import * as dom_ex from "ext:deno_web/01_dom_exception.js";
import * as mimesniff from "ext:deno_web/01_mimesniff.js";
import * as event from "ext:deno_web/02_event.js";
import * as sc from "ext:deno_web/02_structured_clone.js";
import * as ab from "ext:deno_web/03_abort_signal.js";
import * as gi from "ext:deno_web/04_global_interfaces.js";
import * as b64 from "ext:deno_web/05_base64.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as te from "ext:deno_web/08_text_encoding.js";
import * as file from "ext:deno_web/09_file.js";
import * as filereader from "ext:deno_web/10_filereader.js";
import * as mp from "ext:deno_web/13_message_port.js";
import * as cpp from "ext:deno_web/14_compression.js";
import * as perf from "ext:deno_web/15_performance.js";
import * as webSocket from "ext:deno_websocket/01_websocket.js";
import * as ws2 from "ext:deno_websocket/02_websocketstream.js";
import * as f20 from "ext:deno_fetch/20_headers.js";
import * as f21 from "ext:deno_fetch/21_formdata.js";
import * as f22 from "ext:deno_fetch/22_body.js";
import * as f223 from "ext:deno_fetch/22_http_client.js";
import * as f23 from "ext:deno_fetch/23_request.js";
import * as f233 from "ext:deno_fetch/23_response.js";
import * as f26 from "ext:deno_fetch/26_fetch.js";
import * as f27 from "ext:deno_fetch/27_eventsource.js";

// utils
import {nonEnumerable, getterOnly, writable, readOnly} from "ext:pjs_extension/src/js/06_util.js";

// core
const { core } = Deno;
const { ops } = core;

const {
	Error,
	ObjectDefineProperty,
	ObjectDefineProperties,
	ObjectSetPrototypeOf,
	ObjectFreeze,
	StringPrototypeSplit,
} = globalThis.__bootstrap.primordials;

// custom console methods (only overrides log/errors)
function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ");
}

const console = {
  log: (...args) => {
    core.print(`[out]: ${argsToMessage(...args)}\n`, false);
  },
  error: (...args) => {
    core.print(`[err]: ${args}\n`, true);
    core.print(`[err]: ${argsToMessage(...args)}\n`, true);
  },
};

globalThis.console.log = console.log;
globalThis.console.error = console.error;



// Fix timeout (NOTE: make an isolated case to deno_core)
globalThis.setTimeout = (callback, delay) => {
  core.opAsync("op_set_timeout", delay).then(callback);
};


const globalScope = {
  // URL
  URL: nonEnumerable(url.URL),
  URLPatter: nonEnumerable(urlPattern.URLPattern),
  location: location.locationDescriptor,

  // timers
  clearInterval: writable(timers.clearInterval),
  clearTimeout: writable(timers.clearTimeout),
  setInterval: writable(timers.setInterval),
//  setTimeout: writable(timers.setTimeout),

	// crypto
	CryptoKey: nonEnumerable(crypto.CryptoKey),
	crypto: readOnly(crypto.crypto),
	Crypto: nonEnumerable(crypto.Crypto),
	SubtleCrypto: nonEnumerable(crypto.SubtleCrypto),
}
ObjectDefineProperties(globalThis, globalScope);


// include the pjs bundle and expose under `pjs` object.
import * as _ from "ext:pjs_extension/src/js/pjs_bundle.js";
globalThis.pjs = {
  util: polkadotUtil,
  utilCrypto: polkadotUtilCrypto,
  keyring: polkadotKeyring,
  types: polkadotTypes,
  api: polkadotApi,
}

// Delete bootstrap
delete globalThis.__bootstrap;
