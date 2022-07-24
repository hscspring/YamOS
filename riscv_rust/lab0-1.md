## Lab0-1

保护操作系统不受出错程序破坏的机制被称为**特权级** (Privilege) 机制， 它实现了用户态和内核态的隔离。

关于 RISC-V 的备注说明（寄存器<=>ABI 接口名称）：

- a0-a7 对应 x10-x17
- t0-t2 对应 x5-x7
- x1 对应 ra
- x2 对应 sp

另外，根据官方手册描述：

> sscratch 寄存器是一个 SXLEN 位读/写寄存器，专用于监控器使用。通常，sscratch 用于在 hart 执行用户代码时保存指向 hart-local supervisor context 的指针。在 Trap 处理程序开始时，sscratch 与用户寄存器交换以提供初始工作寄存器。

### 应用程序

主要包括：入口函数、内存布局和系统调用。应用程序由于处于「用户级」，因此这里单独构建了项目。

- 入口函数定义在 `lib.rs`，这里需要使用 `link_section` 指定用户程序入口。
- 内存布局需要将入口函数所在的段放在开头 `0x80400000`，即批处理系统在加载应用后，跳转到该地址进入入口函数。
- 系统调用：由于程序在用户态，`ecall` 会触发 `Environment call from U-mode` 异常，并 Trap 进入 S 模式执行批处理系统针对这个异常特别提供的服务程序。这里直接使用 ABI 接口。

最后执行编译并使用 `objcopy` 工具删除 ELF header 和符号，得到 `bin` 文件。它将被链接进内核，并在合适的时机加载到内存。

### 批处理操作系统

使用批处理系统将之前的应用程序加载到内核。

- 在 `main` 函数中 `include`：`core::arch::global_asm!(include_str!("link_app.S"));`。
- 找到 APP，初始化一个 MANAGER。
- 加载：
    - 将参数 `app_id` 对应的二进制文件加载到 OS 与应用程序约定的常数地址：`0x80400000`。
    - 清理指令缓存（i-cache）。
    - 清空一块内存并复制二进制文件。

### 特权级切换

某个应用程序执行完或出错时，需要切换到下一个程序，这就涉及到 S/U 级的切换。

**RISC-V 特权级切换**

OS 为应用程序准备执行环境，包括：

- 启动时需初始化用户态上下文，并切换到用户态执行程序。
- 程序发起系统调用后能切换到系统进行处理。
- 程序出错时系统应杀死该程序并切换到下一个。
- 程序执行结束时程序需加载运行下一个。

仅考虑 U 到 S，进入 S Trap 的相关 CSR：

- `sstatus`：Trap 发生前 CPU 的特权级信息。
- `sepc`：Trap 是异常时，记录之前执行的最后一条指令。
- `scause`：Trap 原因。
- `stval`：Trap 附加信息。
- `stvec`：控制 Trap 处理代码的入口地址。

CPU 执行完 Trap 返回时执行 S 级特权指令 `sret`，功能包括：

- CPU 将 `sstatus` 的 `SPP` 设置为 U 或 S。
- 跳转到 `sepc` 指向的指令继续执行。

**用户栈和内核栈**

在从 U 进入到 S 时需使用「内核栈」保存原控制流的寄存器状态。

- 定义用户栈和内核栈，换栈时将 `sp` 的寄存器值改成 `get_sp` 的返回值即可。
- 定义 Trap 上下文。
- 保存寄存器（`x0` 和 `tp(x4)` 除外，但依然预留空间）。
- 保存 CSR（控制状态寄存器）：
    - `scause/stval` 在 Trap 处理的第一时间被调用或在其他地方保存，没有被修改。
    - `sstatus/sepc` 在 Trap 全程有意义，Trap 嵌套时会被覆盖，因此要保存，并在 `sret` 之前恢复。

**Trap 管理**

Trap（陷入）上下文保存和恢复：

- 修改 `stvec` 寄存器指向正确的 Trap 处理入口。
- 通过 `__alltraps` 将 Trap 上下文保存在内核栈上。
    - `.align 2` 对齐后续代码。
    - `csrrw` 交换 sscratch 和 sp；交换前，sp 指向用户栈，sscratch 指向内核栈；交换后，sp 指向内核栈，sscratch 指向用户栈。
    - 预分配栈帧，用于在内核栈上保存 Trap 上下文。
    - 保存除 x0，x4 和 x2 之外的其他通用寄存器。
    - 将 `sstatus` 和 `spec` 读到 t0 和 t1，然后保存到内核栈对应位置。
    - 将 sscratch 读到 t2 并保存到内核栈。sp 指向内核栈。
    - 让寄存器 a0 指向内核栈的栈指针（刚刚保存的 Trap 上下文地址）用于接下来的函数调用（它的第一个参数要从 a0 获取）。
- 跳转到 `trap_handler` 完成 Trap 分发及处理。
    - a0 寄存器在处理完后原样返回，依然指向分配 Trap 上下文之后的内核栈顶。
    - 使用 Rust 三方库 riscv 处理 CSR。
    - 修改内核栈 Trap 上下文里面的 `spec`，让其增加 4（`ecall` 指令码长度），即跳到下一条指令。`sret` 之后会从这里执行。a0 也会发生变化。
    - 处理错误和非法指令，并调用 `run_next_app` 切换到下一个程序。不支持的 Trap 类型则直接 panic 退出。
- `trap_handler` 返回后使用 `__restore` 从保存在内核栈上的 Trap 上下文恢复寄存器。
    - 从内核栈栈顶的 Trap 上下文恢复通用寄存器和 CSR（先 CSR）。
    - 回收栈内存。
    - 交换 sp 和 sscratch；sp 指向用户栈，sscratch 指向内核栈顶，且依然保存进入 Trap 之前的状态。
- 通过 `sret` 回到应用程序执行，在此之前需要完成（通过复用 `__restore`）：
    - 跳转到程序入口 `0x80400000`。
    - 栈切换到用户栈。
    - sscratch 指向内核栈。
    - 从 S 特权级切换到 U。

### 实验报告

```bash
cd os2
make run LOG=INFO
```

结果如下所示：

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
[kernel] Hello, world!
[ INFO] [kernel] num_app = 7
[ INFO] [kernel] app_0 [0x8020a048, 0x80210118)
[ INFO] [kernel] app_1 [0x80210118, 0x802161e8)
[ INFO] [kernel] app_2 [0x802161e8, 0x8021c2b8)
[ INFO] [kernel] app_3 [0x8021c2b8, 0x80222388)
[ INFO] [kernel] app_4 [0x80222388, 0x80228458)
[ INFO] [kernel] app_5 [0x80228458, 0x8022e528)
[ INFO] [kernel] app_6 [0x8022e528, 0x802345f8)
[ INFO] [kernel] Loading app_0
[ERROR] [kernel] PageFault in application, core dumped.
[ INFO] [kernel] Loading app_1
[ERROR] [kernel] IllegalInstruction in application, core dumped.
[ INFO] [kernel] Loading app_2
[ERROR] [kernel] IllegalInstruction in application, core dumped.
[ INFO] [kernel] Loading app_3
Hello, world from user mode program!
[ INFO] [kernel] Application exited with code 0
[ INFO] [kernel] Loading app_4
power_3 [10000/200000]
power_3 [20000/200000]
power_3 [30000/200000]
power_3 [40000/200000]
power_3 [50000/200000]
power_3 [60000/200000]
power_3 [70000/200000]
power_3 [80000/200000]
power_3 [90000/200000]
power_3 [100000/200000]
power_3 [110000/200000]
power_3 [120000/200000]
power_3 [130000/200000]
power_3 [140000/200000]
power_3 [150000/200000]
power_3 [160000/200000]
power_3 [170000/200000]
power_3 [180000/200000]
power_3 [190000/200000]
power_3 [200000/200000]
3^200000 = 871008973(MOD 998244353)
Test power_3 OK!
[ INFO] [kernel] Application exited with code 0
[ INFO] [kernel] Loading app_5
power_5 [10000/140000]
power_5 [20000/140000]
power_5 [30000/140000]
power_5 [40000/140000]
power_5 [50000/140000]
power_5 [60000/140000]
power_5 [70000/140000]
power_5 [80000/140000]
power_5 [90000/140000]
power_5 [100000/140000]
power_5 [110000/140000]
power_5 [120000/140000]
power_5 [130000/140000]
power_5 [140000/140000]
5^140000 = 386471875(MOD 998244353)
Test power_5 OK!
[ INFO] [kernel] Application exited with code 0
[ INFO] [kernel] Loading app_6
power_7 [10000/160000]
power_7 [20000/160000]
power_7 [30000/160000]
power_7 [40000/160000]
power_7 [50000/160000]
power_7 [60000/160000]
power_7 [70000/160000]
power_7 [80000/160000]
power_7 [90000/160000]
power_7 [100000/160000]
power_7 [110000/160000]
power_7 [120000/160000]
power_7 [130000/160000]
power_7 [140000/160000]
power_7 [150000/160000]
power_7 [160000/160000]
7^160000 = 667897727(MOD 998244353)
Test power_7 OK!
[ INFO] [kernel] Application exited with code 0
Panicked at src/batch.rs:68 All applications completed!
```

执行测试：

```bash
cd ..
make test2
```

结果显示：

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
[kernel] Hello, world!
Hello, world from user mode program!
power_3 [10000/200000]
power_3 [20000/200000]
power_3 [30000/200000]
power_3 [40000/200000]
power_3 [50000/200000]
power_3 [60000/200000]
power_3 [70000/200000]
power_3 [80000/200000]
power_3 [90000/200000]
power_3 [100000/200000]
power_3 [110000/200000]
power_3 [120000/200000]
power_3 [130000/200000]
power_3 [140000/200000]
power_3 [150000/200000]
power_3 [160000/200000]
power_3 [170000/200000]
power_3 [180000/200000]
power_3 [190000/200000]
power_3 [200000/200000]
3^200000 = 871008973(MOD 998244353)
Test power_3 OK19074!
power_5 [10000/140000]
power_5 [20000/140000]
power_5 [30000/140000]
power_5 [40000/140000]
power_5 [50000/140000]
power_5 [60000/140000]
power_5 [70000/140000]
power_5 [80000/140000]
power_5 [90000/140000]
power_5 [100000/140000]
power_5 [110000/140000]
power_5 [120000/140000]
power_5 [130000/140000]
power_5 [140000/140000]
5^140000 = 386471875(MOD 998244353)
Test power_5 OK19074!
power_7 [10000/160000]
power_7 [20000/160000]
power_7 [30000/160000]
power_7 [40000/160000]
power_7 [50000/160000]
power_7 [60000/160000]
power_7 [70000/160000]
power_7 [80000/160000]
power_7 [90000/160000]
power_7 [100000/160000]
power_7 [110000/160000]
power_7 [120000/160000]
power_7 [130000/160000]
power_7 [140000/160000]
power_7 [150000/160000]
power_7 [160000/160000]
7^160000 = 667897727(MOD 998244353)
Test power_7 OK19074!
Panicked at src/batch.rs:68 All applications completed!
make[2]: Leaving directory '/mnt/workplace/os'
python3 check/ch2.py < stdout-ch2
[PASS] found <Hello, world from user mode program!>
[PASS] found <Test power_3 OK19074!>
[PASS] found <Test power_5 OK19074!>
[PASS] found <Test power_7 OK19074!>
[PASS] not found <FAIL: T.T>

Test passed19074: 5/5
```

### 参考和说明

- 教程、图片和代码均来自：https://learningos.github.io/rust-based-os-comp2022/



