use rusqlite::{params, Connection};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestFiles(Vec<PathBuf>);
impl TestFiles {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn path(&mut self, label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "bendahara-{label}-{}-{stamp}.sqlite",
            std::process::id()
        ));
        self.0.push(path.clone());
        path
    }
}
impl Drop for TestFiles {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = fs::remove_file(path);
            let _ = fs::remove_file(path.with_extension("sqlite-wal"));
            let _ = fs::remove_file(path.with_extension("sqlite-shm"));
        }
    }
}

fn initialized(path: &PathBuf) -> Connection {
    let db = Connection::open(path).unwrap();
    db.pragma_update(None, "foreign_keys", "ON").unwrap();
    db.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    db
}

#[test]
fn sqlite_constraints_snapshots_date_totals_persistence_and_backup() {
    let mut files = TestFiles::new();
    let active_path = files.path("active");
    let backup_path = files.path("backup");
    {
        let db = initialized(&active_path);
        let class_id: i64 = db
            .query_row("SELECT id FROM classes WHERE name='An Naas'", [], |r| {
                r.get(0)
            })
            .unwrap();
        let payment_id: i64 = db
            .query_row("SELECT id FROM payment_types WHERE name='SPP'", [], |r| {
                r.get(0)
            })
            .unwrap();
        db.execute(
            "INSERT INTO students(name,class_id) VALUES('Siswa Uji',?1)",
            [class_id],
        )
        .unwrap();
        let student_id = db.last_insert_rowid();
        db.execute("INSERT INTO incomes(date,student_id,student_name,class_name,amount,payment_type_id,payment_type,note) VALUES('2026-12-31',?1,'Siswa Uji','An Naas',125000,?2,'SPP','')", params![student_id,payment_id]).unwrap();
        db.execute("INSERT INTO incomes(date,student_id,student_name,class_name,amount,payment_type_id,payment_type,note) VALUES('2027-01-01',?1,'Siswa Uji','An Naas',50000,?2,'SPP','')", params![student_id,payment_id]).unwrap();
        db.execute(
            "INSERT INTO expenses(date,amount,note) VALUES('2027-01-01',25000,'Alat tulis')",
            [],
        )
        .unwrap();

        assert!(db
            .execute(
                "INSERT INTO expenses(date,amount,note) VALUES('2027-01-02',0,'Nominal nol')",
                []
            )
            .is_err());
        assert!(db.execute("INSERT INTO incomes(date,student_id,student_name,class_name,amount,payment_type_id,payment_type,note) VALUES('2027-01-02',999,'Tidak ada','An Naas',1000,?1,'SPP','')", [payment_id]).is_err());

        let al_ikhlas: i64 = db
            .query_row("SELECT id FROM classes WHERE name='Al Ikhlas'", [], |r| {
                r.get(0)
            })
            .unwrap();
        db.execute(
            "UPDATE students SET class_id=?1 WHERE id=?2",
            params![al_ikhlas, student_id],
        )
        .unwrap();
        let historical_class: String = db
            .query_row(
                "SELECT class_name FROM incomes WHERE student_id=?1 ORDER BY id LIMIT 1",
                [student_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(historical_class, "An Naas");

        let (income, expense): (i64, i64) = db.query_row(
            "SELECT (SELECT COALESCE(SUM(amount),0) FROM incomes WHERE date>=?1 AND date<=?2), (SELECT COALESCE(SUM(amount),0) FROM expenses WHERE date>=?1 AND date<=?2)",
            params!["2027-01-01", "2027-01-01"], |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(
            (income, expense, income - expense),
            (50_000, 25_000, 25_000)
        );
        let yearly_total: i64 = db
            .query_row(
                "SELECT COALESCE(SUM(amount),0) FROM incomes WHERE date>=?1 AND date<=?2",
                params!["2026-01-01", "2026-12-31"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(yearly_total, 125_000);

        let mut backup = Connection::open(&backup_path).unwrap();
        rusqlite::backup::Backup::new(&db, &mut backup)
            .unwrap()
            .run_to_completion(10, std::time::Duration::from_millis(5), None)
            .unwrap();
    }

    // Reopen the active database, then open the backup independently as a restore target.
    let reopened = Connection::open(&active_path).unwrap();
    let active_count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM incomes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(active_count, 2);
    let restored = Connection::open(&backup_path).unwrap();
    let restored_balance: i64 = restored.query_row(
        "SELECT (SELECT COALESCE(SUM(amount),0) FROM incomes) - (SELECT COALESCE(SUM(amount),0) FROM expenses)", [], |r| r.get(0),
    ).unwrap();
    assert_eq!(restored_balance, 150_000);
}
