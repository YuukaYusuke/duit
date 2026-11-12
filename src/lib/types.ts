export type Student = { id: number; name: string; class_name: string; active: boolean };
export type Income = { id: number; date: string; student_id: number; student_name: string; class_name: string; amount: number; payment_type: string; note: string };
export type Expense = { id: number; date: string; amount: number; note: string };
export type Summary = { total_income: number; total_expense: number; balance: number; recent: Income[]; expenses: Expense[] };
export const classes = ['An Naas', 'Al Ikhlas', 'Al Kautsar', 'An Nahl', 'An-Nisa'];
export const types = ['SPP', 'Pengembangan', 'Buku', 'Foto', 'UIM', 'Infaq', 'Kegiatan'];
