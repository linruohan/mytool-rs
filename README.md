# mytool gui of rust

study of from https://github.com/flxzt/rnote

## Windows 构建

## 预置条件

- Install [MSYS2](https://www.msys2.org/).
- Install [Rust](https://www.rust-lang.org/).
- OPTIONAL: Install [Inno Setup](https://jrsoftware.org/isinfo.php) for building the installer.

### 环境变量配置

```bash
# path
C:\software\msys2\mingw64\bin
C:\software\msys2\usr\bin
```

### MSYS2 依赖安装

```bash
pacman -S git mingw-w64-x86_64-xz mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-clang \
mingw-w64-x86_64-toolchain mingw-w64-x86_64-autotools mingw-w64-x86_64-make mingw-w64-x86_64-cmake \
mingw-w64-x86_64-meson mingw-w64-x86_64-diffutils mingw-w64-x86_64-desktop-file-utils mingw-w64-x86_64-appstream \
mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-poppler mingw-w64-x86_64-poppler-data \
mingw-w64-x86_64-angleproject
```

### cargo 配置

```bash
# ~/.bashrc
### cargo 配置
export PATH=$PATH:/c/Users/$USER/.cargo/bin
### Inno Setup 安装程序构建程序设置
export PATH=$PATH:/c/Program\ Files\ \(x86\)/Inno\ Setup\ 6
```

#### gnu 安装

```bash
rustup toolchain install stable-gnu
rustup default stable-gnu
```

开启 windows 调试模式，允许创建链接文件
libpthread bug

```bash
mv /mingw64/lib/libpthread.dll.a /mingw64/lib/libpthread.dll.a.bak
```

## 构建应用程序

```bash
# MSYS2路径不是C:\software\msys2，需要特殊指定一下
meson configure -Dmsys-path='C:\path\to\msys64' _mesonbuild
# 初始化
meson setup --prefix=C:/software/msys2/mingw64 _mesonbuild
# 编译
meson compile -C _mesonbuild
# install  `C:\msys64\mingw64\bin\mytool.exe`  非绿色安装包
meson install -C _mesonbuild

# 构建安装程序  `_mesonbuild/mytool-win-installer.exe`.
meson compile mytool-gmo -C _mesonbuild
meson compile build-installer -C _mesonbuild
```
