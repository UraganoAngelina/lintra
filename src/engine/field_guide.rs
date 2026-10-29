//! Read-only explanations of compiled declarations, never additional validators.
//! Consumers can page a domain without collecting a whole interval in memory.
use crate::{
    input::encoding::render_diagnostic_value,
    schema::compiled::{
        AlignmentRule, CompiledConstraint, CompiledFieldSpec, ContentBoundaryPolicy, EncodingSpec,
        FieldType, IntegerLiteral, LexicalPlan,
    },
};

/// An exact, resumable enumeration. Range arithmetic works on decimal digits,
/// so even schema bounds larger than u128 remain consultable. The iterator owns
/// only the current integer and borrows an enum's already compiled values.
pub enum DeclaredDomain<'a> {
    Values(std::slice::Iter<'a, Vec<u8>>, &'a EncodingSpec),
    Range {
        next: Option<IntegerLiteral>,
        max: &'a IntegerLiteral,
    },
}
impl Iterator for DeclaredDomain<'_> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Values(values, encoding) => values.next().map(|value| visible(value, encoding)),
            Self::Range { next, max } => {
                let value = next.take()?;
                if value < **max {
                    *next = Some(successor(&value));
                }
                Some(value.to_string())
            }
        }
    }
}
/// Only literal enums and integer ranges are enumerable here. Regexes and
/// reference datasets must not be replaced with invented example domains.
pub fn declared_domain<'a>(
    constraint: &'a CompiledConstraint,
    encoding: &'a EncodingSpec,
) -> Option<DeclaredDomain<'a>> {
    match constraint {
        CompiledConstraint::AllowedValues { values, .. }
        | CompiledConstraint::ReferenceData {
            fallback_values: values,
            ..
        } if !values.is_empty() => Some(DeclaredDomain::Values(values.iter(), encoding)),
        CompiledConstraint::Range { min, max } => Some(DeclaredDomain::Range {
            next: (min <= max).then(|| min.clone()),
            max,
        }),
        _ => None,
    }
}

fn successor(value: &IntegerLiteral) -> IntegerLiteral {
    let mut digits = value.magnitude().to_vec();
    if value.is_negative() {
        for digit in digits.iter_mut().rev() {
            if *digit != b'0' {
                *digit -= 1;
                break;
            }
            *digit = b'9';
        }
    } else {
        let mut carry = true;
        for digit in digits.iter_mut().rev() {
            if *digit != b'9' {
                *digit += 1;
                carry = false;
                break;
            }
            *digit = b'0';
        }
        if carry {
            digits.insert(0, b'1');
        }
    }
    // Every mutation above preserves decimal syntax; FromStr canonicalizes -0.
    let mut text = String::from_utf8(digits).expect("decimal digits are ASCII");
    if value.is_negative() {
        text.insert(0, '-');
    }
    text.parse().expect("successor preserves integer syntax")
}

/// Quotes delimit width; spaces and controls are explicit rather than invisible.
fn visible(bytes: &[u8], encoding: &EncodingSpec) -> String {
    let value = render_diagnostic_value(bytes, encoding);
    let escaped = value.display.replace('"', "\\\"");
    format!(
        "\"{}\"{}",
        escaped.replace(' ', "␠"),
        value
            .raw_hex
            .map(|hex| format!(" [hex {hex}]"))
            .unwrap_or_default()
    )
}

/// Shared rendering for schema and reference literals: quotes distinguish an
/// empty literal from no values; fill, controls and undecodable bytes stay visible.
pub fn domain_literal(bytes: &[u8], encoding: &EncodingSpec) -> String {
    visible(bytes, encoding)
}

/// Physical representation is described independently of the failed predicate.
/// Format 1 does not implement a physical alignment validator: this explanation
/// does not claim a domain failure proves incorrect positioning.
pub fn field_format_guide(field: &CompiledFieldSpec, encoding: &EncodingSpec) -> String {
    let kind = match field.field_type() {
        FieldType::Numeric => "Numerico: cifre ASCII da 0 a 9, senza segno né separatori".to_owned(),
        FieldType::Alphanumeric => "Alfanumerico: testo nella codifica dello schema; le cifre non sono vietate dal solo tipo AN. Restano applicabili i vincoli lessicali e di dominio dichiarati".to_owned(),
        FieldType::Date(_) => "Data: GGMMAAAA, con giorno, mese e anno validi".to_owned(),
        FieldType::Decimal(spec) => format!("Decimale senza segno: {} byte, {} cifre dopo il separatore {}", spec.total_length(), spec.scale(), visible(&[spec.decimal_separator()], encoding)),
    };
    let lexical = match field.lexical() {
        Some(LexicalPlan::ByteSet(class)) => {
            let values: Vec<_> = class
                .allowed
                .iter()
                .enumerate()
                .filter(|(_, allowed)| **allowed)
                .map(|(byte, _)| visible(&[byte as u8], encoding))
                .collect();
            format!(
                " Classe lessicale dichiarata (byte ammessi): {}.",
                values.join(", ")
            )
        }
        _ => String::new(),
    };
    let encoding_name = match encoding {
        EncodingSpec::Utf8 => "UTF-8",
        EncodingSpec::Windows1252 => "Windows-1252",
        EncodingSpec::Iso8859_1 => "ISO-8859-1",
        EncodingSpec::Other(name) => name,
    };
    let alignment = match field.alignment() {
        AlignmentRule::Exact => "Larghezza esatta: nessuna rimozione di riempimento".to_owned(),
        AlignmentRule::LeftPad {
            fill,
            content_policy,
        } => padding("destra", "sinistra", fill, content_policy, encoding),
        AlignmentRule::RightPad {
            fill,
            content_policy,
        } => padding("sinistra", "destra", fill, content_policy, encoding),
    };
    format!(
        "{kind}.{lexical} Slot: byte {}–{} inclusi ({} byte). {alignment}. Codifica: {encoding_name}.",
        field.range().start() + 1,
        field.range().end_exclusive(),
        field.range().len()
    )
}
fn padding(
    content_side: &str,
    fill_side: &str,
    fill: u8,
    policy: ContentBoundaryPolicy,
    encoding: &EncodingSpec,
) -> String {
    let extraction = match policy {
        ContentBoundaryPolicy::StripLeadingFill => {
            "il valore logico esclude il riempimento iniziale; uno slot composto soltanto da fill produce un valore logico vuoto"
        }
        ContentBoundaryPolicy::StripTrailingFill => {
            "il valore logico esclude il riempimento finale; uno slot composto soltanto da fill produce un valore logico vuoto"
        }
        ContentBoundaryPolicy::PreserveWidth => "il valore logico conserva l'intera larghezza",
    };
    format!(
        "Contenuto a {content_side}; posizioni residue a {fill_side} riempite con {} (0x{fill:02X}); {extraction}",
        visible(&[fill], encoding)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_are_complete_signed_and_not_machine_bounded() {
        for (min, max, expected) in [
            (
                "-11",
                "2",
                vec![
                    "-11", "-10", "-9", "-8", "-7", "-6", "-5", "-4", "-3", "-2", "-1", "0", "1",
                    "2",
                ],
            ),
            (
                "340282366920938463463374607431768211455",
                "340282366920938463463374607431768211456",
                vec![
                    "340282366920938463463374607431768211455",
                    "340282366920938463463374607431768211456",
                ],
            ),
        ] {
            let constraint = CompiledConstraint::Range {
                min: min.parse().unwrap(),
                max: max.parse().unwrap(),
            };
            let mut domain = declared_domain(&constraint, &EncodingSpec::Utf8).unwrap();
            assert_eq!(domain.by_ref().take(2).collect::<Vec<_>>(), expected[..2]);
            assert_eq!(domain.collect::<Vec<_>>(), expected[2..]);
        }
    }
    #[test]
    fn enum_retains_padding_and_non_enumerable_constraints_are_explicit() {
        let constraint = CompiledConstraint::AllowedValues {
            values: vec![b" A".to_vec(), b"00".to_vec()],
            target: crate::ValueTarget::Raw,
        };
        assert_eq!(
            declared_domain(&constraint, &EncodingSpec::Utf8)
                .unwrap()
                .collect::<Vec<_>>(),
            ["\"␠A\"", "\"00\""]
        );
        assert!(declared_domain(&CompiledConstraint::NonEmpty, &EncodingSpec::Utf8).is_none());
        assert_eq!(domain_literal(b"", &EncodingSpec::Utf8), "\"\"");
        assert_eq!(domain_literal(b" A ", &EncodingSpec::Utf8), "\"␠A␠\"");
    }
}

/// A continuation belongs to the same compiled constraint selected by the caller.
/// Values are schema literals, never patient data. Invalid cursors fail closed.
#[derive(Clone, Debug)]
pub enum DomainPosition {
    Enum(usize),
    Integer(IntegerLiteral),
}
pub struct DomainPage {
    pub values: Vec<String>,
    pub next: Option<DomainPosition>,
}
/// At most 64 entries are allocated, independent of the historical data volume
/// or interval cardinality. Integer continuations avoid rescanning prior pages.
pub fn declared_domain_page(
    constraint: &CompiledConstraint,
    encoding: &EncodingSpec,
    position: Option<&DomainPosition>,
) -> Option<DomainPage> {
    const PAGE: usize = 64;
    match constraint {
        CompiledConstraint::AllowedValues { values, .. }
        | CompiledConstraint::ReferenceData {
            fallback_values: values,
            ..
        } if !values.is_empty() => {
            let start = match position {
                None => 0,
                Some(DomainPosition::Enum(index)) if *index <= values.len() => *index,
                _ => return None,
            };
            let end = start.saturating_add(PAGE).min(values.len());
            Some(DomainPage {
                values: values[start..end]
                    .iter()
                    .map(|v| visible(v, encoding))
                    .collect(),
                next: (end < values.len()).then_some(DomainPosition::Enum(end)),
            })
        }
        CompiledConstraint::Range { min, max } => {
            let start = match position {
                None => min.clone(),
                Some(DomainPosition::Integer(value)) if value >= min && value <= max => {
                    value.clone()
                }
                _ => return None,
            };
            let mut domain = DeclaredDomain::Range {
                next: (start <= *max).then_some(start),
                max,
            };
            let values = domain.by_ref().take(PAGE).collect();
            let next = match domain {
                DeclaredDomain::Range { next, .. } => next.map(DomainPosition::Integer),
                _ => unreachable!(),
            };
            Some(DomainPage { values, next })
        }
        _ => None,
    }
}

#[cfg(test)]
mod paging_tests {
    use super::*;
    #[test]
    fn paging_has_no_gaps_and_rejects_wrong_continuations() {
        let constraint = CompiledConstraint::Range {
            min: "-100".parse().unwrap(),
            max: "100".parse().unwrap(),
        };
        let mut position = None;
        let mut actual = Vec::new();
        loop {
            let page =
                declared_domain_page(&constraint, &EncodingSpec::Utf8, position.as_ref()).unwrap();
            assert!(page.values.len() <= 64);
            actual.extend(page.values);
            position = page.next;
            if position.is_none() {
                break;
            }
        }
        assert_eq!(
            actual,
            (-100..=100)
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        );
        assert!(
            declared_domain_page(
                &constraint,
                &EncodingSpec::Utf8,
                Some(&DomainPosition::Enum(0))
            )
            .is_none()
        );
        assert!(
            declared_domain_page(
                &constraint,
                &EncodingSpec::Utf8,
                Some(&DomainPosition::Integer("101".parse().unwrap()))
            )
            .is_none()
        );
        let huge = CompiledConstraint::Range {
            min: "0".parse().unwrap(),
            max: "999999999999999999999999999999999999999999999999999999999999999999"
                .parse()
                .unwrap(),
        };
        let page = declared_domain_page(&huge, &EncodingSpec::Utf8, None).unwrap();
        assert_eq!(page.values.len(), 64);
        assert!(page.next.is_some());
    }
}

/// Explains which bytes a declaration compares, without executing that rule.
pub fn constraint_comparison_guide(constraint: &CompiledConstraint) -> &'static str {
    use crate::ValueTarget;
    match constraint {
        CompiledConstraint::AllowedValues { target, .. }
        | CompiledConstraint::Regex { target, .. } => match target {
            ValueTarget::Raw => "Confronto sullo slot completo (raw, riempimento incluso).",
            ValueTarget::Logical => {
                "Confronto sul valore logico dopo l’estrazione del riempimento dichiarata."
            }
        },
        CompiledConstraint::Range { .. } => {
            "Confronto sul valore intero logico; gli zeri iniziali non ne cambiano il valore."
        }
        CompiledConstraint::ReferenceData { .. } => {
            "Confronto sui byte esatti del valore logico; i fallback sono letterali separati dai dataset."
        }
        _ => "Vincolo dichiarato nello schema.",
    }
}
