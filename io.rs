use Std;
use ap::io;
pub use ap::io::ErrorKind;

pub use ap::io::Read;

pub type Result<T> = io::Result<T, Std>;
pub type Error = io::Error<Std>;

pub use ap::io::SeekFrom;
pub use ap::io::copy;
