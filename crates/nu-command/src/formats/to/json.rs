use nu_protocol::ast::{Call, PathMember};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Value,
};

#[derive(Clone)]
pub struct ToJson;

impl Command for ToJson {
    fn name(&self) -> &str {
        "to json"
    }

    fn signature(&self) -> Signature {
        Signature::build("to json").category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Converts table data into JSON text."
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, ShellError> {
        to_json(call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description:
                "Outputs an unformatted JSON string representing the contents of this table",
            example: "[1 2 3] | to json",
            result: Some(Value::test_string("[\n  1,\n  2,\n  3\n]")),
        }]
    }
}

pub fn value_to_json_value(v: &Value) -> Result<nu_json::Value, ShellError> {
    Ok(match v {
        Value::Bool { val, .. } => nu_json::Value::Bool(*val),
        Value::Filesize { val, .. } => nu_json::Value::I64(*val),
        Value::Duration { val, .. } => nu_json::Value::I64(*val),
        Value::Date { val, .. } => nu_json::Value::String(val.to_string()),
        Value::Float { val, .. } => nu_json::Value::F64(*val),
        Value::Int { val, .. } => nu_json::Value::I64(*val),
        Value::Nothing { .. } => nu_json::Value::Null,
        Value::String { val, .. } => nu_json::Value::String(val.to_string()),
        Value::CellPath { val, .. } => nu_json::Value::Array(
            val.members
                .iter()
                .map(|x| match &x {
                    PathMember::String { val, .. } => Ok(nu_json::Value::String(val.clone())),
                    PathMember::Int { val, .. } => Ok(nu_json::Value::U64(*val as u64)),
                })
                .collect::<Result<Vec<nu_json::Value>, ShellError>>()?,
        ),

        Value::List { vals, .. } => nu_json::Value::Array(json_list(vals)?),
        Value::Error { error } => return Err(error.clone()),
        Value::Block { .. } | Value::Range { .. } => nu_json::Value::Null,
        Value::Binary { val, .. } => {
            nu_json::Value::Array(val.iter().map(|x| nu_json::Value::U64(*x as u64)).collect())
        }
        Value::Record { cols, vals, .. } => {
            let mut m = nu_json::Map::new();
            for (k, v) in cols.iter().zip(vals) {
                m.insert(k.clone(), value_to_json_value(v)?);
            }
            nu_json::Value::Object(m)
        }
        Value::CustomValue { val, .. } => val.to_json(),
    })
}

fn json_list(input: &[Value]) -> Result<Vec<nu_json::Value>, ShellError> {
    let mut out = vec![];

    for value in input {
        out.push(value_to_json_value(value)?);
    }

    Ok(out)
}

fn to_json(call: &Call, input: PipelineData) -> Result<PipelineData, ShellError> {
    let span = call.head;

    let value = input.into_value(span);

    let json_value = value_to_json_value(&value)?;
    match nu_json::to_string(&json_value) {
        Ok(serde_json_string) => Ok(Value::String {
            val: serde_json_string,
            span,
        }
        .into_pipeline_data()),
        _ => Ok(Value::Error {
            error: ShellError::CantConvert("JSON".into(), value.get_type().to_string(), span),
        }
        .into_pipeline_data()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(ToJson {})
    }
}
