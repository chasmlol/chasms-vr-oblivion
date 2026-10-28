"""Starts a program without showing or activating it (tests: the game on the emulated headset while
the user works; the adapter's `background` setting keeps its window off-screen after that). Prints
its process id; with `--wait <seconds>`, waits that long and prints its exit code if it ended
(debugging a start). (Not on a desktop of its own: Direct3D 9 can't make a device there.)

    python tools/start_hidden.py [--wait <s>] <exe> [args...]

Oblivion runs straight from its exe when `SteamAppId` is set (Steam running); the caller sets it.
"""
import ctypes
import ctypes.wintypes as w
import os
import subprocess
import sys

STARTF_USESHOWWINDOW = 1
SW_SHOWNOACTIVATE = 4


class STARTUPINFO(ctypes.Structure):
    _fields_ = [("cb", w.DWORD), ("lpReserved", w.LPWSTR), ("lpDesktop", w.LPWSTR), ("lpTitle", w.LPWSTR),
                ("dwX", w.DWORD), ("dwY", w.DWORD), ("dwXSize", w.DWORD), ("dwYSize", w.DWORD),
                ("dwXCountChars", w.DWORD), ("dwYCountChars", w.DWORD), ("dwFillAttribute", w.DWORD),
                ("dwFlags", w.DWORD), ("wShowWindow", w.WORD), ("cbReserved2", w.WORD),
                ("lpReserved2", ctypes.c_void_p), ("hStdInput", w.HANDLE), ("hStdOutput", w.HANDLE), ("hStdError", w.HANDLE)]


class PROCESS_INFORMATION(ctypes.Structure):
    _fields_ = [("hProcess", w.HANDLE), ("hThread", w.HANDLE), ("dwProcessId", w.DWORD), ("dwThreadId", w.DWORD)]


def main():
    args = sys.argv[1:]
    wait = None
    if args[:1] == ["--wait"]:
        wait = float(args[1])
        args = args[2:]
    exe = os.path.abspath(args[0])
    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    si = STARTUPINFO()
    si.cb = ctypes.sizeof(si)
    si.dwFlags = STARTF_USESHOWWINDOW
    si.wShowWindow = SW_SHOWNOACTIVATE
    pi = PROCESS_INFORMATION()
    cmd = subprocess.list2cmdline([exe] + args[1:])
    if not k32.CreateProcessW(exe, cmd, None, None, False, 0, None, os.path.dirname(exe), ctypes.byref(si), ctypes.byref(pi)):
        sys.exit(f"CreateProcess: {ctypes.get_last_error()}")
    print(pi.dwProcessId)
    if wait is not None:
        k32.WaitForSingleObject(pi.hProcess, int(wait * 1000))
        code = w.DWORD()
        k32.GetExitCodeProcess(pi.hProcess, ctypes.byref(code))
        print("running" if code.value == 259 else "exited with %#x" % code.value)


if __name__ == "__main__":
    main()
