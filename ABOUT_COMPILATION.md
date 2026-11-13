

# For Windows (from, say, a Linux machine):
```bash
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc
sudo apt update && sudo apt install -y clang lld llvm
cargo xwin build --target x86_64-pc-windows-msvc --release
```

# Linux Musl 
I compiled the normal x86_64-unknown-linux-gnu target executable on a normal machine and tried running it on a machine with older glibc version, it gave the error "glibc v... not found". I then tried the x86_64-unknown-linux-musl target. Result: musl seems to run fine on both the normal and older machine. So, I assume this is one possible reason to use the musl target. 
```bash
sudo apt install musl-tools -y
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

# Android 
Targets:  aarch64-unknown-linux-musl and aarch64-linux-android

Note: The Android executables must be started by prepending the SSL_CERT_FILE variable, like this:

```bash
SSL_CERT_FILE=$PREFIX/etc/tls/cert.pem ./executable
```

Otherwise, the first time you go visit, say, the signup page or any page that uses Reqwest to connect to another site (in this case, to Cloudflare Captcha), the whole server will panic with error "No CA certificates were loaded from the system". The SSL_CERT_FILE solves this, though, admittedly, I would much rather prefer to be able to cleanly run the executable like "./executable". 

Possible better solutions: Embedding all CAs in the executable by using the webpki roots thingie, and I think there are other possibilities as well. I don't know if I wanna use webpki_roots since this might cause vulnerabilities if a CA gets their trust revoked. But I assume this is mainly an issue for client apps rather than server apps, since I can just always run the latest version? Eh, I dunno. 


# 1. aarch64-unknown-linux-musl:

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

# 2. aarch64-linux-android

Note: Requires setting up the whole Android SDK thingie. So, first, let's do that, then install cargo-ndk for a more comfortable compilation experience and finally run cargo ndk to actually compile the aarch64-linux-android executable.

Steps 1 and 2. Install Android SDK and cargo-sdk as described in [https://github.com/Nistoehchelon/common_setup/blob/main/ANDROID_SDK_INSTALLATION.md](https://github.com/Nistoehchelon/common_setup/blob/main/ANDROID_SDK_INSTALLATION.md)

Step 3. use cargo-ndk to compile:

```bash
cd /workspaces/manual_server_2 # or whatever the repo root is
cargo ndk build --target=aarch64-linux-android --release # Yay!
```

Cargo stores all the resulting executables in the target folder inside the respective target-named subfolders.
