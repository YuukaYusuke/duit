# Bendahara Sekolah

Aplikasi desktop offline untuk mencatat data siswa, pemasukan, pengeluaran, rekap, ekspor, dan backup. Data disimpan lokal dalam SQLite; nominal Rupiah menggunakan bilangan bulat dan aturan keuangan dijalankan di Rust.

## Pengembangan di Arch Linux

Pasang Node.js, pnpm, Rust (stable), serta paket desktop Tauri untuk WebKitGTK/GTK, librsvg, patchelf, dan build tools dari repositori Arch. Nama paket mengikuti versi Arch yang digunakan.

```sh
pnpm install
pnpm tauri dev
```

Pemeriksaan antarmuka: `pnpm check`. Build frontend: `pnpm build`. Jalankan pemeriksaan Rust dari `src-tauri` dengan `cargo check` setelah dependency Rust tersedia.

Aplikasi menyimpan database di direktori data aplikasi Tauri milik pengguna, dengan nama `bendahara.sqlite`. Database baru dibuat otomatis saat pertama kali dijalankan. Jangan menaruh data dummy pada database pengguna.

## Fitur yang tersedia

- Ringkasan saldo dari seluruh transaksi tersimpan.
- CRUD siswa, filter kelas/nama, dan penonaktifan tanpa menghapus riwayat.
- Impor XLSX dengan pemilihan sheet/kolom dan pratinjau sebelum simpan; duplikat dilewati dan dilaporkan sebagai gagal.
- Pencatatan pemasukan memakai daftar kategori tetap: SPP, Pengembangan, Buku, Foto, UIM, Infaq, Kegiatan.
- Pencatatan dan perubahan pengeluaran.
- Rekap rentang tanggal, ekspor CSV/PDF, dan backup SQLite.

## Backup dan akses database

Gunakan tombol **Simpan cadangan database** di aplikasi agar SQLite membuat salinan konsisten. Backup dapat disimpan di lokasi pilihan pengguna. Database aktif ada di folder data aplikasi Tauri untuk sistem operasi tersebut; lokasi tepatnya berbeda antara Linux dan Windows. Hindari menyalin file database aktif secara manual saat aplikasi sedang berjalan.

## Build Windows

Build Windows dilakukan di Windows, bukan cross-compile dari Linux:

1. Pasang Node.js, pnpm, dan Rust stable.
2. Clone repository, lalu jalankan `pnpm install`.
3. Jalankan `pnpm tauri build`.
4. Installer NSIS/MSI tersedia di `src-tauri/target/release/bundle/`.

Workflow `.github/workflows/windows.yml` membangun installer pada Windows runner setiap push ke branch utama atau saat dijalankan manual. Hasil build diunggah sebagai artifact. Build Linux tidak membuktikan build Windows berhasil.

## Catatan status dan hal yang perlu dilanjutkan

Ini masih versi pengembangan, bukan rilis final. Modul transport dan tipe frontend sudah dipisahkan, tetapi halaman fitur masih terkumpul di `src/App.svelte`; pemecahan menjadi komponen per halaman belum selesai. Daftar transaksi juga masih dimuat sekaligus, sehingga pagination perlu ditambahkan sebelum pemakaian pada data besar. Ekspor PDF saat ini berupa laporan teks sederhana dan perlu diuji pada laporan panjang.

Perlu uji pengguna untuk CRUD, impor dengan variasi file nyata, pemulihan dari backup, ekspor CSV/PDF, serta validasi installer Windows sebelum distribusi. Dependency belum dikunci dalam `pnpm-lock.yaml`. Pemeriksaan build di lingkungan ini tertunda: dependency Rust `calamine` tidak tersedia di cache offline dan akses crates.io gagal.
