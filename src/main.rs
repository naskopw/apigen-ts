mod templates;
use apigen_plugin_utils::{error::Error, input, oas3_utils, parser::Parser, types, Result};
use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use serde_json::Number;

struct ParserImpl<'a> {
    templates: templates::Templates<'a>,
}

impl<'a> ParserImpl<'a> {
    pub fn new() -> Self {
        ParserImpl {
            templates: templates::Templates::new(),
        }
    }
}

impl<'a> Parser for ParserImpl<'a> {
    fn str_to_enum_variant(&mut self, name: &str) -> String {
        let name = name.to_case(Case::UpperCamel);
        if name.chars().next().unwrap_or_default().is_numeric() {
            format!(
                "_{}_{}",
                name.chars().next().expect("No first char"),
                &name[1..]
            )
        } else {
            name.to_string()
        }
    }

    fn str_to_variable_name(&mut self, name: &str) -> String {
        let name = if name.chars().next().unwrap_or_default().is_numeric() {
            format!("_{}", name)
        } else {
            name.to_string()
        };
        name.to_case(Case::Snake)
    }

    fn map_oas3_to_output_type(
        &mut self,
        oas3_type: oas3::spec::SchemaType,
        format: Option<&str>,
        min_value: &Option<Number>,
    ) -> Result<String> {
        let is_unsigned = match min_value {
            Some(min) => min.as_i64().unwrap_or(-1) == 0,
            None => false,
        };
        let default_int = Ok("i32".to_string());
        let default_float = Ok("f64".to_string());

        match oas3_type {
            SchemaType::String => Ok("String".to_string()),
            SchemaType::Number => match format {
                Some(format) => match format {
                    "float" => Ok("f32".to_string()),
                    "double" => Ok("f64".to_string()),
                    _ => default_float,
                },
                None => default_float,
            },
            SchemaType::Integer => match format {
                Some(format) => match format {
                    "int32" => {
                        if is_unsigned {
                            Ok("u32".to_string())
                        } else {
                            Ok("i32".to_string())
                        }
                    }
                    "int64" => {
                        if is_unsigned {
                            Ok("u64".to_string())
                        } else {
                            Ok("i64".to_string())
                        }
                    }
                    _ => default_int,
                },
                None => default_int,
            },
            SchemaType::Boolean => Ok("bool".to_string()),
            SchemaType::Array => Ok("Array".to_string()),
            _ => Err(Error::Codegen(format!("Unsupported type: {:?}", oas3_type))),
        }
    }

    fn parse_struct_or_enum(
        &mut self,
        schema: (&std::string::String, &ObjectOrReference<oas3::Schema>),
    ) -> Result<String> {
        let obj = oas3_utils::ObjectOrReference::object_or_error(schema.1)?;
        if obj.enum_values.is_empty() {
            let s = Self::parse_struct(self, schema.0, schema.1)?;
            self.templates.render_struct_template(&s)
        } else {
            let e = Self::parse_enum(self, schema)?;
            self.templates.render_enum_template(&e)
        }
    }
}

fn run() -> Result<()> {
    let spec = input::read_and_parse()?;
    ParserImpl::new().generate(&spec)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        apigen_plugin_utils::error::log(e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_oas3_to_output_type_f32() {
        let schema_type = SchemaType::Number;
        let format = Some("float");
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "f32"
        );
    }

    #[test]
    fn test_map_oas3_to_output_type_f64() {
        let schema_type = SchemaType::Number;
        let format = Some("double");
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "f64"
        );
    }

    #[test]
    fn test_map_oas3_to_output_type_i32() {
        let schema_type = SchemaType::Integer;
        let format = Some("int32");
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "i32"
        );
    }

    #[test]
    fn test_map_oas3_to_output_type_u32() {
        let schema_type = SchemaType::Integer;
        let format = Some("int32");
        let min_value = Some(Number::from(0));
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "u32"
        );
    }

    #[test]
    fn test_map_oas3_to_output_type_i64() {
        let schema_type = SchemaType::Integer;
        let format = Some("int64");
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "i64"
        );
    }

    #[test]
    fn map_oas3_to_output_type_u64_test() {
        let schema_type = SchemaType::Integer;
        let format = Some("int64");
        let min_value = Some(Number::from(0));
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "u64"
        );
    }

    #[test]
    fn map_oas3_to_output_type_string_test() {
        let schema_type = SchemaType::String;
        let format = None;
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "String"
        );
    }

    #[test]
    fn map_oas3_to_output_type_bool_test() {
        let schema_type = SchemaType::Boolean;
        let format = None;
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "bool"
        );
    }

    #[test]
    fn map_oas3_to_output_type_array_test() {
        let schema_type = SchemaType::Array;
        let format = None;
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "Array"
        );
    }

    #[test]
    fn map_oas3_to_output_type_default_int_test() {
        let schema_type = SchemaType::Integer;
        let format = None;
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "i32"
        );
    }

    #[test]
    fn map_oas3_to_output_type_default_float_test() {
        let schema_type = SchemaType::Number;
        let format = None;
        let min_value = None;
        assert_eq!(
            ParserImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "f64"
        );
    }

    #[test]
    fn str_to_enum_variant_test() {
        assert_eq!(ParserImpl::new().str_to_enum_variant("test"), "Test");
        assert_eq!(ParserImpl::new().str_to_enum_variant("TEST"), "Test");
        assert_eq!(
            ParserImpl::new().str_to_enum_variant("Test-Test"),
            "TestTest"
        );
        assert_eq!(
            ParserImpl::new().str_to_enum_variant("test test"),
            "TestTest"
        );
        assert_eq!(ParserImpl::new().str_to_enum_variant("1test"), "_1_Test");
    }

    #[test]
    fn str_to_variable_name_test() {
        assert_eq!(ParserImpl::new().str_to_variable_name("test"), "test");
        assert_eq!(
            ParserImpl::new().str_to_variable_name("test-test"),
            "test_test"
        );
        assert_eq!(
            ParserImpl::new().str_to_variable_name("test test"),
            "test_test"
        );
    }
}
