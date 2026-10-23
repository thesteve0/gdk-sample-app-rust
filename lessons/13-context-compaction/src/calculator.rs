// Lesson 13
use goose_providers::conversation::message::ToolResult;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ErrorData, JsonObject, Tool,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
pub(super) const TOOL_NAME: &str = "maximum_planned_loss";
pub(super) fn execute_allowed_tool(
    tool_call: &ToolResult<CallToolRequestParams>,
) -> ToolResult<CallToolResult> {
    // Parse boundary: model output can already carry a protocol error.
    let parsed_call = match tool_call {
        Ok(call) => call,
        Err(error) => {
            let failure = format!("the tool call could not be parsed: {}", error.message);
            return Err(ErrorData::invalid_params(failure, None));
        }
    };

    // Capability boundary: only the advertised calculator may execute.
    let requested_name = parsed_call.name.as_ref();
    if requested_name != TOOL_NAME {
        let failure = format!(
            "tool '{}' is not on this application's allowlist; '{}' is the only executable tool",
            requested_name, TOOL_NAME
        );
        return Err(ErrorData::invalid_params(failure, None));
    }

    let arguments = match &parsed_call.arguments {
        Some(arguments) => arguments,
        None => {
            return Err(ErrorData::invalid_params(
                "the tool call carried no arguments".to_string(),
                None,
            ));
        }
    };

    match execute_maximum_planned_loss(arguments) {
        Ok(loss) => Ok(CallToolResult::success(vec![ContentBlock::text(loss)])),
        Err(failure) => Err(ErrorData::invalid_params(failure, None)),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LossArguments {
    entry_price: String,
    stop_price: String,
    share_count: u32,
}

fn execute_maximum_planned_loss(arguments: &JsonObject) -> Result<String, String> {
    // Shape boundary: reject missing, unknown, and wrongly typed fields.
    let deserialized = serde_json::from_value(Value::Object(arguments.clone()));
    let args: LossArguments = match deserialized {
        Ok(parsed) => parsed,
        Err(error) => {
            let failure = format!("arguments do not match the advertised schema: {}", error);
            return Err(failure);
        }
    };

    let entry = parse_price("entry_price", &args.entry_price)?;
    let stop = parse_price("stop_price", &args.stop_price)?;
    if args.share_count == 0 {
        return Err("share_count must be a positive whole number".to_string());
    }
    if entry <= stop {
        return Err(format!(
            "entry_price {} must be above stop_price {} for a long position",
            entry, stop
        ));
    }

    let shares = Decimal::from(args.share_count);
    let per_share = match entry.checked_sub(stop) {
        Some(value) => value,
        None => return Err("price difference exceeds exact decimal range".to_string()),
    };
    let loss = match per_share.checked_mul(shares) {
        Some(value) => value,
        None => return Err("planned loss exceeds exact decimal range".to_string()),
    };
    Ok(format!("${:.2}", loss))
}

fn parse_price(field: &str, raw: &str) -> Result<Decimal, String> {
    // Domain boundary: plain positive decimal strings, never binary floating point.
    let mut parts = raw.split('.');
    let whole = match parts.next() {
        Some(whole) => whole,
        None => return Err(format!("{} is not a plain decimal number", field)),
    };
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{} '{}' is not a plain decimal number", field, raw));
    }
    if let Some(fraction) = parts.next() {
        if fraction.is_empty()
            || fraction.len() > 4
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(format!("{} requires one to four decimal places", field));
        }
    }
    if parts.next().is_some() {
        return Err(format!("{} '{}' is not a plain decimal number", field, raw));
    }
    let parse_result = Decimal::from_str_exact(raw);
    let value = match parse_result {
        Ok(parsed_price) => parsed_price,
        Err(_parse_error) => {
            let failure = format!("{} '{}' is not a plain decimal number", field, raw);
            return Err(failure);
        }
    };
    if value.scale() > 4 {
        return Err(format!(
            "{} '{}' has more than four decimal places",
            field, raw
        ));
    }
    if value <= Decimal::ZERO {
        return Err(format!("{} must be a positive dollar amount", field));
    }
    Ok(value)
}

pub(super) fn tool_definition() -> Tool {
    let schema: JsonObject = serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "entry_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "stop_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "share_count": {"type": "integer", "minimum": 1}
        },
        "required": ["entry_price", "stop_price", "share_count"],
        "additionalProperties": false
    }))
    .expect("static input schema must be a JSON object");

    Tool::new(
        TOOL_NAME,
        "Calculate the maximum planned loss for a hypothetical long stock position from \
         decimal-dollar entry and stop prices and a whole-number share count. This excludes \
         fees, slippage, and a gap through the stop.",
        Arc::new(schema),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::arguments;
    #[test]
    fn exact_loss_and_tweak_are_validated() -> Result<(), String> {
        let default_loss = execute_maximum_planned_loss(&arguments(200))?;
        let tweaked_loss = execute_maximum_planned_loss(&arguments(100))?;
        assert_eq!(default_loss, "$100.00");
        assert_eq!(tweaked_loss, "$50.00");
        Ok(())
    }

    #[test]
    fn untrusted_shape_and_domain_values_fail_without_panicking() {
        for value in [
            json!({"entry_price":51.20,"stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":200,"ticker":"X"}),
            json!({"entry_price":"51.20","stop_price":"50.70"}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":-1}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":0}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":1.5}),
            json!({"entry_price":"50.70","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"49.00","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20000","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"0","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20","stop_price":"-1","share_count":200}),
            json!({"entry_price":"79228162514264337593543950335","stop_price":"1","share_count":200}),
        ] {
            let args = match value {
                Value::Object(args) => args,
                _ => panic!("test fixture is a JSON object"),
            };
            assert!(execute_maximum_planned_loss(&args).is_err());
        }
        for raw in ["1e2", "+1", " 1", "1_0", ".5", "5.", "1.2.3", "not-money"] {
            assert!(parse_price("price", raw).is_err());
        }
        assert!(execute_allowed_tool(&Ok(CallToolRequestParams::new(TOOL_NAME))).is_err());
        let unknown = CallToolRequestParams::new("place_order").with_arguments(arguments(200));
        assert!(execute_allowed_tool(&Ok(unknown)).is_err());
        assert!(execute_allowed_tool(&Err(ErrorData::invalid_params("broken", None))).is_err());
    }
}
