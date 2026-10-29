//! Real local repository. SQL exists only in this adapter; no regulatory rules,
//! approval transitions, raw clinical fields or implicit storage discovery.
use super::*;
use crate::ProtectionError;
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::{Mutex, MutexGuard},
};
mod codec;
mod security;
// Format2 adds explicit omission-consent metadata; legacy payloads/MACs stay valid.
const FORMAT: i64 = 2;
const APPLICATION: i64 = 0x4c4e5452;
const SCHEMA: &str = r#"
CREATE TABLE storage_meta(singleton INTEGER PRIMARY KEY CHECK(singleton=1),audit BLOB NOT NULL);
CREATE TABLE namespaces(ns BLOB PRIMARY KEY,generation INTEGER NOT NULL CHECK(generation>0)) WITHOUT ROWID;
CREATE TABLE snapshots(ns BLOB NOT NULL,revision INTEGER NOT NULL CHECK(revision>0),submission TEXT NOT NULL,source BLOB NOT NULL,proof BLOB NOT NULL CHECK(length(proof)=32),record_count INTEGER NOT NULL CHECK(record_count>=0),PRIMARY KEY(ns,revision),UNIQUE(ns,submission),FOREIGN KEY(ns) REFERENCES namespaces(ns)) WITHOUT ROWID;
CREATE TABLE records(ns BLOB NOT NULL,revision INTEGER NOT NULL,ordinal INTEGER NOT NULL,identity BLOB NOT NULL,payload BLOB NOT NULL,key_proof BLOB NOT NULL CHECK(length(key_proof)=32),record_proof BLOB NOT NULL CHECK(length(record_proof)=32),PRIMARY KEY(ns,revision,ordinal),UNIQUE(ns,revision,identity),FOREIGN KEY(ns,revision) REFERENCES snapshots(ns,revision)) WITHOUT ROWID;
CREATE TABLE latest(ns BLOB NOT NULL,identity BLOB NOT NULL,revision INTEGER NOT NULL,ordinal INTEGER NOT NULL,PRIMARY KEY(ns,identity),FOREIGN KEY(ns,revision,ordinal) REFERENCES records(ns,revision,ordinal)) WITHOUT ROWID;
"#;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SQLiteStorageSettings {
    pub format_version: u32,
    pub sqlite_version: String,
    pub journal_mode: String,
    pub synchronous: i64,
    pub cache_kib: i64,
    pub mmap_bytes: i64,
}
struct Inner {
    connection: Mutex<Connection>,
    protector: IdentityProtector,
    _database: File,
    _root: security::RootGuard,
    #[cfg(test)]
    fault: Mutex<Option<TestFault>>,
}
#[derive(Clone)]
pub struct SQLitePersistenceAdapter {
    inner: Arc<Inner>,
}
impl fmt::Debug for SQLitePersistenceAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SQLitePersistenceAdapter { .. }")
    }
}
fn sql(error: rusqlite::Error) -> PersistenceError {
    match error.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
            PersistenceError::StorageBusy
        }
        Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase) => {
            PersistenceError::CorruptStorage
        }
        _ => PersistenceError::StorageUnavailable,
    }
}
fn number(value: u64) -> Result<i64, PersistenceError> {
    i64::try_from(value).map_err(|_| PersistenceError::RevisionOverflow)
}
fn header(file: &File) -> Result<(), PersistenceError> {
    let mut file = file
        .try_clone()
        .map_err(|_| PersistenceError::StorageUnavailable)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| PersistenceError::StorageUnavailable)?;
    let mut bytes = [0u8; 100];
    file.read_exact(&mut bytes)
        .map_err(|_| PersistenceError::CorruptStorage)?;
    if &bytes[..16] != b"SQLite format 3\0" {
        return Err(PersistenceError::CorruptStorage);
    }
    let word = |start: usize| {
        u32::from_be_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ]) as i64
    };
    if word(68) != APPLICATION {
        return Err(PersistenceError::UnrecognizedStorage);
    }
    if !matches!(word(60), 1 | FORMAT) {
        return Err(PersistenceError::UnsupportedStorageVersion { found: word(60) });
    }
    if bytes[18] != 1 || bytes[19] != 1 {
        return Err(PersistenceError::UnsupportedStorageConfiguration);
    }
    Ok(())
}
fn configure(c: &Connection, new: bool) -> Result<(), PersistenceError> {
    c.busy_timeout(std::time::Duration::ZERO).map_err(sql)?;
    if !new {
        let mode: String = c
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .map_err(sql)?;
        if mode != "delete" {
            return Err(PersistenceError::UnsupportedStorageConfiguration);
        }
    }
    let mode: String = c
        .query_row("PRAGMA journal_mode=DELETE", [], |r| r.get(0))
        .map_err(sql)?;
    c.execute_batch("PRAGMA synchronous=EXTRA; PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF; PRAGMA cache_size=-2048; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;").map_err(sql)?;
    c.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
        .map_err(sql)?;
    let read = |name: &str| {
        c.pragma_query_value(None, name, |r| r.get::<_, i64>(0))
            .map_err(sql)
    };
    if mode != "delete"
        || read("synchronous")? != 3
        || read("foreign_keys")? != 1
        || read("trusted_schema")? != 0
        || read("cache_size")? != -2048
        || read("mmap_size")? != 0
    {
        return Err(PersistenceError::UnsupportedStorageConfiguration);
    }
    Ok(())
}
impl SQLitePersistenceAdapter {
    /// Explicit NEW local archive; never used as a fallback from open failure.
    pub fn create(
        root: &Path,
        key_id: Option<&str>,
        provider: &dyn PersistenceKeyProvider,
    ) -> Result<Self, PersistenceError> {
        let protector = IdentityProtector::preflight(key_id, provider, None)
            .map_err(PersistenceError::Protection)?;
        let root = security::root(root, true)?;
        security::check_files(&root.path, true)?;
        let file = security::database(&root.path, true)?;
        let mut connection = Connection::open_with_flags(
            root.path.join("lintra.db"),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(sql)?;
        configure(&connection, true)?;
        {
            let tx = connection
                .transaction_with_behavior(TransactionBehavior::Exclusive)
                .map_err(sql)?;
            tx.execute_batch(SCHEMA).map_err(sql)?;
            tx.pragma_update(None, "application_id", APPLICATION)
                .map_err(sql)?;
            tx.pragma_update(None, "user_version", FORMAT)
                .map_err(sql)?;
            tx.execute(
                "INSERT INTO storage_meta VALUES(1,?1)",
                [codec::encode(
                    &protector.audit().map_err(PersistenceError::Protection)?,
                )?],
            )
            .map_err(sql)?;
            tx.commit().map_err(sql)?;
        }
        connection.set_prepared_statement_cache_capacity(8);
        Ok(Self {
            inner: Arc::new(Inner {
                connection: Mutex::new(connection),
                protector,
                _database: file,
                _root: root,
                #[cfg(test)]
                fault: Mutex::new(None),
            }),
        })
    }
    /// Existing archive only. Header/version and filesystem are checked before
    /// SQLite can run recovery or mutate a foreign/unrecognized database.
    pub fn open(
        root: &Path,
        key_id: Option<&str>,
        provider: &dyn PersistenceKeyProvider,
    ) -> Result<Self, PersistenceError> {
        let root = security::root(root, false)?;
        security::check_files(&root.path, false)?;
        let file = security::database(&root.path, false)?;
        header(&file)?;
        let connection = Connection::open_with_flags(
            root.path.join("lintra.db"),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(sql)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(sql)?;
        if !matches!(version, 1 | FORMAT) {
            return Err(PersistenceError::UnsupportedStorageVersion { found: version });
        }
        let integrity: String = connection
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(sql)?;
        if integrity != "ok" {
            return Err(PersistenceError::CorruptStorage);
        }
        // A structurally valid SQLite file with missing/replaced Lintra tables is
        // corrupt storage, not an empty archive or an infrastructure outage.
        for probe in [
            "SELECT ns,generation FROM namespaces LIMIT 0",
            "SELECT ns,revision,submission,source,proof,record_count FROM snapshots LIMIT 0",
            "SELECT ns,revision,ordinal,identity,payload,key_proof,record_proof FROM records LIMIT 0",
            "SELECT ns,identity,revision,ordinal FROM latest LIMIT 0",
        ] {
            connection
                .prepare(probe)
                .map_err(|_| PersistenceError::CorruptStorage)?;
        }
        let audit: Vec<u8> = connection
            .query_row(
                "SELECT audit FROM storage_meta WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(|_| PersistenceError::CorruptStorage)?;
        let audit: ProtectionAudit = codec::decode(&audit)?;
        let protector = IdentityProtector::preflight(key_id, provider, Some(&audit))
            .map_err(PersistenceError::Protection)?;
        configure(&connection, false)?;
        connection.set_prepared_statement_cache_capacity(8);
        let invalid = connection
            .prepare("PRAGMA foreign_key_check")
            .map_err(sql)?
            .exists([])
            .map_err(sql)?;
        if invalid {
            return Err(PersistenceError::CorruptStorage);
        }
        Ok(Self {
            inner: Arc::new(Inner {
                connection: Mutex::new(connection),
                protector,
                _database: file,
                _root: root,
                #[cfg(test)]
                fault: Mutex::new(None),
            }),
        })
    }
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, PersistenceError> {
        let connection = self
            .inner
            .connection
            .lock()
            .map_err(|_| PersistenceError::StorageUnavailable)?;
        // A failed rollback may leave a private transaction active after an I/O
        // fault. Never expose that connection's uncommitted view as published.
        // Reopening the archive performs SQLite recovery before probing again.
        if !connection.is_autocommit() {
            return Err(PersistenceError::StorageUnavailable);
        }
        storage_version(&connection)?;
        Ok(connection)
    }
    pub fn settings(&self) -> Result<SQLiteStorageSettings, PersistenceError> {
        let c = self.lock()?;
        Ok(SQLiteStorageSettings {
            format_version: storage_version(&c)? as u32,
            sqlite_version: rusqlite::version().into(),
            journal_mode: c
                .pragma_query_value(None, "journal_mode", |r| r.get(0))
                .map_err(sql)?,
            synchronous: c
                .pragma_query_value(None, "synchronous", |r| r.get(0))
                .map_err(sql)?,
            cache_kib: 2048,
            mmap_bytes: c
                .pragma_query_value(None, "mmap_size", |r| r.get(0))
                .map_err(sql)?,
        })
    }
    /// Explicit, non-destructive capability upgrade. Open never calls this.
    /// Close older application instances before updating this local archive.
    /// No source, signature, snapshot, generation or index is rewritten.
    pub fn upgrade_storage_v1_to_v2(&self) -> Result<(), PersistenceError> {
        let mut c = self.lock()?;
        if storage_version(&c)? == FORMAT {
            return Ok(());
        }
        #[cfg(test)]
        self.configure_commit_fault(&mut c);
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Exclusive)
            .map_err(sql)?;
        // Another compatible process may have upgraded before this lock.
        if storage_version(&tx)? == 1 {
            tx.pragma_update(None, "user_version", FORMAT)
                .map_err(sql)?;
        }
        #[cfg(test)]
        self.before_commit(&tx)?;
        tx.commit()
            .map_err(|_| PersistenceError::StorageUpgradeOutcomeUnknown)?;
        if !c.is_autocommit() {
            return Err(PersistenceError::StorageUpgradeOutcomeUnknown);
        }
        #[cfg(test)]
        match self.inner.fault.lock().unwrap().take() {
            Some(TestFault::AfterLost) => {
                return Err(PersistenceError::StorageUpgradeOutcomeUnknown);
            }
            Some(TestFault::AfterExit) => std::process::exit(87),
            _ => {}
        }
        Ok(())
    }
    /// Read-only access to the configured producer, already checked against the
    /// archive audit. Services still select fields via compiled profiles.
    pub fn protector(&self) -> &IdentityProtector {
        &self.inner.protector
    }
}
fn storage_version(c: &Connection) -> Result<i64, PersistenceError> {
    let version = c
        .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
        .map_err(sql)?;
    if !matches!(version, 1 | FORMAT) {
        return Err(PersistenceError::UnsupportedStorageVersion { found: version });
    }
    Ok(version)
}
fn allow_source_version(c: &Connection, source: &SourceSubmission) -> Result<(), PersistenceError> {
    let version = storage_version(c)?;
    if version == 1 && source.missing_trace_consent().is_some() {
        return Err(PersistenceError::StorageUpgradeRequired {
            found: 1,
            required: FORMAT,
        });
    }
    Ok(())
}
fn generation(c: &Connection, ns: &[u8]) -> Result<Option<SnapshotRevision>, PersistenceError> {
    let value: Option<i64> = c
        .query_row("SELECT generation FROM namespaces WHERE ns=?1", [ns], |r| {
            r.get(0)
        })
        .optional()
        .map_err(sql)?;
    value
        .map(|v| {
            if v > 0 {
                Ok(SnapshotRevision::new(v as u64))
            } else {
                Err(PersistenceError::CorruptStorage)
            }
        })
        .transpose()
}
fn check(
    expected: Option<SnapshotRevision>,
    actual: Option<SnapshotRevision>,
) -> Result<(), PersistenceError> {
    if expected != actual {
        Err(PersistenceError::GenerationConflict { expected, actual })
    } else {
        Ok(())
    }
}
impl PersistentContextRepository for SQLitePersistenceAdapter {
    fn lookup_identity(
        &self,
        ns: &PersistenceNamespace,
        id: &PersistentRecordIdentity,
    ) -> Result<Option<StoredIdentity>, PersistenceError> {
        if id.namespace() != ns {
            return Err(PersistenceError::NamespaceMismatch);
        }
        self.inner
            .protector
            .verify_identity(id)
            .map_err(PersistenceError::Protection)?;
        let ns = codec::encode(ns)?;
        let key = codec::encode(id)?;
        let c = self.lock()?;
        let result:Option<(Vec<u8>,Vec<u8>,Vec<u8>)>=c.prepare_cached("SELECT r.payload,r.key_proof,r.record_proof FROM latest l JOIN records r ON r.ns=l.ns AND r.revision=l.revision AND r.ordinal=l.ordinal WHERE l.ns=?1 AND l.identity=?2").map_err(sql)?.query_row(params![ns,key],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(sql)?;
        result
            .map(|(body, kp, rp)| {
                let record = codec::read_record(&body, &kp, &rp, &self.inner.protector)?;
                if record.identity() != id {
                    return Err(PersistenceError::CorruptStorage);
                }
                Ok(record)
            })
            .transpose()
    }
    fn load_current_snapshot(
        &self,
        ns: &PersistenceNamespace,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        let encoded = codec::encode(ns)?;
        let c = self.lock()?;
        match generation(&c, &encoded)? {
            Some(revision) => {
                read_snapshot(&c, ns, &encoded, revision, &self.inner.protector).map(Some)
            }
            None => Ok(None),
        }
    }
    fn load_snapshot(
        &self,
        ns: &PersistenceNamespace,
        revision: SnapshotRevision,
    ) -> Result<Option<HistoricalSnapshot>, PersistenceError> {
        let encoded = codec::encode(ns)?;
        let c = self.lock()?;
        let exists = c
            .prepare_cached("SELECT 1 FROM snapshots WHERE ns=?1 AND revision=?2")
            .map_err(sql)?
            .exists(params![encoded, number(revision.value())?])
            .map_err(sql)?;
        if !exists {
            return Ok(None);
        }
        read_snapshot(&c, ns, &encoded, revision, &self.inner.protector).map(Some)
    }
    fn load_submission(
        &self,
        ns: &PersistenceNamespace,
        id: &str,
    ) -> Result<Option<SourceSubmission>, PersistenceError> {
        let encoded = codec::encode(ns)?;
        let c = self.lock()?;
        let value: Option<(Vec<u8>, Vec<u8>)> = c
            .prepare_cached("SELECT source,proof FROM snapshots WHERE ns=?1 AND submission=?2")
            .map_err(sql)?
            .query_row(params![encoded, id], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()
            .map_err(sql)?;
        value
            .map(|(body, proof)| {
                let source = codec::read_source(&body, &proof, &self.inner.protector)?;
                if source.namespace() != ns || source.submission_id() != id {
                    return Err(PersistenceError::CorruptStorage);
                }
                Ok(source)
            })
            .transpose()
    }
    fn begin_transaction(
        &self,
        ns: &PersistenceNamespace,
        expected: Option<SnapshotRevision>,
    ) -> Result<Box<dyn PersistenceTransaction>, PersistenceError> {
        let encoded = codec::encode(ns)?;
        check(expected, generation(&*self.lock()?, &encoded)?)?;
        Ok(Box::new(SQLiteTransaction {
            adapter: self.clone(),
            namespace: ns.clone(),
            expected,
            staged: None,
        }))
    }
}
fn read_snapshot(
    c: &Connection,
    ns: &PersistenceNamespace,
    encoded: &[u8],
    revision: SnapshotRevision,
    p: &IdentityProtector,
) -> Result<HistoricalSnapshot, PersistenceError> {
    let (body, proof, count): (Vec<u8>, Vec<u8>, i64) = c
        .query_row(
            "SELECT source,proof,record_count FROM snapshots WHERE ns=?1 AND revision=?2",
            params![encoded, number(revision.value())?],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|_| PersistenceError::CorruptStorage)?;
    let source = codec::read_source(&body, &proof, p)?;
    if source.namespace() != ns || count < 0 {
        return Err(PersistenceError::CorruptStorage);
    }
    let mut records = Vec::new();
    let mut query=c.prepare("SELECT ordinal,payload,key_proof,record_proof FROM records WHERE ns=?1 AND revision=?2 ORDER BY ordinal").map_err(sql)?;
    let mut rows = query
        .query(params![encoded, number(revision.value())?])
        .map_err(sql)?;
    while let Some(row) = rows.next().map_err(sql)? {
        let ordinal: i64 = row.get(0).map_err(sql)?;
        if ordinal != records.len() as i64 {
            return Err(PersistenceError::CorruptStorage);
        }
        let payload: Vec<u8> = row.get(1).map_err(sql)?;
        let kp: Vec<u8> = row.get(2).map_err(sql)?;
        let rp: Vec<u8> = row.get(3).map_err(sql)?;
        records.push(codec::read_record(&payload, &kp, &rp, p)?);
    }
    if records.len() as i64 != count {
        return Err(PersistenceError::CorruptStorage);
    }
    HistoricalSnapshot::new(revision, source, records).map_err(|_| PersistenceError::CorruptStorage)
}
struct SQLiteTransaction {
    adapter: SQLitePersistenceAdapter,
    namespace: PersistenceNamespace,
    expected: Option<SnapshotRevision>,
    staged: Option<HistoricalSnapshot>,
}
impl PersistenceTransaction for SQLiteTransaction {
    fn stage_snapshot(&mut self, snapshot: &HistoricalSnapshot) -> Result<(), PersistenceError> {
        self.staged = None;
        if snapshot.namespace() != &self.namespace {
            return Err(PersistenceError::NamespaceMismatch);
        }
        let next = self
            .expected
            .map(|r| r.next())
            .transpose()
            .map_err(|_| PersistenceError::RevisionOverflow)?
            .unwrap_or(SnapshotRevision::new(1));
        if snapshot.revision() != next {
            return Err(PersistenceError::InvalidRevision);
        }
        self.adapter
            .inner
            .protector
            .verify_submission(snapshot.source())
            .map_err(PersistenceError::Protection)?;
        for record in snapshot.identities() {
            self.adapter
                .inner
                .protector
                .verify_stored_identity(record)
                .map_err(PersistenceError::Protection)?;
        }
        allow_source_version(&*self.adapter.lock()?, snapshot.source())?;
        self.staged = Some(snapshot.clone());
        Ok(())
    }
    fn commit(self: Box<Self>) -> Result<CommitReceipt, PersistenceError> {
        let snapshot = self.staged.ok_or(PersistenceError::NothingStaged)?;
        let encoded = codec::encode(&self.namespace)?;
        let mut c = self.adapter.lock()?;
        #[cfg(test)]
        self.adapter.configure_commit_fault(&mut c);
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        check(self.expected, generation(&tx, &encoded)?)?;
        allow_source_version(&tx, snapshot.source())?;
        if tx
            .prepare("SELECT 1 FROM snapshots WHERE ns=?1 AND submission=?2")
            .map_err(sql)?
            .exists(params![encoded, snapshot.source().submission_id()])
            .map_err(sql)?
        {
            return Err(PersistenceError::DuplicateSubmission);
        }
        let revision = number(snapshot.revision().value())?;
        tx.execute("INSERT INTO namespaces(ns,generation) VALUES(?1,?2) ON CONFLICT(ns) DO UPDATE SET generation=excluded.generation",params![encoded,revision]).map_err(sql)?;
        let proof = snapshot
            .source()
            .protection_evidence
            .0
            .as_ref()
            .ok_or(PersistenceError::Protection(ProtectionError::Unattested))?;
        tx.execute(
            "INSERT INTO snapshots VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                encoded,
                revision,
                snapshot.source().submission_id(),
                codec::encode(snapshot.source())?,
                proof.as_slice(),
                number(snapshot.identities().len() as u64)?
            ],
        )
        .map_err(sql)?;
        {
            let mut insert = tx
                .prepare_cached("INSERT INTO records VALUES(?1,?2,?3,?4,?5,?6,?7)")
                .map_err(sql)?;
            let mut latest=tx.prepare_cached("INSERT INTO latest VALUES(?1,?2,?3,?4) ON CONFLICT(ns,identity) DO UPDATE SET revision=excluded.revision,ordinal=excluded.ordinal").map_err(sql)?;
            for (index, record) in snapshot.identities().iter().enumerate() {
                let id = codec::encode(record.identity())?;
                let kp = record
                    .identity()
                    .protection_evidence
                    .0
                    .as_ref()
                    .ok_or(PersistenceError::Protection(ProtectionError::Unattested))?;
                let rp = record
                    .protection_evidence
                    .0
                    .as_ref()
                    .ok_or(PersistenceError::Protection(ProtectionError::Unattested))?;
                insert
                    .execute(params![
                        encoded,
                        revision,
                        index as i64,
                        id,
                        codec::record(record)?,
                        kp.as_slice(),
                        rp.as_slice()
                    ])
                    .map_err(sql)?;
                latest
                    .execute(params![encoded, id, revision, index as i64])
                    .map_err(sql)?;
            }
        }
        #[cfg(test)]
        {
            self.adapter.before_commit(&tx)?;
        }
        // Once COMMIT has been invoked, an error is not automatically a rollback.
        // Consumers resolve the immutable submission/revision through the port.
        tx.commit()
            .map_err(|_| PersistenceError::CommitOutcomeUnknown {
                revision: snapshot.revision(),
            })?;
        if !c.is_autocommit() {
            return Err(PersistenceError::CommitOutcomeUnknown {
                revision: snapshot.revision(),
            });
        }
        #[cfg(test)]
        {
            self.adapter.after_commit(snapshot.revision())?;
        }
        Ok(CommitReceipt {
            revision: snapshot.revision(),
        })
    }
    fn abort(self: Box<Self>) -> Result<(), PersistenceError> {
        Ok(())
    }
}
#[cfg(test)]
mod tests;
#[cfg(test)]
use tests::TestFault;
