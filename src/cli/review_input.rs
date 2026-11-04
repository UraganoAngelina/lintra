//! Bounded, cancellation-aware review input, without detached stdin threads.
//! Only ASCII confirmation text is accepted, never key material or passwords.
use lintra::CancellationToken;
use std::{error::Error, io};
type Failure = Box<dyn Error>;

#[derive(Default)]
struct Line(Vec<u8>);
impl Line {
    fn push(&mut self, byte: u8) -> Result<Option<String>, Failure> {
        if matches!(byte, b'\n' | b'\r') {
            return Ok(Some(self.finish()));
        }
        if !byte.is_ascii() || byte < b' ' || self.0.len() >= 256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid or oversized historical review input",
            )
            .into());
        }
        self.0.push(byte);
        Ok(None)
    }
    fn finish(&mut self) -> String {
        // push accepts only ASCII, so UTF-8 decoding has no external fallibility.
        String::from_utf8(std::mem::take(&mut self.0)).expect("ASCII review input")
    }
    fn eof(&mut self) -> Option<String> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.finish())
        }
    }
}

#[cfg(unix)]
pub(super) fn read(cancellation: &CancellationToken) -> Result<Option<String>, Failure> {
    read_fd(libc::STDIN_FILENO, cancellation)
}
#[cfg(unix)]
fn read_fd(
    fd: std::os::fd::RawFd,
    cancellation: &CancellationToken,
) -> Result<Option<String>, Failure> {
    let mut line = Line::default();
    loop {
        if cancellation.is_cancelled() {
            return Ok(None);
        }
        let mut event = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one correctly sized writable pollfd and finite50ms timeout.
        let ready = unsafe { libc::poll(&mut event, 1, 50) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error.into());
        }
        if ready == 0 {
            continue;
        }
        if event.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err(io::Error::other("historical review input is unavailable").into());
        }
        let mut bytes = [0u8; 256];
        // SAFETY: fd is the caller's live input, buffer is sized writable. poll
        // established readability/EOF; no second reader consumes this stream.
        let count = unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
        if count < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error.into());
        }
        if count == 0 {
            return Ok(line.eof());
        }
        for byte in &bytes[..count as usize] {
            if let Some(value) = line.push(*byte)? {
                return Ok(Some(value));
            }
        }
    }
}

#[cfg(windows)]
pub(super) fn read(cancellation: &CancellationToken) -> Result<Option<String>, Failure> {
    use windows_sys::Win32::{Foundation::*, System::Console::*};
    // SAFETY: GetStdHandle returns a borrowed process handle; ownership is not
    // transferred, and this function never closes the original standard input.
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::other("historical review input is unavailable").into());
    }
    read_handle(handle, cancellation)
}
#[cfg(windows)]
fn read_handle(
    handle: windows_sys::Win32::Foundation::HANDLE,
    cancellation: &CancellationToken,
) -> Result<Option<String>, Failure> {
    use std::{
        fs::File,
        io::{Read, Write},
        os::windows::io::BorrowedHandle,
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::*,
        Storage::FileSystem::*,
        System::{Console::*, Pipes::*, Threading::*},
    };
    let mut mode = 0;
    // SAFETY: caller supplies a live borrowed input handle and writable mode.
    let console = unsafe { GetConsoleMode(handle, &mut mode) } != 0;
    // SAFETY: same live handle. No error data/paths or input text are logged.
    let kind = unsafe { GetFileType(handle) };
    // SAFETY: borrowed for this call only; cloning creates a separately owned
    // handle. Dropping File closes the clone, never the process's stdin handle.
    let owned = unsafe { BorrowedHandle::borrow_raw(handle) }.try_clone_to_owned()?;
    let mut input = File::from(owned);
    let mut line = Line::default();
    loop {
        if cancellation.is_cancelled() {
            return Ok(None);
        }
        if console {
            // Console input signals for mouse/window events too. Read one raw
            // INPUT_RECORD rather than a line-buffered ReadFile that could wait
            // indefinitely after a key event and hide Ctrl+C cancellation.
            // SAFETY: live console handle, finite50ms wait.
            match unsafe { WaitForSingleObject(handle, 50) } {
                WAIT_TIMEOUT => continue,
                WAIT_OBJECT_0 => {}
                _ => return Err(io::Error::other("historical console wait failed").into()),
            }
            let mut record = INPUT_RECORD::default();
            let mut count = 0;
            // SAFETY: readiness established; correctly sized initialized output.
            if unsafe { ReadConsoleInputW(handle, &mut record, 1, &mut count) } == 0 {
                return Err(io::Error::other("historical console read failed").into());
            }
            if count == 0 || u32::from(record.EventType) != KEY_EVENT {
                continue;
            }
            // SAFETY: EventType was checked before accessing the union arm.
            let key = unsafe { record.Event.KeyEvent };
            if key.bKeyDown == 0 {
                continue;
            }
            // SAFETY: ReadConsoleInputW populates the UnicodeChar union member.
            let character = unsafe { key.uChar.UnicodeChar };
            if character == 0 {
                continue;
            }
            if character > 127 {
                return Err(io::Error::other("historical confirmation requires ASCII").into());
            }
            for _ in 0..key.wRepeatCount {
                if character == 8 {
                    if line.0.pop().is_some() {
                        io::stderr().write_all(b"\x08 \x08")?;
                    }
                    continue;
                }
                let completed = line.push(character as u8)?;
                let mut echo = io::stderr().lock();
                echo.write_all(&[if character == 13 {
                    b'\n'
                } else {
                    character as u8
                }])?;
                echo.flush()?;
                if let Some(value) = completed {
                    return Ok(Some(value));
                }
            }
            continue;
        }
        let available = if kind == FILE_TYPE_PIPE {
            let mut bytes = 0;
            // SAFETY: live input pipe; no payload buffer, writable byte count.
            if unsafe {
                PeekNamedPipe(
                    handle,
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut bytes,
                    ptr::null_mut(),
                )
            } == 0
            {
                // SAFETY: called immediately after the failed Win32 API.
                if unsafe { GetLastError() } == ERROR_BROKEN_PIPE {
                    return Ok(line.eof());
                }
                return Err(io::Error::other("historical pipe read failed").into());
            }
            if bytes == 0 {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            bytes.min(256) as usize
        } else if kind == FILE_TYPE_DISK {
            256
        } else {
            return Err(io::Error::other("unsupported historical review input").into());
        };
        let mut bytes = [0u8; 256];
        let count = input.read(&mut bytes[..available])?;
        if count == 0 {
            return Ok(line.eof());
        }
        for byte in &bytes[..count] {
            if let Some(value) = line.push(*byte)? {
                return Ok(Some(value));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn native_windows_pipe_readiness_eof_and_cancellation_are_real() {
        use std::{
            fs::File,
            io::Write,
            os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
            ptr,
        };
        use windows_sys::Win32::System::Pipes::CreatePipe;
        let (mut reader, mut writer) = (ptr::null_mut(), ptr::null_mut());
        // SAFETY: writable handle outputs; null attributes mean non-inheritable
        // private test handles, and zero selects the normal system buffer size.
        assert_ne!(
            unsafe { CreatePipe(&mut reader, &mut writer, ptr::null(), 0) },
            0
        );
        // SAFETY: CreatePipe returned two distinct newly owned kernel handles.
        let reader = unsafe { OwnedHandle::from_raw_handle(reader) };
        // SAFETY: the second successful output is transferred exactly once.
        let mut writer = File::from(unsafe { OwnedHandle::from_raw_handle(writer) });
        writer.write_all(b"ABORT\n").unwrap();
        assert_eq!(
            read_handle(reader.as_raw_handle(), &CancellationToken::default()).unwrap(),
            Some("ABORT".into())
        );
        let token = CancellationToken::default();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(20));
                token.cancel();
            });
            let start = std::time::Instant::now();
            assert_eq!(read_handle(reader.as_raw_handle(), &token).unwrap(), None);
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
        });
        drop(writer);
        assert_eq!(
            read_handle(reader.as_raw_handle(), &CancellationToken::default()).unwrap(),
            None
        );
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "parent invokes this helper in an isolated synthetic native console"]
    fn native_console_worker() {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Console::*;
        // SAFETY: this helper runs in its own child process. It detaches only
        // that process, then allocates its own input queue, never the user's.
        unsafe {
            FreeConsole();
            assert_ne!(AllocConsole(), 0);
        }
        // Command uses STARTF_USESTDHANDLES for null stdio; AllocConsole need
        // not replace those redirected handles. Open this child's own console
        // queue explicitly, instead of accidentally testing its NUL stdin.
        let console_input = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONIN$")
            .unwrap();
        let input = console_input.as_raw_handle();
        let records: Vec<_> = "ABORT\r"
            .encode_utf16()
            .map(|character| INPUT_RECORD {
                EventType: KEY_EVENT as u16,
                Event: INPUT_RECORD_0 {
                    KeyEvent: KEY_EVENT_RECORD {
                        bKeyDown: 1,
                        wRepeatCount: 1,
                        uChar: KEY_EVENT_RECORD_0 {
                            UnicodeChar: character,
                        },
                        ..Default::default()
                    },
                },
            })
            .collect();
        let mut written = 0;
        // SAFETY: records is an initialized INPUT_RECORD array alive through
        // the call; input belongs to this isolated process's synthetic console.
        assert_ne!(
            unsafe {
                WriteConsoleInputW(input, records.as_ptr(), records.len() as u32, &mut written)
            },
            0
        );
        assert_eq!(written as usize, records.len());
        assert_eq!(
            read_handle(input, &CancellationToken::default()).unwrap(),
            Some("ABORT".into())
        );
        let token = CancellationToken::default();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(20));
                token.cancel();
            });
            let start = std::time::Instant::now();
            assert_eq!(read_handle(input, &token).unwrap(), None);
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
        });
        // The OS releases the private console at child exit. Keep it alive for
        // libtest's terminal write; no standard handle in the parent is changed.
    }
    #[cfg(windows)]
    #[test]
    fn windows_console_events_and_idle_cancellation_run_in_an_isolated_process() {
        use std::{
            os::windows::process::CommandExt,
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        struct Guard(std::process::Child);
        impl Drop for Guard {
            fn drop(&mut self) {
                if !matches!(self.0.try_wait(), Ok(Some(_))) {
                    let _ = self.0.kill();
                    let _ = self.0.wait();
                }
            }
        }
        let mut child = Guard(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "cli::review_input::tests::native_console_worker",
                    "--ignored",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .unwrap(),
        );
        let end = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if Instant::now() >= end {
                child.0.kill().unwrap();
                child.0.wait().unwrap();
                panic!("native console deadline");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    #[test]
    fn review_text_is_bounded_ascii_and_never_trims_a_mismatched_approval() {
        let mut line = Line::default();
        for byte in b" APPROVE 1 value " {
            assert!(line.push(*byte).unwrap().is_none());
        }
        assert_eq!(line.push(b'\n').unwrap().unwrap(), " APPROVE 1 value ");
        assert_eq!(Line::default().eof(), None);
        assert!(Line::default().push(0xff).is_err());
        let mut line = Line::default();
        for _ in 0..256 {
            line.push(b'A').unwrap();
        }
        assert!(line.push(b'A').is_err());
    }
    #[cfg(unix)]
    #[test]
    fn real_pipe_input_returns_a_line_and_cancellation_interrupts_an_idle_pipe() {
        use std::{
            fs::File,
            io::Write,
            os::fd::{AsRawFd, FromRawFd},
        };
        let mut pair = [-1; 2];
        // SAFETY: sized writable pair, no invalid flags.
        assert_eq!(unsafe { libc::pipe(pair.as_mut_ptr()) }, 0);
        // SAFETY: pipe returned two new distinct owned descriptors.
        let (reader, mut writer) =
            unsafe { (File::from_raw_fd(pair[0]), File::from_raw_fd(pair[1])) };
        writer.write_all(b"ABORT\n").unwrap();
        assert_eq!(
            read_fd(reader.as_raw_fd(), &CancellationToken::default()).unwrap(),
            Some("ABORT".into())
        );
        let token = CancellationToken::default();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(20));
                token.cancel();
            });
            let start = std::time::Instant::now();
            assert_eq!(read_fd(reader.as_raw_fd(), &token).unwrap(), None);
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
        });
        drop(writer);
        assert_eq!(
            read_fd(reader.as_raw_fd(), &CancellationToken::default()).unwrap(),
            None
        );
    }
}
