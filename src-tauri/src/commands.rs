use crate::{
    config::{Db, CLASSES, TYPES},
    models::{
        Expense, ExpenseInput, GroupTotal, ImportPreview, ImportResult, Income, IncomeInput,
        PreviewRow, Report, Student, StudentInput, Summary, TypeTotal,
    },
    validation::{valid_date, validate_amount},
};
use calamine::{open_workbook, Data, Reader, Xlsx};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs::File;
use tauri::State;

const MAX_STUDENT_IMPORT_ROWS: usize = 1000;

fn conn<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    db.0.lock()
        .map_err(|_| "Database sedang digunakan. Coba lagi.".to_string())
}
fn income_rows(
    c: &Connection,
    filter: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<Income>, String> {
    let sql=format!("SELECT id,date,student_id,student_name,class_name,amount,payment_type,note FROM incomes WHERE 1=1 {filter} AND (?1 IS NULL OR date>=?1) AND (?2 IS NULL OR date<=?2) ORDER BY date DESC,id DESC");
    let mut q = c.prepare(&sql).map_err(err)?;
    let rows = q
        .query_map(params![from, to], |r| {
            Ok(Income {
                id: r.get(0)?,
                date: r.get(1)?,
                student_id: r.get(2)?,
                student_name: r.get(3)?,
                class_name: r.get(4)?,
                amount: r.get(5)?,
                payment_type: r.get(6)?,
                note: r.get(7)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}
fn expense_rows(
    c: &Connection,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<Expense>, String> {
    let mut q=c.prepare("SELECT id,date,amount,note FROM expenses WHERE (?1 IS NULL OR date>=?1) AND (?2 IS NULL OR date<=?2) ORDER BY date DESC,id DESC").map_err(err)?;
    let rows = q
        .query_map(params![from, to], |r| {
            Ok(Expense {
                id: r.get(0)?,
                date: r.get(1)?,
                amount: r.get(2)?,
                note: r.get(3)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}
fn err(e: rusqlite::Error) -> String {
    format!("Kesalahan database: {e}")
}
#[tauri::command]
pub fn list_students(db: State<Db>) -> Result<Vec<Student>, String> {
    let c = conn(&db)?;
    let mut q=c.prepare("SELECT s.id,s.name,c.name,s.active FROM students s JOIN classes c ON c.id=s.class_id ORDER BY s.name COLLATE NOCASE").map_err(err)?;
    let rows = q
        .query_map([], |r| {
            Ok(Student {
                id: r.get(0)?,
                name: r.get(1)?,
                class_name: r.get(2)?,
                active: r.get::<_, i64>(3)? == 1,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}
#[tauri::command]
pub fn add_student(db: State<Db>, student: StudentInput) -> Result<i64, String> {
    let name = student.name.trim();
    if name.is_empty() {
        return Err("Nama siswa wajib diisi.".into());
    }
    let c = conn(&db)?;
    let id: Option<i64> = c
        .query_row(
            "SELECT id FROM classes WHERE name=?1",
            [student.class_name],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?;
    let cid = id.ok_or("Kelas tidak dikenal.")?;
    c.execute(
        "INSERT INTO students(name,class_id) VALUES(?1,?2)",
        params![name, cid],
    )
    .map_err(err)?;
    Ok(c.last_insert_rowid())
}
#[tauri::command]
pub fn update_student(db: State<Db>, student: StudentInput) -> Result<(), String> {
    let id = student.id.ok_or("ID siswa tidak valid.")?;
    let name = student.name.trim();
    if name.is_empty() {
        return Err("Nama siswa wajib diisi.".into());
    }
    let c = conn(&db)?;
    let cid: i64 = c
        .query_row(
            "SELECT id FROM classes WHERE name=?1",
            [student.class_name],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?
        .ok_or("Kelas tidak dikenal.")?;
    c.execute(
        "UPDATE students SET name=?1,class_id=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3",
        params![name, cid, id],
    )
    .map_err(err)?;
    Ok(())
}
#[tauri::command]
pub fn deactivate_student(db: State<Db>, id: i64) -> Result<(), String> {
    conn(&db)?
        .execute(
            "UPDATE students SET active=0,updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            [id],
        )
        .map_err(err)?;
    Ok(())
}
#[tauri::command]
pub fn list_incomes(db: State<Db>) -> Result<Vec<Income>, String> {
    let c = conn(&db)?;
    income_rows(&c, "", None, None)
}
#[tauri::command]
pub fn add_income(db: State<Db>, input: IncomeInput) -> Result<i64, String> {
    valid_date(&input.date)?;
    validate_amount(input.amount)?;
    let mut c = conn(&db)?;
    let tx = c.transaction().map_err(err)?;
    let (name,class):(String,String)=tx.query_row("SELECT s.name,c.name FROM students s JOIN classes c ON c.id=s.class_id WHERE s.id=?1 AND s.active=1",[input.student_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err)?.ok_or("Siswa tidak ditemukan atau sudah nonaktif.")?;
    if !TYPES.contains(&input.payment_type.as_str()) {
        return Err("Jenis pembayaran tidak dikenal.".into());
    }
    let type_id: i64 = tx
        .query_row(
            "SELECT id FROM payment_types WHERE name=?1",
            [&input.payment_type],
            |r| r.get(0),
        )
        .map_err(err)?;
    tx.execute("INSERT INTO incomes(date,student_id,student_name,class_name,amount,payment_type_id,payment_type,note) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![input.date,input.student_id,name,class,input.amount,type_id,input.payment_type,input.note.unwrap_or_default().trim()]).map_err(err)?;
    let id = tx.last_insert_rowid();
    tx.commit().map_err(err)?;
    Ok(id)
}
#[tauri::command]
pub fn list_expenses(db: State<Db>) -> Result<Vec<Expense>, String> {
    let c = conn(&db)?;
    expense_rows(&c, None, None)
}
#[tauri::command]
pub fn add_expense(db: State<Db>, input: ExpenseInput) -> Result<i64, String> {
    valid_date(&input.date)?;
    validate_amount(input.amount)?;
    if input.note.trim().is_empty() {
        return Err("Keterangan wajib diisi.".into());
    }
    let c = conn(&db)?;
    c.execute(
        "INSERT INTO expenses(date,amount,note) VALUES(?1,?2,?3)",
        params![input.date, input.amount, input.note.trim()],
    )
    .map_err(err)?;
    Ok(c.last_insert_rowid())
}
#[tauri::command]
pub fn update_expense(db: State<Db>, input: ExpenseInput) -> Result<(), String> {
    valid_date(&input.date)?;
    validate_amount(input.amount)?;
    if input.note.trim().is_empty() {
        return Err("Keterangan wajib diisi.".into());
    }
    let id = input.id.ok_or("ID pengeluaran tidak valid.")?;
    conn(&db)?.execute("UPDATE expenses SET date=?1,amount=?2,note=?3,updated_at=CURRENT_TIMESTAMP WHERE id=?4",params![input.date,input.amount,input.note.trim(),id]).map_err(err)?;
    Ok(())
}
#[tauri::command]
pub fn dashboard(db: State<Db>) -> Result<Summary, String> {
    let c = conn(&db)?;
    let inc: i64 = c
        .query_row("SELECT COALESCE(SUM(amount),0) FROM incomes", [], |r| {
            r.get(0)
        })
        .map_err(err)?;
    let exp: i64 = c
        .query_row("SELECT COALESCE(SUM(amount),0) FROM expenses", [], |r| {
            r.get(0)
        })
        .map_err(err)?;
    let mut incomes = income_rows(&c, "", None, None)?;
    incomes.truncate(6);
    let mut expenses = expense_rows(&c, None, None)?;
    expenses.truncate(6);
    Ok(Summary {
        total_income: inc,
        total_expense: exp,
        balance: inc - exp,
        recent: incomes,
        expenses,
    })
}
#[tauri::command]
pub fn report(
    db: State<Db>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<Report, String> {
    if let Some(ref d) = from_date {
        valid_date(d)?
    }
    if let Some(ref d) = to_date {
        valid_date(d)?
    }
    if let (Some(from), Some(to)) = (&from_date, &to_date) {
        if from > to {
            return Err("Rentang tanggal tidak valid: tanggal awal melebihi tanggal akhir.".into());
        }
    }
    let c = conn(&db)?;
    let incomes = income_rows(&c, "", from_date.as_deref(), to_date.as_deref())?;
    let expenses = expense_rows(&c, from_date.as_deref(), to_date.as_deref())?;
    let income = incomes.iter().map(|x| x.amount).sum();
    let expense = expenses.iter().map(|x| x.amount).sum();
    let mut by_class = Vec::<GroupTotal>::new();
    let mut by_type = Vec::<TypeTotal>::new();
    for i in &incomes {
        if let Some(v) = by_class.iter_mut().find(|v| v.class_name == i.class_name) {
            v.amount += i.amount
        } else {
            by_class.push(GroupTotal {
                class_name: i.class_name.clone(),
                amount: i.amount,
            })
        }
        if let Some(v) = by_type
            .iter_mut()
            .find(|v| v.payment_type == i.payment_type)
        {
            v.amount += i.amount
        } else {
            by_type.push(TypeTotal {
                payment_type: i.payment_type.clone(),
                amount: i.amount,
            })
        }
    }
    Ok(Report {
        income,
        expense,
        balance: income - expense,
        by_class,
        by_type,
        incomes,
        expenses,
    })
}
fn excel_rows(path: &str, sheet: Option<&str>) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut wb: Xlsx<_> =
        open_workbook(path).map_err(|e| format!("File Excel tidak dapat dibuka: {e}"))?;
    let sheets = wb.sheet_names().to_vec();
    let chosen = sheet
        .filter(|s| sheets.iter().any(|v| v == *s))
        .or_else(|| sheets.first().map(String::as_str))
        .ok_or("File tidak memiliki sheet.")?
        .to_string();
    let range = wb
        .worksheet_range(&chosen)
        .map_err(|e| format!("Sheet tidak dapat dibaca: {e}"))?;
    let rows = range
        .rows()
        .map(|r| {
            r.iter()
                .map(|c| match c {
                    Data::String(s) => s.trim().to_string(),
                    Data::Int(n) => n.to_string(),
                    Data::Float(n) => n.to_string(),
                    Data::Bool(b) => b.to_string(),
                    _ => String::new(),
                })
                .collect()
        })
        .collect();
    Ok((sheets, rows))
}
#[tauri::command]
pub fn preview_student_import(
    path: String,
    sheet: Option<String>,
    name_column: Option<usize>,
    class_column: Option<usize>,
) -> Result<ImportPreview, String> {
    let (sheets, rows) = excel_rows(&path, sheet.as_deref())?;
    let headers = rows.first().cloned().unwrap_or_default();
    let guess = |needle: &str| {
        headers
            .iter()
            .position(|h| h.to_lowercase().contains(needle))
    };
    let ni = name_column.or_else(|| guess("nama")).unwrap_or(0);
    let ci = class_column.or_else(|| guess("kelas")).unwrap_or(1);
    let preview = rows
        .into_iter()
        .skip(1)
        .filter(|r| r.iter().any(|v| !v.trim().is_empty()))
        .take(MAX_STUDENT_IMPORT_ROWS)
        .map(|r| {
            let name = r.get(ni).cloned().unwrap_or_default();
            let class_name = r.get(ci).cloned().unwrap_or_default();
            let valid = !name.trim().is_empty() && CLASSES.contains(&class_name.as_str());
            let reason = if name.trim().is_empty() {
                "Nama kosong"
            } else if !CLASSES.contains(&class_name.as_str()) {
                "Kelas tidak dikenal"
            } else {
                "Siap"
            }
            .to_string();
            PreviewRow {
                name,
                class_name,
                valid,
                reason,
            }
        })
        .collect();
    Ok(ImportPreview {
        sheets,
        headers,
        rows: preview,
    })
}
#[tauri::command]
pub fn import_students(
    db: State<Db>,
    path: String,
    sheet: String,
    name_column: usize,
    class_column: usize,
) -> Result<ImportResult, String> {
    let (_, rows) = excel_rows(&path, Some(&sheet))?;
    let import_rows = rows
        .into_iter()
        .skip(1)
        .filter(|row| row.iter().any(|value| !value.trim().is_empty()))
        .collect::<Vec<_>>();
    if import_rows.len() > MAX_STUDENT_IMPORT_ROWS {
        return Err(format!(
            "File berisi lebih dari {MAX_STUDENT_IMPORT_ROWS} baris data. Pecah file agar semua baris dapat diperiksa sebelum diimpor."
        ));
    }
    let mut c = conn(&db)?;
    let tx = c.transaction().map_err(err)?;
    let mut success = 0;
    let mut failed = 0;
    for row in import_rows {
        let name = row.get(name_column).map(|s| s.trim()).unwrap_or("");
        let class = row.get(class_column).map(|s| s.trim()).unwrap_or("");
        if name.is_empty() || !CLASSES.contains(&class) {
            failed += 1;
            continue;
        }
        let dup:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM students WHERE lower(name)=lower(?1) AND class_id=(SELECT id FROM classes WHERE name=?2))",params![name,class],|r|r.get(0)).map_err(err)?;
        if dup {
            failed += 1;
            continue;
        }
        tx.execute(
            "INSERT INTO students(name,class_id) VALUES(?1,(SELECT id FROM classes WHERE name=?2))",
            params![name, class],
        )
        .map_err(err)?;
        success += 1
    }
    tx.commit().map_err(err)?;
    Ok(ImportResult { success, failed })
}
fn make_report(db: &State<Db>, from: Option<String>, to: Option<String>) -> Result<Report, String> {
    report(db.clone(), from, to)
}
#[tauri::command]
pub fn export_csv(
    db: State<Db>,
    path: String,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<(), String> {
    let r = make_report(&db, from_date, to_date)?;
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut w = csv::WriterBuilder::new().from_writer(file);
    w.write_record([
        "Tanggal",
        "Jenis",
        "Siswa/Keterangan",
        "Kelas",
        "Kategori",
        "Nominal (Rp)",
    ])
    .map_err(|e| e.to_string())?;
    for i in r.incomes {
        w.write_record([
            i.date,
            "Pemasukan".into(),
            i.student_name,
            i.class_name,
            i.payment_type,
            i.amount.to_string(),
        ])
        .map_err(|e| e.to_string())?
    }
    for e in r.expenses {
        w.write_record([
            e.date,
            "Pengeluaran".into(),
            e.note,
            String::new(),
            String::new(),
            e.amount.to_string(),
        ])
        .map_err(|e| e.to_string())?
    }
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}
fn pdf_document(lines: &[String]) -> Vec<u8> {
    const PAGE_LINES: usize = 55;
    const LINE_WIDTH: usize = 92;
    let mut safe_lines = Vec::new();
    for line in lines {
        let safe: String = line
            .chars()
            .map(|ch| {
                if ch.is_ascii() {
                    ch
                } else if ch.is_whitespace() {
                    ' '
                } else {
                    '?'
                }
            })
            .collect();
        let mut current = String::new();
        for ch in safe.chars() {
            current.push(ch);
            if current.len() >= LINE_WIDTH {
                safe_lines.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() || line.is_empty() {
            safe_lines.push(current);
        }
    }
    if safe_lines.is_empty() {
        safe_lines.push(String::new());
    }
    let pages: Vec<&[String]> = safe_lines.chunks(PAGE_LINES).collect();
    let font_id = 3 + pages.len() * 2;
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize; font_id + 1];
    let write_object = |id: usize, body: &[u8], output: &mut Vec<u8>, offsets: &mut Vec<usize>| {
        offsets[id] = output.len();
        output.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        output.extend_from_slice(body);
        output.extend_from_slice(b"\nendobj\n");
    };

    write_object(
        1,
        b"<< /Type /Catalog /Pages 2 0 R >>",
        &mut pdf,
        &mut offsets,
    );
    let kids = pages
        .iter()
        .enumerate()
        .map(|(index, _)| format!("{} 0 R", 3 + index * 2))
        .collect::<Vec<_>>()
        .join(" ");
    let pages_object = format!("<< /Type /Pages /Kids [{kids}] /Count {} >>", pages.len());
    write_object(2, pages_object.as_bytes(), &mut pdf, &mut offsets);

    for (index, page_lines) in pages.iter().enumerate() {
        let page_id = 3 + index * 2;
        let content_id = page_id + 1;
        let page_object = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
        );
        write_object(page_id, page_object.as_bytes(), &mut pdf, &mut offsets);

        let mut stream = String::from("BT /F1 9 Tf 40 800 Td 13 TL\n");
        for line in *page_lines {
            let escaped = line
                .replace('\\', "\\\\")
                .replace('(', "\\(")
                .replace(')', "\\)");
            stream.push_str(&format!("({escaped}) Tj T*\n"));
        }
        stream.push_str("ET\n");
        let content_object = format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        );
        write_object(
            content_id,
            content_object.as_bytes(),
            &mut pdf,
            &mut offsets,
        );
    }
    write_object(
        font_id,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        &mut pdf,
        &mut offsets,
    );

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

#[cfg(test)]
mod tests {
    use super::{income_rows, pdf_document};
    use rusqlite::Connection;

    #[test]
    fn income_query_accepts_unbounded_date_filters() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        let result = income_rows(&db, "", None, None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pdf_writer_paginates_escapes_text_and_emits_valid_xref_offset() {
        let lines = (0..120)
            .map(|i| format!("Baris {i} (uji) \\ nama é"))
            .collect::<Vec<_>>();
        let pdf = pdf_document(&lines);
        let text = String::from_utf8(pdf.clone()).unwrap();
        assert_eq!(text.matches("/Type /Page ").count(), 3);
        assert!(text.contains("\\(uji\\) \\\\ nama ?"));
        let offset = text
            .rsplit_once("startxref\n")
            .unwrap()
            .1
            .split_once('\n')
            .unwrap()
            .0
            .parse::<usize>()
            .unwrap();
        assert_eq!(&pdf[offset..offset + 5], b"xref\n");
    }
}

#[tauri::command]
pub fn export_pdf(
    db: State<Db>,
    path: String,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<(), String> {
    let r = make_report(&db, from_date.clone(), to_date.clone())?;
    let mut lines = vec![
        "LAPORAN KEUANGAN SEKOLAH".to_string(),
        format!(
            "Periode: {} s.d. {}",
            from_date.as_deref().unwrap_or("awal data"),
            to_date.as_deref().unwrap_or("hari ini")
        ),
        format!("Total pemasukan: Rp {}", r.income),
        format!("Total pengeluaran: Rp {}", r.expense),
        format!("Saldo periode: Rp {}", r.balance),
        String::new(),
        "Tanggal | Jenis | Nama/Keterangan | Kelas | Kategori | Nominal".into(),
    ];
    for i in r.incomes {
        lines.push(format!(
            "{} | Pemasukan | {} | {} | {} | Rp {}",
            i.date, i.student_name, i.class_name, i.payment_type, i.amount
        ));
    }
    for e in r.expenses {
        lines.push(format!(
            "{} | Pengeluaran | {} | | | Rp {}",
            e.date, e.note, e.amount
        ));
    }
    std::fs::write(path, pdf_document(&lines)).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn backup_database(db: State<Db>, path: String) -> Result<(), String> {
    let c = conn(&db)?;
    let mut dest = Connection::open(path).map_err(err)?;
    let backup = rusqlite::backup::Backup::new(&c, &mut dest).map_err(err)?;
    backup
        .run_to_completion(100, std::time::Duration::from_millis(20), None)
        .map_err(err)?;
    Ok(())
}
