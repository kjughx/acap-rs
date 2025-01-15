#[repr(i32)]
#[derive(Copy, Clone)]
pub enum ErrorCode {
    InvalidValue = 1000,
    InternalError = 2000,
    UnexpectedError = 3000,
    Generic = 4000, // Unused
    InvalidArgument = 5000,
    ServiceUnavailable = 6000,
    BackendError = 7000,
    Unknown = 9999,
}

impl From<i32> for ErrorCode {
    fn from(value: i32) -> Self {
        match value {
            1000 => Self::InvalidValue,
            2000 => Self::InternalError,
            3000 => Self::UnexpectedError,
            4000 => Self::Generic,
            5000 => Self::InvalidArgument,
            6000 => Self::ServiceUnavailable,
            7000 => Self::BackendError,
            _ => Self::Unknown,
        }
    }
}

pub struct Error {
    inner: Option<*mut glib_sys::GError>,
    code: Option<ErrorCode>,
}

impl Error {
    fn new(code: ErrorCode) -> Self {
        Self {
            inner: None,
            code: Some(code),
        }
    }

    pub(crate) fn from_raw(inner: *mut glib_sys::GError) -> Self {
        assert!(!inner.is_null());
        return Self {
            inner: Some(inner),
            code: None,
        };
    }
    pub fn code(&self) -> ErrorCode {
        if let Some(inner) = self.inner {
            unsafe { return ErrorCode::from((*(inner)).code) };
        } else if let Some(code) = self.code {
            return code;
        }

        return ErrorCode::Unknown;
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl Drop for Error {
    fn drop(&mut self) {
        if let Some(inner) = self.inner {
            unsafe { glib_sys::g_error_free(inner) };
        }
    }
}


