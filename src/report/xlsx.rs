//! XLSX backend qualification (T-P12-02).
//!
//! The production JSONL-to-workbook writer is T-P12-03. These probes create
//! real constant-memory workbooks; no report writer or ValidationSink is faked.
#[cfg(test)]
mod tests {
    use crate::ERROR_COLUMNS;
    use rust_xlsxwriter::{Workbook, XlsxError};
    use std::{
        fs,
        io::{self, Cursor, Read, Write},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn workbook() -> Workbook {
        let mut workbook = Workbook::new();
        workbook
            .add_worksheet_with_constant_memory()
            .unwrap()
            .set_name("Riepilogo")
            .unwrap()
            .write_string(0, 0, "Synthetic backend probe")
            .unwrap();
        let sheet = workbook.add_worksheet_with_constant_memory().unwrap();
        sheet.set_name("Errori").unwrap();
        for (column, descriptor) in ERROR_COLUMNS.iter().enumerate() {
            sheet
                .write_string(0, column.try_into().unwrap(), descriptor.header())
                .unwrap();
        }
        // Explicit strings: neither leading zeros nor formula-looking values
        // may be reinterpreted by the writer chosen for the future report.
        sheet.write_string(1, 0, "00123").unwrap();
        sheet.write_string(1, 1, "=1+1").unwrap();
        sheet.write_string(1, 2, "à<&\"\n").unwrap();
        workbook
    }
    fn xml(bytes: &[u8], name: &str) -> String {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut text = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn constant_memory_workbook_preserves_text_and_shared_column_headers() {
        let bytes = workbook().save_to_buffer().unwrap();
        assert!(bytes.starts_with(b"PK\x03\x04"));
        let metadata = xml(&bytes, "xl/workbook.xml");
        assert!(metadata.contains("name=\"Riepilogo\""));
        assert!(metadata.contains("name=\"Errori\""));
        let sheet = xml(&bytes, "xl/worksheets/sheet2.xml");
        for column in ERROR_COLUMNS {
            assert!(sheet.contains(&format!("<t>{}</t>", column.header())));
        }
        assert!(sheet.contains("t=\"inlineStr\""));
        assert!(sheet.contains("<t>00123</t>"));
        assert!(sheet.contains("<t>=1+1</t>"));
        assert!(sheet.contains("à&lt;&amp;\""));
        assert!(!sheet.contains("<f>"));
        assert!(
            zip::ZipArchive::new(Cursor::new(&bytes))
                .unwrap()
                .by_name("xl/sharedStrings.xml")
                .is_err()
        );
    }

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "lintra xlsx à {} {}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn native_workbook_file_with_unicode_path_is_a_readable_zip() {
        let directory = Directory::new();
        let path = directory.0.join("report à.xlsx");
        workbook().save(&path).unwrap();
        let bytes = fs::read(path).unwrap();
        assert!(xml(&bytes, "xl/worksheets/sheet2.xml").contains("<t>00123</t>"));
    }

    #[test]
    fn sequential_constant_memory_rows_survive_flushes_without_shared_string_table() {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet_with_constant_memory().unwrap();
        for row in 0..10_000 {
            sheet.write_string(row, 0, format!("{row:05}")).unwrap();
        }
        let bytes = workbook.save_to_buffer().unwrap();
        let sheet = xml(&bytes, "xl/worksheets/sheet1.xml");
        assert_eq!(sheet.matches("<row ").count(), 10_000);
        assert!(sheet.contains("<t>00000</t>"));
        assert!(sheet.contains("<t>09999</t>"));
        assert!(!sheet.contains("t=\"s\""));
    }

    #[test]
    fn backend_errors_include_unusable_tempdir_and_output_failure() {
        let directory = Directory::new();
        assert!(matches!(
            Workbook::new().set_tempdir(directory.0.join("missing")),
            Err(XlsxError::TempFileError(_))
        ));
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::StorageFull,
                    "synthetic output",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let error = workbook().save_to_writer(FailingWriter).unwrap_err();
        let source = match error {
            XlsxError::IoError(source) | XlsxError::ZipError(zip::result::ZipError::Io(source)) => {
                source
            }
            other => panic!("unexpected technical cause: {other:?}"),
        };
        assert_eq!(source.kind(), io::ErrorKind::StorageFull);
    }
}
