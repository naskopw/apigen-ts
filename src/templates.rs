use crate::types;

use super::{types::Enum, Error, Result};
use handlebars::Handlebars;

const ENUM_TEMPLATE: &str = r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
{{#if description}}
/// {{description}}
{{/if}}
pub enum {{name}} {
    {{#each variants}}
    {{name}}{{#if value}} = "{{value
    }}"{{/if}},
    {{/each}}
}
"#;

const STRUCT_TEMPLATE: &str = r#"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
{{#if description}}
/// {{description}}
{{/if}}
pub struct {{name}} {
    {{#each fields}}
    {{#if description}}
    /// {{description}}
    {{/if}}
    pub {{name}}: {{#if is_array}}Vec<{{/if}}{{#unless required}}Option<{{/unless}}{{type_.Value}}{{type_.Ref}}{{#unless required}}>{{/unless}}{{#if is_array}}>{{/if}},
    {{/each}}
}
"#;

pub struct Templates<'a> {
    handlebars: Handlebars<'a>,
}

impl<'a> Templates<'a> {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string("enum", ENUM_TEMPLATE)
            .expect("Failed to register template");

        handlebars
            .register_template_string("struct", STRUCT_TEMPLATE)
            .expect("Failed to register template");

        Templates { handlebars }
    }

    pub fn render_enum_template(&self, e: &Enum) -> Result<String> {
        self.handlebars
            .render("enum", e)
            .map_err(|e: handlebars::RenderError| Error::Codegen(e.to_string()))
    }

    pub fn render_struct_template(&self, s: &types::Struct) -> Result<String> {
        self.handlebars
            .render("struct", s)
            .map_err(|e: handlebars::RenderError| Error::Codegen(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::EnumVariant;

    use super::*;

    #[test]
    fn test_render_enum_template_with_values() {
        let e = Enum {
            name: "Color".to_string(),
            description: Some("Description".to_string()),
            variants: vec![
                EnumVariant {
                    name: "Red".to_string(),
                    value: Some("1".to_string()),
                },
                EnumVariant {
                    name: "Green".to_string(),
                    value: Some("2".to_string()),
                },
            ],
        };

        let expected = r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Description
pub enum Color {
    Red = "1",
    Green = "2",
}
"#
        .to_string();

        let t = Templates::new();
        let actual = t.render_enum_template(&e).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_enum_template_without_values() {
        let e = Enum {
            name: "Color".to_string(),
            description: None,
            variants: vec![
                EnumVariant {
                    name: "Red".to_string(),
                    value: None,
                },
                EnumVariant {
                    name: "Green".to_string(),
                    value: None,
                },
            ],
        };

        let expected = r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    Red,
    Green,
}
"#
        .to_string();

        let t = Templates::new();
        let actual = t.render_enum_template(&e).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_struct_template_simple() {
        let expected = r#"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Description
pub struct Point {
    pub x: i32,
    pub y: i32,
}
"#
        .to_string();

        let s = types::Struct {
            name: "Point".to_string(),
            description: Some("Description".to_string()),
            fields: vec![
                types::StructField {
                    name: "x".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: true,
                    is_array: false,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: true,
                    is_array: false,
                },
            ],
        };

        let t = Templates::new();
        let actual = t.render_struct_template(&s).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_struct_template_optional() {
        let expected = r#"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Description
pub struct Point {
    pub x: Option<i32>,
    pub y: Option<i32>,
}
"#
        .to_string();

        let s = types::Struct {
            name: "Point".to_string(),
            description: Some("Description".to_string()),
            fields: vec![
                types::StructField {
                    name: "x".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: false,
                    is_array: false,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: false,
                    is_array: false,
                },
            ],
        };

        let t = Templates::new();
        let actual = t.render_struct_template(&s).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_struct_template_array() {
        let expected = r#"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}
"#
        .to_string();

        let s = types::Struct {
            name: "Point".to_string(),
            description: None,
            fields: vec![
                types::StructField {
                    name: "x".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: true,
                    is_array: true,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: true,
                    is_array: true,
                },
            ],
        };

        let t = Templates::new();
        let actual = t.render_struct_template(&s).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_struct_template_array_optional() {
        let expected = r#"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: Vec<Option<i32>>,
    pub y: Vec<Option<i32>>,
}
"#
        .to_string();

        let s = types::Struct {
            name: "Point".to_string(),
            description: None,
            fields: vec![
                types::StructField {
                    name: "x".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: false,
                    is_array: true,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("i32".to_string()),
                    required: false,
                    is_array: true,
                },
            ],
        };

        let t = Templates::new();
        let actual = t.render_struct_template(&s).unwrap();
        assert_eq!(actual, expected);
    }
}
