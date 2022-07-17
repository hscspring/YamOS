## Lab0-env

本次实验主要是环境搭建，强烈推荐使用 Docker，可以省不少事情。另外需要说明的是，如果使用 Mac，qemu 是[不支持](https://www.qemu.org/docs/master/user/main.html) User Mode 的。

### 基本配置

步骤如下：

- 新建一个文件夹（如 `riscv_rust`）。
- fork 仓库（因为无法使用 classroom），并 clone 到本地。
- **仓库根目录**（`cd rust-based-os-comp2022`）执行：`make build_docker` 构建官方提供的 Docker。
- 切换到**本目录** `lab0` 下再次构建（在官方基础上）`docker build -t riscv/rust:2022 .`，主要增加了 vim，ubuntu 更新源以及 cargo 的配置。
- 在**文件夹根目录**（`riscv_rust`）启动：`docker run --rm -it -v ${PWD}/rust-based-os-com2022:/mnt -w /mnt riscv/rust:2022 bash`。
- 启动后即可进入 Ubuntu 虚拟环境，里面所有需要的都已配置好。

具体可参考：[第零章：实验环境配置 - Open-Source-OS-Training-Camp-2022 文档](https://learningos.github.io/rust-based-os-comp2022/0setup-devel-env.html)。

需要注意的是：构建容器的目录和执行的目录没有太多关系。

### 可选配置

如果不想每次都重新安装一些 rust 的 component，可以将 `/usr/local/cargo/` 和 `/usr/local/rustup/` 目录映射到本地。

步骤如下：

- 启动容器：`docker run --rm -it --name riscv riscv/rust:2022 bash`
- 复制文件夹到本地：在 `riscv_rust` 目录执行 `docker cp riscv:/usr/local/cargo/ .`，`docker cp riscv:/usr/local/rustup/ .`
- 容器启动命令中加入：` -v ${PWD}/cargo:/usr/local/cargo -v ${PWD}/rustup:/usr/local/rustup`

### 常用命令

Docker 启动：

```bash
docker run --rm -it --name riscv \
-v ${PWD}/cargo:/usr/local/cargo  \
-v ${PWD}/rustup:/usr/local/rustup  \
-v ${PWD}/rust-based-os-comp2022:/mnt \
-w /mnt  riscv/rust:2022 bash
```

Rust 支持跨平台编译：

```bash
# 版本确认
rustc --version --verbose
# 默认平台
cargo run
# 换个平台
cargo run --target riscv64gc-unknown-none-elf
# build
cargo build --target riscv64gc-unknown-none-elf
# 添加target/component
rustup target add riscv64gc-unknown-none-elf
rustup component add llvm-tools-preview
```

也可以在项目下配置：

```bash
# project_root/.cargo/config
[build]
target = "riscv64gc-unknown-none-elf"
```

cargo-binutils常用命令：

```bash
# 查看文件格式
file target/riscv64gc-unknown-none-elf/debug/os

# 查看文件头
rust-readobj -h target/riscv64gc-unknown-none-elf/debug/os

# 反汇编导出汇编程序
rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os

# 将ELF执行文件转为bin文件
rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```

更多参考：https://github.com/rust-embedded/cargo-binutils

qemu命令：

```bash
# 执行
qemu-riscv64 target/riscv64gc-unknown-none-elf/debug/os; echo $?

# 模拟启动机器
LOG=TRACE  # 设置日志显示级别
qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios $(BOOTLOADER) \
            -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
```

更多参考：https://www.qemu.org/docs/master/system/index.html

