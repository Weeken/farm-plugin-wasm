#![deny(clippy::all)]
#![feature(path_file_prefix)]

use std::path::Path;

use farmfe_core::{
  config::Config,
  context::{CompilationContext, EmitFileParams},
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult},
  resource::ResourceType,
  serde_json::from_str,
};

use base64::{engine::general_purpose, Engine as _};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  fs::{read_file_raw, transform_output_filename},
  hash::sha256,
};

mod async_code;
use crate::async_code::{FALLBACK_CODE, STREAMING_CODE};
mod sync_code;
use crate::sync_code::CONVERT_BASE64;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Options {
  isolate: Option<bool>,
}

#[farm_plugin]
pub struct FarmPluginWasm {
  options: Options,
}

pub const PARSE_QUERY_TRUE: &str = "";

impl FarmPluginWasm {
  fn new(_: &Config, _options: String) -> Self {
    let options: Options = from_str(&_options).unwrap();
    Self { options }
  }

  fn stringify_query(query: &Vec<(String, String)>) -> String {
    if query.is_empty() {
      return String::new();
    }

    let mut qs = vec![];

    for (k, v) in query {
      if v == PARSE_QUERY_TRUE || v.is_empty() {
        qs.push(k.to_string());
      } else {
        qs.push(format!("{}={}", k, v));
      }
    }

    format!("?{}", qs.join("&"))
  }

  fn get_resource_name(name: &str, module_id: &str) -> String {
    let last_dot = name.rfind('.').unwrap_or(0);
    if last_dot == 0 {
      format!("{}-{}", name, sha256(module_id.as_bytes(), 6))
    } else {
      format!(
        "{}-{}{}",
        &name[..last_dot],
        sha256(module_id.to_string().as_bytes(), 6),
        &name[last_dot..]
      )
    }
  }
}

impl Plugin for FarmPluginWasm {
  fn name(&self) -> &str {
    "FarmPluginWasm"
  }

  fn load(
    &self,
    _param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if _param.resolved_path.ends_with(".wasm") {
      return Ok(Some(PluginLoadHookResult {
        content: String::new(),
        module_type: ModuleType::from("wasm"),
        source_map: None,
      }));
    }
    Ok(None)
  }

  fn transform(
    &self,
    _param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if matches!(_param.module_type, ModuleType::Custom(ref suffix) if suffix == "wasm") {
      let options = self.options.clone();

      let isolate = options.isolate.unwrap_or(false);

      let bytes = read_file_raw(_param.resolved_path).unwrap_or_default();

      if isolate {
        let ext = Path::new(_param.resolved_path)
          .extension()
          .and_then(|s| s.to_str())
          .unwrap();

        let filename = Path::new(_param.resolved_path)
          .file_prefix()
          .and_then(|s| s.to_str())
          .unwrap();
        let resource_name = transform_output_filename(
          _context.config.output.assets_filename.clone(),
          filename,
          &bytes,
          ext,
        ) + Self::stringify_query(&_param.query).as_str();
        let resource_name = Self::get_resource_name(&resource_name, &_param.module_id);

        let wasm_path = if !_context.config.output.public_path.is_empty() {
          let normalized_public_path = _context.config.output.public_path.trim_end_matches("/");

          format!("{}/{}", normalized_public_path, resource_name)
        } else {
          format!("/{}", resource_name)
        };

        let req = r#"fetch('_WASM_PATH_')"#.replace("_WASM_PATH_", &wasm_path);

        let code = format!(
          r#"
            const req = {req};
            const fallback = function() {{
              return req{FALLBACK_CODE}
            }}
            {STREAMING_CODE}
            export default await req;
          "#
        );
        _context.emit_file(EmitFileParams {
          resolved_path: _param.module_id.clone(),
          name: resource_name,
          content: bytes,
          resource_type: ResourceType::Asset(ext.to_string()),
        });
        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content: code,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      } else {
        let base64 = general_purpose::STANDARD.encode(bytes);

        let base64_content = format!(";base64,{}", base64);

        return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
          content: CONVERT_BASE64.replace("_BYTES_", &base64_content),
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: false,
        }));
      }
    }
    Ok(None)
  }
}
