#include "flutter_window.h"

#include <string.h>

#include <optional>

#include "flutter/generated_plugin_registrant.h"

FlutterWindow::FlutterWindow(const flutter::DartProject& project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  // The size here must match the window dimensions to avoid unnecessary surface
  // creation / destruction in the startup path.
  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  // Ensure that basic setup of the controller was successful.
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }
  RegisterPlugins(flutter_controller_->engine());
  activation_channel_ = std::make_unique<
      flutter::MethodChannel<flutter::EncodableValue>>(
      flutter_controller_->engine()->messenger(),
      "yamantaka/activation", &flutter::StandardMethodCodec::GetInstance());
  window_channel_ = std::make_unique<
      flutter::MethodChannel<flutter::EncodableValue>>(
      flutter_controller_->engine()->messenger(), "yamantaka/window",
      &flutter::StandardMethodCodec::GetInstance());
  window_channel_->SetMethodCallHandler(
      [this](const auto& call, auto result) {
        if (call.method_name() == "isFullScreen") {
          result->Success(flutter::EncodableValue(is_fullscreen_));
          return;
        }
        if (call.method_name() == "setFullScreen") {
          const auto* arguments = std::get_if<flutter::EncodableMap>(
              call.arguments());
          if (arguments == nullptr) {
            result->Error("invalid_arguments",
                          "setFullScreen requires an argument map.");
            return;
          }
          const auto value = arguments->find(flutter::EncodableValue("value"));
          if (value == arguments->end() ||
              !std::holds_alternative<bool>(value->second)) {
            result->Error("invalid_arguments",
                          "setFullScreen requires a boolean value.");
            return;
          }
          if (!SetFullScreen(std::get<bool>(value->second))) {
            result->Error("fullscreen_failed",
                          "Windows could not apply the fullscreen state.");
            return;
          }
          result->Success(flutter::EncodableValue(is_fullscreen_));
          return;
        }
        result->NotImplemented();
      });
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  flutter_controller_->engine()->SetNextFrameCallback([&]() {
    this->Show();
  });

  // Flutter can complete the first frame before the "show window" callback is
  // registered. The following call ensures a frame is pending to ensure the
  // window is shown. It is a no-op if the first frame hasn't completed yet.
  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  if (flutter_controller_) {
    window_channel_.reset();
    activation_channel_.reset();
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

bool FlutterWindow::SetFullScreen(bool value) {
  if (value == is_fullscreen_) {
    return true;
  }

  const HWND window = GetHandle();
  if (window == nullptr) {
    return false;
  }

  if (value) {
    placement_before_fullscreen_.length = sizeof(WINDOWPLACEMENT);
    if (!::GetWindowPlacement(window, &placement_before_fullscreen_)) {
      return false;
    }
    style_before_fullscreen_ = ::GetWindowLongPtr(window, GWL_STYLE);
    extended_style_before_fullscreen_ =
        ::GetWindowLongPtr(window, GWL_EXSTYLE);

    MONITORINFO monitor_info{};
    monitor_info.cbSize = sizeof(MONITORINFO);
    const HMONITOR monitor =
        ::MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
    if (!::GetMonitorInfo(monitor, &monitor_info)) {
      return false;
    }

    const LONG_PTR fullscreen_style =
        (style_before_fullscreen_ & ~WS_OVERLAPPEDWINDOW) | WS_POPUP;
    const LONG_PTR fullscreen_extended_style =
        extended_style_before_fullscreen_ &
        ~(WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE |
          WS_EX_STATICEDGE);
    ::SetWindowLongPtr(window, GWL_STYLE, fullscreen_style);
    ::SetWindowLongPtr(window, GWL_EXSTYLE, fullscreen_extended_style);

    const RECT bounds = monitor_info.rcMonitor;
    is_fullscreen_ = true;
    if (!::SetWindowPos(window, HWND_TOP, bounds.left, bounds.top,
                        bounds.right - bounds.left,
                        bounds.bottom - bounds.top,
                        SWP_NOOWNERZORDER | SWP_FRAMECHANGED)) {
      is_fullscreen_ = false;
      ::SetWindowLongPtr(window, GWL_STYLE, style_before_fullscreen_);
      ::SetWindowLongPtr(window, GWL_EXSTYLE,
                         extended_style_before_fullscreen_);
      return false;
    }
    ::SetForegroundWindow(window);
    return true;
  }

  ::SetWindowLongPtr(window, GWL_STYLE, style_before_fullscreen_);
  ::SetWindowLongPtr(window, GWL_EXSTYLE, extended_style_before_fullscreen_);
  is_fullscreen_ = false;
  if (!::SetWindowPlacement(window, &placement_before_fullscreen_)) {
    is_fullscreen_ = true;
    return false;
  }
  ::SetWindowPos(window, nullptr, 0, 0, 0, 0,
                 SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER |
                     SWP_FRAMECHANGED);
  return true;
}

LRESULT
FlutterWindow::MessageHandler(HWND hwnd, UINT const message,
                              WPARAM const wparam,
                              LPARAM const lparam) noexcept {
  // Give Flutter, including plugins, an opportunity to handle window messages.
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
    case WM_COPYDATA: {
      const auto* data = reinterpret_cast<const COPYDATASTRUCT*>(lparam);
      if (data != nullptr && data->dwData == 0x59414D41 &&
          data->lpData != nullptr && data->cbData > 0 &&
          data->cbData <= 64 * 1024 && activation_channel_) {
        const auto* bytes = static_cast<const char*>(data->lpData);
        const auto length = strnlen_s(bytes, data->cbData);
        activation_channel_->InvokeMethod(
            "open", std::make_unique<flutter::EncodableValue>(
                         std::string(bytes, length)));
        return TRUE;
      }
      break;
    }
    case WM_FONTCHANGE:
      flutter_controller_->engine()->ReloadSystemFonts();
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}
