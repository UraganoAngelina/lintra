//! Controlled message placeholders; no expression language or recursive expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Token {
    RuleId,
    FieldId,
    FieldName,
    Condition,
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum Part<'a> {
    Text(&'a str),
    Token(Token),
}

/// Every brace must form one known placeholder. No undocumented escaping or
/// implicit interpolation is accepted; all other UTF-8 text is retained exactly.
pub(crate) fn parse(template: &str) -> Result<Vec<Part<'_>>, &'static str> {
    if template.is_empty() {
        return Err("rule message template must be nonempty");
    }
    let mut rest = template;
    let mut parts = Vec::new();
    while let Some(index) = rest.find(['{', '}']) {
        if index > 0 {
            parts.push(Part::Text(&rest[..index]));
        }
        rest = &rest[index..];
        if rest.starts_with('}') {
            return Err("unmatched closing template brace");
        }
        let end = rest.find('}').ok_or("unclosed template placeholder")?;
        let token = match &rest[1..end] {
            "rule_id" => Token::RuleId,
            "field_id" => Token::FieldId,
            "field_name" => Token::FieldName,
            "condition" => Token::Condition,
            _ => return Err("unsupported template placeholder"),
        };
        parts.push(Part::Token(token));
        rest = &rest[end + 1..];
    }
    if !rest.is_empty() {
        parts.push(Part::Text(rest));
    }
    Ok(parts)
}
