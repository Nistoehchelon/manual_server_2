# For Windows (from, say, a Linux machine):
```bash
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release
```

# Linux Musl 
I compiled the normal x86_64-unknown-linux-gnu target executable on a normal machine and tried running it on a machine with older glibc version, it gave the error "glibc v... not found". I then tried the x86_64-unknown-linux-musl target. Result: musl seems to run fine on both the normal and older machine. So, I assume this is one possible reason to use the musl target. 
```bash
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

# Android 
Targets:  aarch64-unknown-linux-musl and aarch64-linux-android

aarch64-unknown-linux-musl:
Note: https://musl.cc is just a random website I found which seems popular enough so I would assume it is safe... Do your own research, though.
```bash
cd /workspaces # or wherever you wanna put it
wget https://musl.cc/aarch64-linux-musl-cross.tgz 
tar xzf aarch64-linux-musl-cross.tgz
# temporarily sets some required variables for Rust to use during compilation
export PATH="/workspaces/aarch64-linux-musl-cross/bin:$PATH"
export CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
export AR_aarch64_unknown_linux_musl=aarch64-linux-musl-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
export CXX_aarch64_unknown_linux_musl=aarch64-linux-musl-g++
cd /workspaces/manual_server_2 # or wherever the repo root is
cargo build --release --target aarch64-unknown-linux-musl # voila
```

aarch64-linux-android
Note: Requires setting up the whole Android SDK thingie. So, first, let's do that, then install cargo-ndk for a more comfortable compilation experience and finally run cargo ndk to actually compile the aarch64-linux-android executable.

```bash

```

