#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <windows.h>

#include "flutter_window.h"
#include "utils.h"

int APIENTRY wWinMain(_In_ HINSTANCE instance, _In_opt_ HINSTANCE prev,
                      _In_ wchar_t *command_line, _In_ int show_command) {
  // Attach to console when present (e.g., 'flutter run') or create a
  // new console when running with a debugger.
  if (!::AttachConsole(ATTACH_PARENT_PROCESS) && ::IsDebuggerPresent()) {
    CreateAndAttachConsole();
  }

  // Initialize COM, so that it is available for use in the library and/or
  // plugins.
  ::CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

  // Keep one process per user session. A second Explorer activation forwards
  // its bounded PDF argument list through WM_COPYDATA and exits without
  // creating a competing database owner.
  HANDLE single_instance = ::CreateMutexW(
      nullptr, FALSE, L"Local\\Yamantaka.SingleInstance.v1");
  if (single_instance == nullptr) {
    ::CoUninitialize();
    return EXIT_FAILURE;
  }
  if (::GetLastError() == ERROR_ALREADY_EXISTS) {
    const auto arguments = GetCommandLineArguments();
    std::string message;
    for (size_t index = 0; index < arguments.size() && index < 12; index++) {
      const auto& argument = arguments[index];
      const auto separator = message.empty() ? 0u : 1u;
      if (message.size() + separator + argument.size() + 1 > 64 * 1024) {
        break;
      }
      if (!message.empty()) message.push_back('\n');
      message.append(argument);
    }
    HWND existing = ::FindWindowW(L"FLUTTER_RUNNER_WIN32_WINDOW",
                                  L"Yamantaka");
    if (existing != nullptr && !message.empty()) {
      COPYDATASTRUCT data{};
      data.dwData = 0x59414D41;
      data.cbData = static_cast<DWORD>(message.size() + 1);
      data.lpData = const_cast<char*>(message.c_str());
      ::SendMessage(existing, WM_COPYDATA, 0,
                    reinterpret_cast<LPARAM>(&data));
      ::SetForegroundWindow(existing);
    }
    ::CloseHandle(single_instance);
    ::CoUninitialize();
    return EXIT_SUCCESS;
  }

  flutter::DartProject project(L"data");

  std::vector<std::string> command_line_arguments =
      GetCommandLineArguments();

  project.set_dart_entrypoint_arguments(std::move(command_line_arguments));

  FlutterWindow window(project);
  Win32Window::Point origin(10, 10);
  Win32Window::Size size(1280, 720);
  if (!window.Create(L"Yamantaka", origin, size)) {
    ::CloseHandle(single_instance);
    ::CoUninitialize();
    return EXIT_FAILURE;
  }
  window.SetQuitOnClose(true);

  ::MSG msg;
  while (::GetMessage(&msg, nullptr, 0, 0)) {
    ::TranslateMessage(&msg);
    ::DispatchMessage(&msg);
  }

  ::CoUninitialize();
  ::CloseHandle(single_instance);
  return EXIT_SUCCESS;
}
