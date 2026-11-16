# GitHub & Produksi Minesweeper

Panduan satu file untuk semua hal soal GitHub: upload, build otomatis (Windows + Linux), dan rilis.

## 1. Persiapan (sekali saja)

Git dan GitHub CLI sudah terpasang di `D:\Git` dan `D:\GitHubCLI`.

```powershell
gh auth login                      # pilih GitHub.com -> HTTPS -> login lewat browser
git config --global user.name  "NAMA_KAMU"
git config --global user.email "EMAIL_GITHUB_KAMU"
```

## 2. Upload project ke GitHub (pertama kali)

Jalankan di folder project. File `.gitignore` sudah mengecualikan folder `target`.

```powershell
git init -b main
git add .
git commit -m "Minesweeper pertama"
gh repo create minesweeper --public --source . --push   # ganti --public dengan --private jika mau privat
```

## 3. Update berikutnya

```powershell
git add .
git commit -m "Jelaskan perubahan di sini"
git push
```

## 4. Build otomatis (GitHub Actions)

Setiap `git push`, workflow [.github/workflows/build.yml](.github/workflows/build.yml) membuat:

| Artifact | Isi | Untuk |
|---|---|---|
| `minesweeper-windows` | `minesweeper.exe` | Windows |
| `minesweeper-linux` | `minesweeper` | Linux |

Download dari tab **Actions** -> pilih run terbaru -> bagian **Artifacts**.
Di Linux, beri izin jalan dulu: `chmod +x minesweeper && ./minesweeper`.

## 5. Rilis (versi produksi untuk dibagikan)

Beri tag versi, lalu buat release dengan file hasil build:

```powershell
git tag v0.1.0
git push origin v0.1.0
gh release create v0.1.0 --title "Minesweeper v0.1.0" --notes "Rilis pertama"
```

Lalu unggah binary ke release itu (setelah download dari Actions):

```powershell
gh release upload v0.1.0 minesweeper.exe minesweeper
```

Teman tinggal buka halaman **Releases** di repo dan download sesuai OS-nya.

## 6. Build manual di Linux (tanpa GitHub)

```bash
bash build-linux.sh
```

## Catatan

- Jangan commit folder `target` (besar dan tidak perlu); sudah diabaikan lewat `.gitignore`.
- File skor terbaik disimpan lokal di folder tempat game dijalankan, jadi tidak ikut ke repo.
- `.exe` tidak ditandatangani, jadi Windows SmartScreen bisa memberi peringatan: **More info -> Run anyway**.
