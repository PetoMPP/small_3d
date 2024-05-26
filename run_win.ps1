docker run --rm -v "${PWD}:/app" -t small_3d_win cargo build --target=x86_64-pc-windows-msvc
& ".\target\x86_64-pc-windows-msvc\release\small_3d.exe"
