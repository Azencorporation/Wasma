# WASMA: Windows Assignment System Monitoring Architecture

**WASMA**, or Windows Assignment System Monitoring Architecture, is a cross-platform infrastructure providing **WM, Compositor, and UI frontend support**, offering full support for **interface management**.


<img src="https://raw.githubusercontent.com/Azencorporation/Wasma/refs/heads/main/logo.png" width="500">

---

## Features

- "Directly manage all windows via multi-instances or singularity instances using grpc, http, https, and tor."
- "Directly control and manage the resource usage of windows manually, and if necessary, limit it."
- "Integration between windows provides direct remoteization support to your window or environment over the network without any remoteing system, of course, this is optional."
- "Optional sandboxing isolation support and permission management interface; applications can be managed by defining basic permissions through the app.manifest file."
- ".desktop alternative .manifest format allows for full control over all customization, resource usage, and everything under this file."
- "xdg-wsdg conversion support: now you can use a modern, advanced, fully customizable DG environment with wsdg. You can also convert the xdg environment to wsdg without any problems in any application."
- "Wasma-UBIN provides a binary conversion interface that fully supports all GTK/QT/ICED-based features. You can directly add any missing platform features to any GTK/QT/ICED binary file, allowing you to experience a complete GTK/QT/ICED environment."
- "It has a CPU-powered rendering system; GPU is optional. You can directly adjust resource management."
- "Full conversion and integration support for Wayland/X11 applications."
- "Completely independent, written in Rust."

---

## Platform Support

- Cross-platform; the **only dependency is Rust**. Other libraries are optional in terms of optimization and customization.  
- Supported platforms: **Linux, Windows, macOS, BSD**  
- Optional legacy version: can be used on **SystemV, Unix, Plan 9, Bell**, or **any Rust-supported operating system**.
# Setuping

```
git clone https://github.com/Azencorporation/Wasma
cd Wasma
cargo build --release
```
# Testing
```
cd Wasma/src/client
cargo bench
```
---

## Development Status

Since WASMA is still in **development**, some features are not yet stable.  
If you would like to contribute, you can submit pull requests here: [Pulls](https://github.com/Azencorporation/Wasma/pulls)


