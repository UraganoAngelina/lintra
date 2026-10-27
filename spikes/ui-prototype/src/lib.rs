slint::include_modules!();

#[allow(clippy::too_many_arguments)]
fn row(
    line: &str,
    record: &str,
    field: &str,
    code: &str,
    severity: i32,
    message: &str,
    code_name: &str,
    position: &str,
    actual: &str,
    expected: &str,
    context: &str,
    detail: &str,
) -> ErrorRow {
    ErrorRow {
        line: line.into(),
        record: record.into(),
        field: field.into(),
        code: code.into(),
        severity,
        message: message.into(),
        code_name: code_name.into(),
        position: position.into(),
        actual: actual.into(),
        expected: expected.into(),
        context: context.into(),
        detail: detail.into(),
    }
}

/// Dati di esempio. Con n <= 4 restituisce le righe del mockup; con n > 4 genera righe sintetiche
/// per provare lo scorrimento di tabelle grandi.
pub fn sample_rows(n: usize) -> Vec<ErrorRow> {
    let mut rows = vec![
        row("184", "REC-000184", "2.29", "E003", 0, "Campo obbligatorio non valorizzato",
            "E003 CONDITIONAL_REQUIRED", "118–120", "«   » (3 spazi)",
            "codice Regione di 3 caratteri", "2.2 = 05",
            "Il campo 2.29 è obbligatorio perché il campo 2.2 vale «05», valore diverso da «01» e «02». Il campo risulta non valorizzato."),
        row("221", "REC-000221", "2.7", "E007", 1, "Valore non ammesso dal dominio",
            "E007 INVALID_DOMAIN", "[posizione]", "[valore letto]", "[valore atteso]", "[contesto]", "[messaggio]"),
        row("305", "REC-000305", "2.14", "E005", 1, "Allineamento non conforme al padding",
            "E005 ALIGNMENT", "[posizione]", "[valore letto]", "[valore atteso]", "[contesto]", "[messaggio]"),
        row("412", "REC-000412", "2.31", "E009", 1, "Data non valida",
            "E009 INVALID_DATE", "[posizione]", "[valore letto]", "[valore atteso]", "[contesto]", "[messaggio]"),
    ];
    if n <= 4 {
        rows.truncate(n);
        return rows;
    }
    let kinds = [
        (
            "E003",
            0,
            "Campo obbligatorio non valorizzato",
            "E003 CONDITIONAL_REQUIRED",
        ),
        (
            "E007",
            1,
            "Valore non ammesso dal dominio",
            "E007 INVALID_DOMAIN",
        ),
        (
            "E005",
            1,
            "Allineamento non conforme al padding",
            "E005 ALIGNMENT",
        ),
        ("E009", 1, "Data non valida", "E009 INVALID_DATE"),
    ];
    for i in rows.len()..n {
        let (code, sev, msg, name) = kinds[i % kinds.len()];
        rows.push(row(
            &format!("{}", 1000 + i),
            &format!("REC-{:06}", 1000 + i),
            &format!("2.{}", 1 + i % 40),
            code,
            sev,
            msg,
            name,
            "[posizione]",
            "[valore letto]",
            "[valore atteso]",
            "[contesto]",
            "[messaggio]",
        ));
    }
    rows
}
