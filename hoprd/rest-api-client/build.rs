use std::{env, fs, path::PathBuf};

use hoprd_api::ApiDoc;
use progenitor::Generator;
use serde_json::{Value, json};
use utoipa::OpenApi;

fn main() {
    println!("cargo:rerun-if-changed=../rest-api/src");
    println!("cargo:rerun-if-changed=../rest-api/Cargo.toml");

    let openapi_json = ApiDoc::openapi()
        .to_pretty_json()
        .expect("failed to generate OpenAPI JSON");

    let mut spec_value: Value = serde_json::from_str(&openapi_json).expect("failed to parse OpenAPI JSON");
    normalize_openapi_version(&mut spec_value);
    normalize_nullable_types(&mut spec_value);
    normalize_response_content(&mut spec_value);
    let spec: openapiv3::OpenAPI =
        serde_json::from_value(spec_value).expect("failed to parse OpenAPI JSON after normalization");

    let mut generator = Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .expect("failed to generate API client tokens");

    let ast = syn::parse2(tokens).expect("failed to parse generated tokens");
    let content = prettyplease::unparse(&ast);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR missing"));
    let out_file = out_dir.join("codegen.rs");
    fs::write(&out_file, content).expect("failed to write generated client");
}

fn normalize_openapi_version(value: &mut Value) {
    if let Value::Object(map) = value {
        map.insert("openapi".to_string(), Value::String("3.0.3".to_string()));
        map.remove("jsonSchemaDialect");
    }
}

fn normalize_nullable_types(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                normalize_nullable_types(item);
            }
        }
        Value::Object(map) => {
            if let Some(Value::Array(types)) = map.get_mut("type") {
                let mut nullable = false;
                let mut collected = Vec::new();
                for t in types.iter() {
                    if let Value::String(s) = t {
                        if s == "null" {
                            nullable = true;
                        } else {
                            collected.push(s.clone());
                        }
                    }
                }

                match collected.len() {
                    0 => {
                        map.remove("type");
                    }
                    1 => {
                        map.insert("type".to_string(), Value::String(collected[0].clone()));
                    }
                    _ => {
                        let one_of = collected.into_iter().map(|t| json!({ "type": t })).collect::<Vec<_>>();
                        map.remove("type");
                        map.insert("oneOf".to_string(), Value::Array(one_of));
                    }
                }

                if nullable {
                    map.entry("nullable".to_string()).or_insert(Value::Bool(true));
                }
            }

            for v in map.values_mut() {
                normalize_nullable_types(v);
            }
        }
        _ => {}
    }
}

fn normalize_response_content(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                normalize_response_content(item);
            }
        }
        Value::Object(map) => {
            if let Some(Value::Object(responses)) = map.get_mut("responses") {
                let preferred = ["200", "201", "202", "204", "206", "207"];
                let mut keep_key: Option<String> = None;

                for key in preferred {
                    if response_has_content(responses.get(key)) {
                        keep_key = Some(key.to_string());
                        break;
                    }
                }

                if keep_key.is_none() {
                    for (key, value) in responses.iter() {
                        if response_has_content(Some(value)) {
                            keep_key = Some(key.clone());
                            break;
                        }
                    }
                }

                if let Some(keep) = keep_key {
                    if let Some(mut kept_response) = responses.remove(&keep) {
                        if let Value::Object(resp_obj) = &mut kept_response {
                            if let Some(Value::Object(content)) = resp_obj.get_mut("content") {
                                if content.len() > 1 {
                                    if let Some(json_value) = content.remove("application/json") {
                                        content.clear();
                                        content.insert("application/json".to_string(), json_value);
                                    } else if let Some((first_key, first_value)) =
                                        content.iter().next().map(|(k, v)| (k.clone(), v.clone()))
                                    {
                                        content.clear();
                                        content.insert(first_key, first_value);
                                    }
                                }
                            }
                        }
                        responses.clear();
                        responses.insert(keep, kept_response);
                    }
                }
            }

            for v in map.values_mut() {
                normalize_response_content(v);
            }
        }
        _ => {}
    }
}

fn response_has_content(response: Option<&Value>) -> bool {
    match response {
        Some(Value::Object(resp_obj)) => match resp_obj.get("content") {
            Some(Value::Object(content)) => !content.is_empty(),
            _ => false,
        },
        _ => false,
    }
}
