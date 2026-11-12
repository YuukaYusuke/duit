<div align="center">
  <img src="src-tauri/icons/icon.png" alt="Ikon Bendahara Sekolah" width="88" height="88">
  <h1>Bendahara Sekolah</h1>
  <p><strong>Pembukuan sekolah yang sederhana, rapi, dan tersimpan di perangkat.</strong></p>
  <p>
    <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white">
    <img alt="Svelte 5" src="https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white">
    <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-000000?logo=rust&logoColor=white">
    <img alt="SQLite" src="https://img.shields.io/badge/SQLite-local%20database-003B57?logo=sqlite&logoColor=white">
  </p>
  <p>Aplikasi desktop untuk membantu bendahara sekolah mencatat siswa dan transaksi keuangan tanpa layanan cloud.</p>
</div>

> **Status:** versi pengembangan `0.1.0`. Validasi installer Windows masih mengikuti hasil workflow GitHub Actions.

## ✨ Yang bisa dilakukan

| Bagian | Fitur |
| --- | --- |
| **Siswa** | Tambah, ubah, cari, saring per kelas, nonaktifkan, dan impor dari Excel `.xlsx` dengan pratinjau. |
| **Pemasukan** | Catat pembayaran siswa dengan kategori SPP, Pengembangan, Buku, Foto, UIM, Infaq, atau Kegiatan. |
| **Pengeluaran** | Catat dan ubah pengeluaran sekolah. |
| **Rekap** | Lihat saldo, pilih rentang tanggal, serta kelompokkan pemasukan berdasarkan kelas dan kategori. |
| **Berkas** | Ekspor laporan ke CSV atau PDF dan buat cadangan database SQLite. |

Transaksi menyimpan snapshot nama dan kelas siswa. Perubahan data siswa berikutnya tidak mengubah riwayat transaksi yang sudah dicatat. Impor Excel melewati baris tidak valid dan duplikat; pratinjau dan impor dibatasi hingga 1.000 baris data per file.

## 🧰 Teknologi

- **Desktop:** Tauri 2
- **Backend:** Rust
- **Antarmuka:** Svelte 5 + TypeScript
- **Penyimpanan:** SQLite melalui `rusqlite` dengan SQLite yang dibundel
- **Impor Excel:** `calamine`

## 🚀 Menjalankan untuk pengembangan

Siapkan Node.js, pnpm `12.19.0`, Rust stable, dan dependensi desktop Tauri untuk sistem operasi yang digunakan. Untuk Linux, ikuti [panduan prasyarat resmi Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

Pemeriksaan frontend dan build produksinya:

```sh
pnpm check
pnpm build
```

Pemeriksaan Rust dijalankan dari folder `src-tauri`:

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
```

## 💾 Penyimpanan dan cadangan

Database `bendahara.sqlite` dibuat otomatis di direktori data aplikasi milik pengguna. Lokasinya mengikuti sistem operasi dan tidak berada di folder instalasi. Jangan gunakan database aktif untuk uji coba yang mengubah atau menghapus data.

Gunakan tombol **Simpan cadangan database** untuk membuat salinan SQLite yang konsisten. Simpan cadangan di lokasi aman. Aplikasi belum menyediakan fitur pemulihan dari antarmuka; pemulihan manual perlu dilakukan saat aplikasi tertutup.

## 🪟 Build Windows

Workflow [Windows build](.github/workflows/windows.yml) berjalan saat ada push ke `main` atau dapat dijalankan manual dari tab **Actions** di GitHub. Runner Windows membangun kedua format installer dan memeriksa bahwa keduanya tersedia:

- **NSIS:** file `.exe` di `src-tauri/target/release/bundle/nsis/`
- **MSI:** file `.msi` di `src-tauri/target/release/bundle/msi/`

Setelah workflow sukses, unduh artifact **`bendahara-sekolah-windows-installers`** dari halaman run. Installer menggunakan WebView2 bootstrapper: pada komputer yang belum memiliki WebView2, instalasi runtime memerlukan koneksi internet. Lihat [opsi distribusi WebView2 Tauri](https://v2.tauri.app/distribute/windows-installer/#webview2-installation-options).

Build Linux tidak menghasilkan installer Windows. Keberhasilan workflow dan keberadaan artifact harus diverifikasi sebelum installer dibagikan.

## 🛡️ Data lokal

`.gitignore` mengecualikan database dan backup lokal, file environment/credential, dependensi, skema Tauri yang dihasilkan, serta hasil build. Jangan menambahkan token, kata sandi, atau data keuangan nyata ke repository.
