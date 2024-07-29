use crate::types;

use super::{types::Enum, Error, Result};
use handlebars::Handlebars;

const ENUM_TEMPLATE: &str = r#"
{{#if description}}
/** 
 * {{description}}
 */
{{/if}}
export enum {{name}} {
    {{#each variants}}
    {{name}}{{#if value}} = "{{value
    }}"{{/if}},
    {{/each}}
}
"#;

const STRUCT_TEMPLATE: &str = r#"
{{#if description}}
/** 
 * {{description}}
 */
{{/if}}
export interface {{name}} {
    {{#each fields}}
    {{#if description}}
    /// {{description}}
    {{/if}}
    {{#if is_array}}
    {{name}}: Array<{{type_.Value}}{{type_.Ref}}{{#unless required}} | undefined{{/unless}}>;
    {{else}}
    {{name}}{{#unless required}}?{{/unless}}: {{type_.Value}}{{type_.Ref}}{{#if is_array}}[]{{/if}};
    {{/if}}
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
/** 
 * Description
 */
export enum Color {
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
export enum Color {
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
/** 
 * Description
 */
export interface Point {
    x: number;
    y: number;
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
                    type_: types::StructFieldType::Value("number".to_string()),
                    required: true,
                    is_array: false,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("number".to_string()),
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
export interface Point {
    x?: number;
    y?: number;
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
                    type_: types::StructFieldType::Value("number".to_string()),
                    required: false,
                    is_array: false,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("number".to_string()),
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
export interface Point {
    x: Array<number>;
    y: Array<number>;
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
                    type_: types::StructFieldType::Value("number".to_string()),
                    required: true,
                    is_array: true,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("number".to_string()),
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
export interface Point {
    x: Array<number | undefined>;
    y: Array<number | undefined>;
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
                    type_: types::StructFieldType::Value("number".to_string()),
                    required: false,
                    is_array: true,
                },
                types::StructField {
                    name: "y".to_string(),
                    description: None,
                    type_: types::StructFieldType::Value("number".to_string()),
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
