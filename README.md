[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# HIDMON

Simple library for monitoring HID (keyboard, mouse) events via callbacks.

## Supported Targets

* Windows
    * Tested as working on Windows 11 Build 22631

## Example Usage

See `src/main.rs`

## Limitations/Warnings

* **Windows**

    * You ***MUST*** have a [message loop](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop) running on the same thread as any `HidMonitor` before calling `HidMonitor::enable`, otherwise your system may become unresponsive!  For maximum safety, ensure the message loop is running **before** enabling any hooks, or shortly after.  For applications which otherwise do not care about handling `WinApi` messages, [`HidMonitor::message_loop`] serves as a convenience function for starting a simple message handler.
    * Read more about the implications of `HidMonitor::enable` on the `WinApi` documentation for [`SetWindowsHookExA`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa#remarks)

* **Unix**

    * TODO

## TODO

* Unix support
* Unit testing
* Handle panics that occur inside inside user-defined callbacks (and documentation on this behavior)

## License

This project is licensed under the **Apache License 2.0**.  
See the [LICENSE](./LICENSE) file for details.