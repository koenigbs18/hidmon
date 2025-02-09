[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# HIDMON

Simple library for monitoring HID (keyboard, mouse) events.

## Supported Targets

* Windows
    * Tested as working on Windows 

## Example Usage

See `src/main.rs`

## Limitations

* **Windows**

    * You ***MUST*** have a [message loop](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop) running on the same thread as the `HidMonitor` hooks enabled by this function, otherwise your system may become unresponsive!  For maximum safety, ensure the message loop is running **before** enabling any hooks, or shortly after.  For applications which otherwise do not care about handling `WinApi` messages, [`HidMonitor::message_loop`] serves as a convenience function for starting a simple message handler.
    * Only one unique `HidType` can be monitored per running process.  For example, attempting to start multiple `HidMonitor`'s with `HidType::Mouse` will result in an error.
    * Read more about the implications of this function on the `WinApi` documentation for [`SetWindowsHookExA`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa#remarks)

* **Unix**

    * TODO

## TODO

* Unix support
* Unit testing
* Support for multiple callbacks for a given HID type
* Support for user data in callbacks

## License

This project is licensed under the **Apache License 2.0**.  
See the [LICENSE](./LICENSE) file for details.