# SysY 编译器项目简介

本项目是一个使用 Rust 实现的 SysY 编译器课程设计，覆盖了从源代码输入到目标代码生成的完整编译流程。项目目前已经具备词法分析、语法分析、语义分析、中间表示生成、基础优化以及 LLVM 后端代码生成等模块。

## 项目目标

本项目旨在实现一个较完整的教学型编译器，并通过分阶段处理的方式，将 SysY 源程序逐步转换为可执行的目标代码。整个流程尽量保持模块解耦，便于分别调试各个阶段的结果。

## 编译流程

1. 从 input 文件夹中读取输入源程序
2. lexer.rs 负责词法分析，生成 token 序列
3. parser.rs 负责语法分析，构建抽象语法树
4. semantic.rs 负责语义检查、符号表管理与类型分析
5. ir.rs 将语义树降低为内部 IR
6. optimizer.rs 对 IR 执行基础优化
7. llvm_backend.rs 在启用相关特性后生成 LLVM IR 与 AArch64 汇编
8. 最终结合运行时库完成链接并生成可执行文件

## 目录说明

- input 文件夹用于存放输入文件，例如测试源码和运行输入数据
- output 文件夹用于存放编译过程中的各类输出结果与最终产物
- src 文件夹存放编译器各阶段的核心源码实现

## 常见输出结果

程序运行后，会在 output 文件夹中生成以下内容：

- tokens.txt，词法分析结果
- ast.txt，语法树结构
- parse_errors.txt，语法错误信息
- semantic_ast.txt，语义分析后的结构结果
- semantic_errors.txt，语义错误信息
- ir.txt，初始中间表示
- optimized_ir.txt，优化后的中间表示
- optimization_report.txt，优化统计报告
- output.ll，生成的 LLVM IR
- output_aarch64.s，生成的 AArch64 汇编
- sysy_arm64，最终可执行文件

## 启动方法

### 1. 基础运行

在项目根目录下执行：

```bash
cargo run
```

程序会默认读取 input 文件夹中的 sysy.c，并将编译过程中的输出结果写入 output 文件夹。

### 2. 启用 LLVM 后端

如果本地已正确安装 LLVM 及相关依赖，可以执行：

```bash
cargo run --features llvm-backend
```

执行后会额外生成 LLVM IR 与 AArch64 汇编文件。

### 3. 运行 ARM64 测试

Windows 环境下可直接运行：

```bat
run_arm64_test.cmd
```

该脚本会调用 WSL 执行测试流程，并默认使用 input 文件夹中的测试输入文件。

### 4. 仅检查编译是否通过

如果只想验证工程能否正常构建，可以执行：

```bash
cargo check
```

## 环境要求

为保证项目可以正常运行，建议准备以下环境：

- Rust 工具链
- Cargo 构建工具
- 如果需要启用 LLVM 后端，还需要本地安装可用的 LLVM
- 如果需要执行 ARM64 相关测试，建议在 Windows 下配合 WSL 使用

## 使用说明

- 将待编译的 SysY 源程序放入 input 文件夹，并命名为 sysy.c
- 执行对应命令后，可在 output 文件夹中查看每个阶段生成的结果
- 如果需要调试某个阶段，可直接查看对应输出文件内容

## 注意事项

- 若未启用 llvm-backend 特性，程序将不会生成 LLVM IR 和汇编文件
- 若本地 LLVM 环境不完整，启用后端时可能会出现依赖缺失问题
- 建议先使用 cargo check 或 cargo run 验证前端流程是否正常

## 当前特点

- 支持整型、浮点型、数组、函数、条件分支与循环语句
- 支持常量折叠、公共子表达式消除和死代码消除
- 支持将优化后的 IR 继续降低到 LLVM 后端
- 便于观察每个编译阶段的中间结果

## 适用场景

本项目适合作为编译原理课程实验、SysY 子集编译器实现参考，以及后续继续扩展优化与目标平台适配的基础工程
