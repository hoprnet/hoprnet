/// Extend `std::io::Write` with wincon styling
pub trait WinconStream {
    /// Write colored text to the stream
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize>;
}

impl WinconStream for Box<dyn std::io::Write> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        crate::ansi::write_colored(self, fg, bg, data)
    }
}

impl WinconStream for &'_ mut Box<dyn std::io::Write> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        (**self).write_colored(fg, bg, data)
    }
}

impl WinconStream for std::fs::File {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        crate::ansi::write_colored(self, fg, bg, data)
    }
}

impl WinconStream for &'_ mut std::fs::File {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        (**self).write_colored(fg, bg, data)
    }
}

impl WinconStream for Vec<u8> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        crate::ansi::write_colored(self, fg, bg, data)
    }
}

impl WinconStream for &'_ mut Vec<u8> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        (**self).write_colored(fg, bg, data)
    }
}

impl WinconStream for std::io::Stdout {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        // Ensure exclusive access
        self.lock().write_colored(fg, bg, data)
    }
}

impl WinconStream for std::io::Stderr {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        // Ensure exclusive access
        self.lock().write_colored(fg, bg, data)
    }
}

#[cfg(not(windows))]
mod platform {
    use super::*;

    impl WinconStream for std::io::StdoutLock<'_> {
        fn write_colored(
            &mut self,
            fg: Option<anstyle::AnsiColor>,
            bg: Option<anstyle::AnsiColor>,
            data: &[u8],
        ) -> std::io::Result<usize> {
            crate::ansi::write_colored(self, fg, bg, data)
        }
    }

    impl WinconStream for std::io::StderrLock<'_> {
        fn write_colored(
            &mut self,
            fg: Option<anstyle::AnsiColor>,
            bg: Option<anstyle::AnsiColor>,
            data: &[u8],
        ) -> std::io::Result<usize> {
            crate::ansi::write_colored(self, fg, bg, data)
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;

    impl WinconStream for std::io::StdoutLock<'_> {
        fn write_colored(
            &mut self,
            fg: Option<anstyle::AnsiColor>,
            bg: Option<anstyle::AnsiColor>,
            data: &[u8],
        ) -> std::io::Result<usize> {
            let initial = crate::windows::stdout_initial_colors();
            crate::windows::write_colored(self, fg, bg, data, initial)
        }
    }

    impl WinconStream for std::io::StderrLock<'_> {
        fn write_colored(
            &mut self,
            fg: Option<anstyle::AnsiColor>,
            bg: Option<anstyle::AnsiColor>,
            data: &[u8],
        ) -> std::io::Result<usize> {
            let initial = crate::windows::stderr_initial_colors();
            crate::windows::write_colored(self, fg, bg, data, initial)
        }
    }
}

impl WinconStream for &'_ mut std::io::StdoutLock<'_> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        (**self).write_colored(fg, bg, data)
    }
}

impl WinconStream for &'_ mut std::io::StderrLock<'_> {
    fn write_colored(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        (**self).write_colored(fg, bg, data)
    }
}
