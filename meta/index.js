const Wasm = require('../wasm-node/totp.js').Wasm;
const t = new Wasm("JBSWY3DPEHPK3PXP");
console.log(t.generate(Math.floor(new Date().getTime() / 1000)))
