use Std;
use ap::ffi;

pub type CStr = ffi::CStr<Std>;
pub type CString = ffi::CString<Std>;

pub type OsStr = ffi::OsStr<Std>;
pub type OsString = ffi::OsString<Std>;
