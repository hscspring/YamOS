## Lab0-0

### 如果没有标准库

程序的执行需要一些依赖环境，如下图所示：

![](https://learningos.github.io/rust-based-os-comp2022/_images/app-software-stack.png)

如果移除标准库，我们连最基本的「Hello World」都无法打印出来。具体步骤如下：

- 移除 `println!` 宏，并移除该行：开头加上 `#![no_std]`。
- 提供错误处理函数 `panic_handler`：新建一个 `panic` 函数。
- 移除 `main` 函数解决 `start` 语义（标准库执行 `main` 前的初始化工作）：开头加上 `#![no_main]`。

### 入口函数和退出机制

通过反汇编工具（见《常用命令》）反汇编后，没有任何代码，说明是个空程序，少了编译器规定的入口函数 `_start`。增加入口函数后，重新编译后再反汇编，就可以看到相应的汇编代码了。

除了入口函数，还需要一个「退出机制」，否则执行时会报错（把 `_start` 的 `loop` 去掉）。这里使用了汇编和系统调用。

```rust
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id,
        );
    }
    ret
}
```

为了让屏幕显示出更多内容，需要定制 `println`。这一过程主要是实现 `pint` 和 `prinln` 宏。

```rust
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
      $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
```

### 模拟启动RISC-V

首先编译：

```bash
cargo build --release
```

然后执行如下命令：

```bash
qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios $(BOOTLOADER) \
            -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
```

其中：

- `bios` 是一个 BootLoader 程序（此处为 RustSBI），SBI 是 RISC-V 的底层规范，RustSBI 是它的 Rust 实现。SBI 向操作系统提供很少的服务。
- `device` 表示内存中的位置，addr 的值是 `0x80200000`。

执行命令后，意味着给这台虚拟的 RISC-V 加了电。此时：

- PC 指向 `0x1000`，这里有固化在硬件中的一小段引导代码。
- 然后跳转到 `0x80000000`，RustSBI 完成初始化后，会跳转到 addr 处执行操作系统的第一条指令。

`BOOTLOADER` 在 `/bootloader/` 目录下，`KERNEL_BIN` 需要通过 `cargo build --release` 生成，并使用下面的命令转成 `bin` 文件：

```bash
rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```

然后执行：

```bash
qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios ../bootloader/rustsbi-qemu.bin \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

如下图所示：

```bash
[rustsbi] RustSBI version 0.2.2, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.1.1
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
```

新开一个窗口将其杀掉：

```bash
 docker exec -it riscv bash
 ps aux | grep qemu-system | awk '{print $2; }' | xargs kill -9
```

这里陷入了死循环，原因是入口地址不是 RustSBI 约定的 `0x80200000`。

```bash
$ rust-readobj -h target/riscv64gc-unknown-none-elf/debug/os

File: target/riscv64gc-unknown-none-elf/debug/os
Format: elf64-littleriscv
Arch: riscv64
AddressSize: 64bit
LoadName: <Not found>
ElfHeader {
  Ident {
    Magic: (7F 45 4C 46)
    Class: 64-bit (0x2)
    DataEncoding: LittleEndian (0x1)
    FileVersion: 1
    OS/ABI: SystemV (0x0)
    ABIVersion: 0
    Unused: (00 00 00 00 00 00 00)
  }
  Type: Executable (0x2)
  Machine: EM_RISCV (0xF3)
  Version: 1
  Entry: 0x11E08
  ProgramHeaderOffset: 0x40
  SectionHeaderOffset: 0xE83D8
  Flags [ (0x5)
    EF_RISCV_FLOAT_ABI_DOUBLE (0x4)
    EF_RISCV_RVC (0x1)
  ]
  HeaderSize: 64
  ProgramHeaderEntrySize: 56
  ProgramHeaderCount: 5
  SectionHeaderEntrySize: 64
  SectionHeaderCount: 20
  StringTableSectionIndex: 18
}
```

### 内存和栈空间布局

- 内存空间布局可以通过「链接脚本」调整链接器行为，链接脚本：`src/linker.ld`。
- 栈空间通过 `entry.asm` 配置。
- 最后注意清零 `.bss` 段。

重新执行上面的命令后，程序可以正常退出。

注意事项：

- `console.rs` 中的 `write_str` 需要使用 `console_putchar`，而不是 `sys_write`。
- 重写 `panic`，注意 `shutdown` 的实现。

本节使用命令汇总：

```bash
$ cargo build --release
$ rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
$ LOG=TRACE
$ qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios ../bootloader/rustsbi-qemu.bin \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

### 实验报告

经过以上实践，结合教程[引导](https://learningos.github.io/rust-based-os-comp2022/chapter1/0intro.html#)，运行本章代码：

```bash
cd os1
make run LOG=TRACE
```

等待一会儿后显示：

```bash
[rustsbi] RustSBI version 0.2.2, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.1.1
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
Hello, world!
[TRACE] .text [0x80200000, 0x80203000)
[DEBUG] .rodata [0x80203000, 0x80205000)
[ INFO] .data [0x80205000, 0x80206000)
[ WARN] boot_stack [0x80206000, 0x80216000)
[ERROR] .bss [0x80216000, 0x80217000)
Panicked at src/main.rs:48 Shutdown machine!
```

退回根目录，执行测试：

```
cd ..
make test1
```

结果显示（本章可直接执行）：

```bash
[rustsbi] RustSBI version 0.2.2, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.1.1
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
Hello, world!
Panicked at src/main.rs:48 Shutdown machine!
make[2]: Leaving directory '/mnt/workplace/os'
python3 check/ch1.py < stdout-ch1
[PASS] found <Hello, world!>
[PASS] not found <FAIL: T.T>

Test passed1719: 2/2
make[1]: Leaving directory '/mnt/workplace/ci-user'
```

### 参考和说明

- 教程、图片和代码均来自：https://learningos.github.io/rust-based-os-comp2022/