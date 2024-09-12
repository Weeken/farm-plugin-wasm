pub const CONVERT_BASE64: &str = r#"
const BASE64_MARKER = ';base64,';

function convertDataURIToBinary(dataURI) {
  const base64Index = dataURI.indexOf(BASE64_MARKER) + BASE64_MARKER.length;
  const base64 = dataURI.substring(base64Index);
  const raw = window.atob(base64);
  const rawLength = raw.length;
  const array = new Uint8Array(new ArrayBuffer(rawLength));

  for(i = 0; i < rawLength; i++) {
    array[i] = raw.charCodeAt(i);
  }
  return array;
};
const importObject = {};
const wasm = convertDataURIToBinary('_BYTES_');
const mod = new WebAssembly.Module(wasm);
const instance = new WebAssembly.Instance(mod, importObject);
Object.assign(exports, instance.exports);
"#;
