pub const FALLBACK_CODE: &str = r#"
.then(function(x) { return x.arrayBuffer();})
.then(function(bytes) { return WebAssembly.instantiate(bytes, {});})
.then(function(res) { return Object.assign(exports, res.instance.exports);});
"#;

pub const STREAMING_CODE: &str = r#"
return req.then(function(res) {
  if (typeof WebAssembly.instantiateStreaming === "function") {
    return WebAssembly.instantiateStreaming(res, {})
      .then(
        function(res) { return Object.assign(exports, res.instance.exports);},
        function(e) {
          if(res.headers.get("Content-Type") !== "application/wasm") {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
            return fallback();
          }
          throw e;
        }
      );
  }
  return fallback();
});
"#;
