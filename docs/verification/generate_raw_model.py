"""Development-only DTO generation from the approved grammar; never a runtime compiler.

--write updates raw.rs; --check verifies reproducibility. Review generated changes.
Numeric/string bounds and cross-property conditions remain linter/compiler duties.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / 'docs/schema-authoring/flow-v2.schema.json'
GRAMMAR = json.loads(PATH.read_text())
NAMES = {key: 'Raw' + ''.join(s.title() for s in key.split('_')) for key in GRAMMAR['$defs']}
NAMES.update(trace='RawRecordSchema', field='RawFieldSpec', rule='RawRule',
             identity_binding='RawIdentitySpec', join='RawJoinSpec', encoding='RawEncodingSpec',
             reference='RawReferenceDataDependency', expr='RawExpr', ref='RawRef')
DEFINITIONS = []
ENUMS = {}
DERIVE = '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n'


def pascal(s):
    operators = {'==': 'Eq', '!=': 'Ne', '<': 'Lt', '<=': 'Le', '>': 'Gt', '>=': 'Ge', '.': 'Dot'}
    return operators.get(s) or ''.join(x.title() for x in re.split(r'[^a-zA-Z0-9]+', s))


def field_name(s):
    return 'r#' + s if s in ('ref', 'type', 'match', 'const', 'in') else s


def string_enum(values, name):
    key = tuple(values)
    if key in ENUMS:
        return ENUMS[key]
    ENUMS[key] = name
    body = '\n'.join(f'    #[serde(rename = {json.dumps(v)})]\n    {pascal(v)},' for v in values)
    DEFINITIONS.append(DERIVE + f'pub enum {name} {{\n{body}\n}}\n')
    return name


def fields(node, name, exclude=()):
    lines = []
    required = node.get('required', [])
    for key, prop in node.get('properties', {}).items():
        if key in exclude:
            continue
        ty = rust_type(prop, name + pascal(key))
        if key == 'schema_format_version':
            ty = 'u32'
        if key not in required:
            lines.append('    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]')
            ty = f'Option<{ty}>'
        elif ty.startswith('Option<'):
            lines.append('    #[serde(deserialize_with = "required_option")]')
        elif ty.startswith('BTreeMap<'):
            lines.append('    #[serde(deserialize_with = "unique_map")]')
        lines.append(f'    pub {field_name(key)}: {ty},')
    if name == 'RawEncodingSpec':
        # Approved Rust-only runtime binding; not an authoring JSON property.
        lines.extend(['    /// Explicit adapter binding, never part of authoring JSON.', '    #[serde(skip)]', '    pub resolved: Option<RawResolvedEncoding>,'])
    return '\n'.join(lines)


def variant_fields(node, name, tag):
    # Enum fields inherit public accessibility, no visibility keyword allowed.
    return fields(node, name, (tag,)).replace('    pub ', '    ')


def emit(node, name):
    choices = node.get('oneOf')
    if choices:
        # Tagged kinds/types remain typed enums with strict per-variant fields.
        tag = next((key for key in ('kind', 'type') if all(
            isinstance(v.get('properties', {}).get(key, {}).get('const'), str) for v in choices)), None)
        if tag:
            body = []
            for choice in choices:
                value = choice['properties'][tag]['const']
                content = variant_fields(choice, name + pascal(value), tag)
                body.append(f'    #[serde(rename = "{value}")]\n    {pascal(value)} {{\n{content}\n    }},')
            DEFINITIONS.append(DERIVE + f'#[serde(tag = "{tag}", deny_unknown_fields)]\npub enum {name} {{\n' + '\n'.join(body) + '\n}\n')
        elif name in ('RawExpr', 'RawOperand'):
            body = []
            for choice in choices:
                key, prop = next(iter(choice['properties'].items()))
                body.append(f'    #[serde(rename = "{key}")]\n    {pascal(key)}({rust_type(prop, name + pascal(key))}),')
            DEFINITIONS.append(DERIVE + f'pub enum {name} {{\n' + '\n'.join(body) + '\n}\n')
        else:
            body = []
            for index, choice in enumerate(choices):
                if choice.get('type') == 'null':
                    continue
                label = ('Local', 'External')[index] if name == 'RawRef' else ('Text', 'Integer')[index] if name == 'RawValue' else f'Variant{index + 1}'
                ty = rust_type(choice, name + label)
                body.append(f'    {label}({ty}),')
            DEFINITIONS.append(DERIVE + '#[serde(untagged)]\n' + f'pub enum {name} {{\n' + '\n'.join(body) + '\n}\n')
    elif node.get('type') == 'object' and 'properties' in node:
        content = fields(node, name)
        DEFINITIONS.append(DERIVE + '#[serde(deny_unknown_fields)]\n' + f'pub struct {name} {{\n{content}\n}}\n')
    else:
        ty = rust_type(node, name, inline=True)
        if ty != name:
            DEFINITIONS.append(f'pub type {name} = {ty};\n')
    return name


def rust_type(node, name, inline=False, in_array=False):
    if name == 'RawFlowSchemaEncoding':
        name = 'RawEncodingSpec'
    if '$ref' in node:
        target = NAMES[node['$ref'].split('/')[-1]]
        return f'Box<{target}>' if target == 'RawExpr' and not in_array else target
    choices = node.get('oneOf')
    if choices:
        non_null = [v for v in choices if v.get('type') != 'null']
        if len(non_null) != len(choices):
            assert len(non_null) == 1
            return f'Option<{rust_type(non_null[0], name)}>'
        return emit(node, name)
    values = node.get('enum') or ([node['const']] if 'const' in node else None)
    if values and all(isinstance(v, str) for v in values):
        return string_enum(values, name)
    kind = node.get('type')
    if kind == 'string':
        return 'String'
    if kind == 'boolean' or isinstance(node.get('const'), bool):
        return 'bool'
    if kind == 'integer' or values and all(isinstance(v, int) for v in values):
        if node.get('maximum') == 255:
            return 'u8'
        if node.get('maximum') == 4294967295 or values:
            return 'u32'
        return 'RawInteger'
    if kind == 'array':
        return f'Vec<{rust_type(node["items"], name + "Item", in_array=True)}>'
    if kind == 'object':
        if 'properties' in node:
            return emit(node, name)
        extra = node.get('additionalProperties')
        return f'BTreeMap<String, {rust_type(extra, name + "Value")}>' if isinstance(extra, dict) else 'BTreeMap<String, Value>'
    raise ValueError((name, node))


HEADER = '''//! Raw format-1/2 DTO superset, generated from the approved flow-v2.schema.json.
//! Format-specific acceptance belongs to lint/compiler; raw data is not executable.
//! Not executable schemas: bounds, references and compatibility require lint/compile.
//! Regenerate using docs/verification/generate_raw_model.py --write.

use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{Error, MapAccess, Visitor};
use serde_json::{Number, Value};
use std::{collections::BTreeMap, fmt, marker::PhantomData};

/// Adapter-supplied binding, excluded from authoring JSON and never inferred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawResolvedEncoding {
    Utf8,
    Windows1252,
    Iso8859_1,
    Other(String),
}

/// Exact JSON integer, including values beyond i64/u64; no rounding via float.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Number", into = "Number")]
pub struct RawInteger(pub Number);

impl TryFrom<Number> for RawInteger {
    type Error = &'static str;
    fn try_from(number: Number) -> Result<Self, Self::Error> {
        let text = number.to_string();
        let digits = text.strip_prefix('-').unwrap_or(&text);
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            Ok(Self(number))
        } else {
            Err("expected a JSON integer without fraction or exponent")
        }
    }
}
impl From<RawInteger> for Number {
    fn from(value: RawInteger) -> Self { value.0 }
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn unique_map<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<BTreeMap<String, T>, D::Error> {
    struct Unique<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Unique<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("an object with unique keys") }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if values.insert(key, value).is_some() { return Err(A::Error::custom("duplicate map key")); }
            }
            Ok(values)
        }
    }
    d.deserialize_map(Unique(PhantomData))
}
'''


def main():
    referenced = set(re.findall(r'"\$ref": "#/\$defs/([^"]+)"', PATH.read_text()))
    for key, node in GRAMMAR['$defs'].items():
        if key in referenced:
            emit(node, NAMES[key])
    emit(GRAMMAR, 'RawFlowSchema')
    text = HEADER + '\n// Grammar SHA-256: ' + hashlib.sha256(PATH.read_bytes()).hexdigest() + '\n\n' + '\n'.join(DEFINITIONS)
    text += '\n#[cfg(test)]\n#[path = "raw_tests.rs"]\nmod tests;\n'
    formatted = subprocess.run(['rustfmt', '--edition', '2024', '--emit', 'stdout'], input=text, text=True, capture_output=True, check=True).stdout
    output = ROOT / 'src/schema/raw.rs'
    if sys.argv[1:] == ['--write']:
        output.write_text(formatted)
    else:
        assert sys.argv[1:] == ['--check']
        assert output.read_text() == formatted, 'raw.rs differs from approved grammar generation'
    print('PASS: reproducible format-1/2 DTO superset; format acceptance remains in lint/compiler')


if __name__ == '__main__':
    main()
