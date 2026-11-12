use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Student {
    pub id: i64,
    pub name: String,
    pub class_name: String,
    pub active: bool,
}
#[derive(Deserialize)]
pub struct StudentInput {
    pub id: Option<i64>,
    pub name: String,
    pub class_name: String,
}
#[derive(Serialize)]
pub struct Income {
    pub id: i64,
    pub date: String,
    pub student_id: i64,
    pub student_name: String,
    pub class_name: String,
    pub amount: i64,
    pub payment_type: String,
    pub note: String,
}
#[derive(Deserialize)]
pub struct IncomeInput {
    pub date: String,
    pub student_id: i64,
    pub amount: i64,
    pub payment_type: String,
    pub note: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Expense {
    pub id: i64,
    pub date: String,
    pub amount: i64,
    pub note: String,
}
#[derive(Deserialize)]
pub struct ExpenseInput {
    pub id: Option<i64>,
    pub date: String,
    pub amount: i64,
    pub note: String,
}
#[derive(Serialize)]
pub struct Summary {
    pub total_income: i64,
    pub total_expense: i64,
    pub balance: i64,
    pub recent: Vec<Income>,
    pub expenses: Vec<Expense>,
}
#[derive(Serialize)]
pub struct GroupTotal {
    pub class_name: String,
    pub amount: i64,
}
#[derive(Serialize)]
pub struct TypeTotal {
    pub payment_type: String,
    pub amount: i64,
}
#[derive(Serialize)]
pub struct Report {
    pub income: i64,
    pub expense: i64,
    pub balance: i64,
    pub by_class: Vec<GroupTotal>,
    pub by_type: Vec<TypeTotal>,
    pub incomes: Vec<Income>,
    pub expenses: Vec<Expense>,
}
#[derive(Serialize)]
pub struct ImportPreview {
    pub sheets: Vec<String>,
    pub headers: Vec<String>,
    pub rows: Vec<PreviewRow>,
}
#[derive(Serialize)]
pub struct PreviewRow {
    pub name: String,
    pub class_name: String,
    pub valid: bool,
    pub reason: String,
}
#[derive(Serialize)]
pub struct ImportResult {
    pub success: usize,
    pub failed: usize,
}
