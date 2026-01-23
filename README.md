<p align="center">
  <img src="assets/icons/logo.svg" alt="zTerm Logo" width="128" height="128">
</p>

<h1 align="center">zTerm</h1>

<p align="center">
  <strong>下一代终端体验，为效率而生</strong>
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=flat-square" alt="Platform"></a>
  <a href="#"><img src="https://img.shields.io/badge/rust-1.85+-orange?style=flat-square&logo=rust" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-CC%20BY--NC%204.0-green?style=flat-square" alt="License"></a>
  <a href="#"><img src="https://img.shields.io/badge/status-alpha-yellow?style=flat-square" alt="Status"></a>
</p>

<p align="center">
  <a href="#-特性">特性</a> •
  <a href="#-安装">安装</a> •
  <a href="#-主题">主题</a> •
  <a href="#-路线图">路线图</a> •
  <a href="#-许可证">许可证</a>
</p>

---

<p align="center">
  <img src="https://via.placeholder.com/800x500/1a1a1a/00d4aa?text=zTerm+Screenshot" alt="zTerm Screenshot" width="800">
</p>

---

## ✨ 特性

<table>
<tr>
<td width="50%">

### ⚡ 极致性能

基于 Rust 和 GPU 加速渲染引擎构建，启动迅速，响应流畅，即使处理大量输出也能保持丝滑体验。

</td>
<td width="50%">

### 🎨 精美界面

现代化的 UI 设计，支持多种精美主题，让你的终端工作变成一种享受。

</td>
</tr>
<tr>
<td width="50%">

### 🖥️ 跨平台

原生支持 Windows、macOS 和 Linux，一致的体验，无缝切换。

</td>
<td width="50%">

### 📑 多标签与分屏

灵活的多标签页和分屏布局，轻松管理多个终端会话。

</td>
</tr>
<tr>
<td width="50%">

### 🔍 智能补全

内置智能命令补全和历史搜索，让你的命令行效率倍增。

</td>
<td width="50%">

### 🧱 区块化输出

每条命令的输出独立成块，便于浏览、复制和重新执行。

</td>
</tr>
</table>

---

## 🚀 安装

zTerm 目前处于早期开发阶段，暂未提供预编译版本。

如需体验，请从源码构建：

```bash
git clone https://github.com/user/zterm.git
cd zterm
cargo build --release
```

> 构建需要 Rust 1.85+ 环境

---

## 🎨 主题

zTerm 内置多款精心设计的主题：

<table>
<tr>
<td align="center" width="33%">
<img src="https://via.placeholder.com/200x120/1a1a1a/00d4aa?text=Dark" alt="Dark Theme"><br>
<strong>Dark</strong><br>
<sub>经典深色主题</sub>
</td>
<td align="center" width="33%">
<img src="https://via.placeholder.com/200x120/282a36/bd93f9?text=Dracula" alt="Dracula Theme"><br>
<strong>Dracula</strong><br>
<sub>优雅的紫色调</sub>
</td>
<td align="center" width="33%">
<img src="https://via.placeholder.com/200x120/24273a/8aadf4?text=More..." alt="More Themes"><br>
<strong>更多主题</strong><br>
<sub>持续更新中...</sub>
</td>
</tr>
</table>

---

## 🛣️ 路线图

| 状态 | 功能 | 描述 |
|:---:|---|---|
| ✅ | 基础终端 | 核心终端模拟功能 |
| 🔄 | 完整 VT 支持 | 完整的 VT100/VT220 转义序列支持 |
| 🔄 | 多标签页 | 标签页管理和快捷切换 |
| 📋 | 分屏布局 | 水平/垂直分屏 |
| 📋 | 智能补全 | 命令自动补全和建议 |
| 📋 | 命令面板 | 快速访问所有功能 |
| 📋 | SSH 集成 | 内置 SSH 连接管理 |
| 📋 | AI 助手 | AI 驱动的命令帮助 |

<sub>✅ 已完成 &nbsp; 🔄 进行中 &nbsp; 📋 计划中</sub>

---

## ⌨️ 快捷键

| 快捷键 | 功能 |
|---|---|
| `Ctrl + Shift + T` | 新建标签页 |
| `Ctrl + Shift + W` | 关闭当前标签页 |
| `Ctrl + Tab` | 切换到下一个标签页 |
| `Ctrl + Shift + P` | 打开命令面板 |
| `Ctrl + Shift + D` | 垂直分屏 |
| `Ctrl + Shift + E` | 水平分屏 |

---

## 💡 为什么选择 zTerm？

> *"终端是开发者的家，应该既强大又舒适。"*

- **更快** - Rust + GPU 渲染，比传统终端快数倍
- **更美** - 精心设计的界面，告别单调的黑底白字
- **更智能** - 内置智能功能，减少重复操作
- **更现代** - 区块化输出，让终端不再是滚动的文字流

---

## 📄 许可证

zTerm 采用 [CC BY-NC 4.0](LICENSE) 许可证，允许非商业使用、修改和分发，禁止商业用途。

---

<p align="center">
  <sub>使用 ❤️ 和 Rust 构建</sub>
</p>
