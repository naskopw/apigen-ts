mod templates;
use apigen_plugin_utils::{codegen::Codegen, error::Error, input, oas3_utils, types, Result};
use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use serde_json::Number;

struct CodegenImpl<'a> {
    templates: templates::Templates<'a>,
}

impl<'a> CodegenImpl<'a> {
    pub fn new() -> Self {
        CodegenImpl {
            templates: templates::Templates::new(),
        }
    }
}

impl<'a> Codegen for CodegenImpl<'a> {
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
        name.to_case(Case::Camel)
    }

    fn map_oas3_to_output_type(
        &mut self,
        oas3_type: oas3::spec::SchemaType,
        _: Option<&str>,
        _: &Option<Number>,
    ) -> Result<String> {
        let default_number = Ok("number".to_string());

        match oas3_type {
            SchemaType::String => Ok("String".to_string()),
            SchemaType::Number => default_number,
            SchemaType::Integer => default_number,
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
    CodegenImpl::new().generate(&spec)?;
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
    fn test_map_oas3_to_output_type_number() {
        let schema_type = SchemaType::Integer;
        let format = None;
        let min_value = None;
        assert_eq!(
            CodegenImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "number"
        );
    }

    #[test]
    fn map_oas3_to_output_type_string_test() {
        let schema_type = SchemaType::String;
        let format = None;
        let min_value = None;
        assert_eq!(
            CodegenImpl::new()
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
            CodegenImpl::new()
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
            CodegenImpl::new()
                .map_oas3_to_output_type(schema_type, format, &min_value)
                .unwrap(),
            "Array"
        );
    }

    #[test]
    fn str_to_enum_variant_test() {
        assert_eq!(CodegenImpl::new().str_to_enum_variant("test"), "Test");
        assert_eq!(CodegenImpl::new().str_to_enum_variant("TEST"), "Test");
        assert_eq!(
            CodegenImpl::new().str_to_enum_variant("Test-Test"),
            "TestTest"
        );
        assert_eq!(
            CodegenImpl::new().str_to_enum_variant("test test"),
            "TestTest"
        );
        assert_eq!(CodegenImpl::new().str_to_enum_variant("1test"), "_1_Test");
    }

    #[test]
    fn str_to_variable_name_test() {
        assert_eq!(CodegenImpl::new().str_to_variable_name("test"), "test");
        assert_eq!(
            CodegenImpl::new().str_to_variable_name("test-test"),
            "testTest"
        );
        assert_eq!(
            CodegenImpl::new().str_to_variable_name("test test"),
            "testTest"
        );
    }
}
