use Std;
use ap::io;
pub use ap::io::ErrorKind;

pub type Result<T> = io::Result<T, Std>;
pub type Error = io::Error<Std>;