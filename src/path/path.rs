use std::ffi::OsStr;

use crate::path::{Ancestors, Components, Display, PathBuf};
use crate::{fs, io};

/// This struct is an async version of [`std::path::Path`].
///
/// [`std::path::Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
#[derive(Debug, PartialEq)]
pub struct Path {
    inner: std::path::Path,
}

impl Path {
    /// Produces an iterator over `Path` and its ancestors.
    ///
    /// The iterator will yield the `Path` that is returned if the [`parent`] method is used zero
    /// or more times. That means, the iterator will yield `&self`, `&self.parent().unwrap()`,
    /// `&self.parent().unwrap().parent().unwrap()` and so on. If the [`parent`] method returns
    /// [`None`], the iterator will do likewise. The iterator will always yield at least one value,
    /// namely `&self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// let mut ancestors = Path::new("/foo/bar").ancestors();
    /// assert_eq!(ancestors.next(), Some(Path::new("/foo/bar").into()));
    /// assert_eq!(ancestors.next(), Some(Path::new("/foo").into()));
    /// assert_eq!(ancestors.next(), Some(Path::new("/").into()));
    /// assert_eq!(ancestors.next(), None);
    /// ```
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html
    /// [`parent`]: struct.Path.html#method.parent
    pub fn ancestors(&self) -> Ancestors<'_> {
        self.inner.ancestors()
    }

    /// Yields the underlying [`OsStr`] slice.
    ///
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    pub fn as_os_str(&self) -> &OsStr {
        self.inner.as_os_str()
    }

    /// Returns the canonical, absolute form of the path with all intermediate
    /// components normalized and symbolic links resolved.
    ///
    /// This is an alias to [`fs::canonicalize`].
    ///
    /// [`fs::canonicalize`]: ../fs/fn.canonicalize.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_std::path::{Path, PathBuf};
    ///
    /// let path = Path::new("/foo/test/../test/bar.rs");
    /// assert_eq!(path.canonicalize().await.unwrap(), PathBuf::from("/foo/test/bar.rs"));
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn canonicalize(&self) -> io::Result<PathBuf> {
        fs::canonicalize(self).await
    }

    /// Produces an iterator over the [`Component`]s of the path.
    ///
    /// When parsing the path, there is a small amount of normalization:
    ///
    /// * Repeated separators are ignored, so `a/b` and `a//b` both have
    ///   `a` and `b` as components.
    ///
    /// * Occurrences of `.` are normalized away, except if they are at the
    ///   beginning of the path. For example, `a/./b`, `a/b/`, `a/b/.` and
    ///   `a/b` all have `a` and `b` as components, but `./a/b` starts with
    ///   an additional [`CurDir`] component.
    ///
    /// * A trailing slash is normalized away, `/a/b` and `/a/b/` are equivalent.
    ///
    /// Note that no other normalization takes place; in particular, `a/c`
    /// and `a/b/../c` are distinct, to account for the possibility that `b`
    /// is a symbolic link (so its parent isn't `a`).
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::{Path, Component};
    /// use std::ffi::OsStr;
    ///
    /// let mut components = Path::new("/tmp/foo.txt").components();
    ///
    /// assert_eq!(components.next(), Some(Component::RootDir));
    /// assert_eq!(components.next(), Some(Component::Normal(OsStr::new("tmp"))));
    /// assert_eq!(components.next(), Some(Component::Normal(OsStr::new("foo.txt"))));
    /// assert_eq!(components.next(), None)
    /// ```
    ///
    /// [`Component`]: enum.Component.html
    /// [`CurDir`]: enum.Component.html#variant.CurDir
    pub fn components(&self) -> Components<'_> {
        self.inner.components()
    }

    /// Returns an object that implements [`Display`] for safely printing paths
    /// that may contain non-Unicode data.
    ///
    /// [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// let path = Path::new("/tmp/foo.rs");
    ///
    /// println!("{}", path.display());
    /// ```
    pub fn display(&self) -> Display<'_> {
        self.inner.display()
    }

    /// Determines whether `child` is a suffix of `self`.
    ///
    /// Only considers whole path components to match.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// let path = Path::new("/etc/passwd");
    ///
    /// assert!(path.ends_with("passwd"));
    /// ```
    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool
    where
        P: std::convert::AsRef<std::path::Path>,
    {
        self.inner.ends_with(child)
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `false`.
    ///
    /// If you cannot access the directory containing the file, e.g., because of a
    /// permission error, this will return `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_std::path::Path;
    /// assert_eq!(Path::new("does_not_exist.txt").exists().await, false);
    /// #
    /// # Ok(()) }) }
    /// ```
    ///
    /// # See Also
    ///
    /// This is a convenience function that coerces errors to false. If you want to
    /// check errors, call [fs::metadata].
    ///
    /// [fs::metadata]: ../fs/fn.metadata.html
    pub async fn exists(&self) -> bool {
        fs::metadata(self).await.is_ok()
    }

    /// Extracts the extension of [`self.file_name`], if possible.
    ///
    /// The extension is:
    ///
    /// * [`None`], if there is no file name;
    /// * [`None`], if there is no embedded `.`;
    /// * [`None`], if the file name begins with `.` and has no other `.`s within;
    /// * Otherwise, the portion of the file name after the final `.`
    ///
    /// [`self.file_name`]: struct.Path.html#method.file_name
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// let path = Path::new("foo.rs");
    ///
    /// assert_eq!("rs", path.extension().unwrap());
    /// ```
    pub fn extension(&self) -> Option<&OsStr> {
        self.inner.extension()
    }

    /// Converts a [`Box<Path>`][`Box`] into a [`PathBuf`] without copying or
    /// allocating.
    ///
    /// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
    /// [`PathBuf`]: struct.PathBuf.html
    pub fn into_path_buf(self: Box<Path>) -> PathBuf {
        let rw = Box::into_raw(self) as *mut std::path::Path;
        let inner = unsafe { Box::from_raw(rw) };
        inner.into_path_buf().into()
    }

    /// Queries the file system to get information about a file, directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// This is an alias to [`fs::metadata`].
    ///
    /// [`fs::metadata`]: ../fs/fn.metadata.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_std::path::Path;
    ///
    /// let path = Path::new("/Minas/tirith");
    /// let metadata = path.metadata().await.expect("metadata call failed");
    /// println!("{:?}", metadata.file_type());
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn metadata(&self) -> io::Result<fs::Metadata> {
        fs::metadata(self).await
    }

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This is an alias to [`fs::symlink_metadata`].
    ///
    /// [`fs::symlink_metadata`]: ../fs/fn.symlink_metadata.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
    /// #
    /// use async_std::path::Path;
    ///
    /// let path = Path::new("/Minas/tirith");
    /// let metadata = path.symlink_metadata().await.expect("symlink_metadata call failed");
    /// println!("{:?}", metadata.file_type());
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn symlink_metadata(&self) -> io::Result<fs::Metadata> {
        fs::symlink_metadata(self).await
    }

    /// Directly wraps a string slice as a `Path` slice.
    ///
    /// This is a cost-free conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// Path::new("foo.txt");
    /// ```
    ///
    /// You can create `Path`s from `String`s, or even other `Path`s:
    ///
    /// ```
    /// use async_std::path::Path;
    ///
    /// let string = String::from("foo.txt");
    /// let from_string = Path::new(&string);
    /// let from_path = Path::new(&from_string);
    /// assert_eq!(from_string, from_path);
    /// ```
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(std::path::Path::new(s) as *const std::path::Path as *const Path) }
    }

    /// Converts a `Path` to an owned [`PathBuf`].
    ///
    /// [`PathBuf`]: struct.PathBuf.html
    ///
    /// # Examples
    ///
    /// ```
    /// use async_std::path::{Path, PathBuf};
    ///
    /// let path_buf = Path::new("foo.txt").to_path_buf();
    /// assert_eq!(path_buf, PathBuf::from("foo.txt"));
    /// ```
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.inner.to_path_buf())
    }
}

impl<'a> From<&'a std::path::Path> for &'a Path {
    fn from(path: &'a std::path::Path) -> &'a Path {
        &Path::new(path.as_os_str())
    }
}

impl<'a> Into<&'a std::path::Path> for &'a Path {
    fn into(self) -> &'a std::path::Path {
        std::path::Path::new(&self.inner)
    }
}

impl AsRef<OsStr> for Path {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_ref()
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for OsStr {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}
