use std::fs;
use std::io;
use std::path;

type CachePath<'a> = &'a [&'a str];

/// Checks if file exists in cache tree.
pub fn exists(path: CachePath) -> bool {
    let x = get_cache_path(path);
    path::PathBuf::from(x).exists()
}

/// Opens file handle for reading on cache tree.
///
/// Throws error upon non-existent file lookup.
pub fn read(path: CachePath) -> io::Result<fs::File> {
    if !exists(path) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "attempting to read non-existent file",
        ));
    }
    Ok(fs::File::open(get_cache_path(path))?)
}

/// Opens file handle for writing on cache tree.
///
/// Throws error upon duplicate file.
pub fn write(path: CachePath) -> io::Result<fs::File> {
    if exists(path) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "attempting to write non-existent file",
        ));
    }
    let path = get_cache_path(path);

    let mut dir_path = path::PathBuf::from(&path);
    dir_path.pop();
    fs::create_dir_all(dir_path.to_str().unwrap())?;

    Ok(fs::File::create(path)?)
}

/// Opens file handle for overwriting on cache tree.
pub fn overwrite(path: CachePath) -> io::Result<fs::File> {
    if exists(path) {
        purge(path)?;
    }
    write(path)
}

/// Removes file (entry) from cache tree.
///
/// Users are required to validate if this entry exists in the cache before
/// purging, otherwise a result should be captured.
pub fn purge(path: CachePath) -> io::Result<()> {
    fs::remove_file(get_cache_path(path))?;
    Ok(())
}

/// Removes entire directory from cache tree.
///
/// **This function is extremely dangerous. Use with care.**
///
/// The original directory would be removed if the to-be-purged directory is
/// a symbolic link to an external location.
pub fn purge_dir(path: CachePath) -> io::Result<()> {
    fs::remove_dir_all(get_cache_path(path))?;
    Ok(())
}

/// Gets the absolute path of file (or directory) with the in-cache path.
fn get_cache_path(path: CachePath) -> String {
    let parent_path = "./cache";
    let parent = match fs::canonicalize(parent_path) {
        Ok(ok) => ok,
        Err(_) => {
            fs::create_dir_all(parent_path).unwrap();
            fs::canonicalize(parent_path).unwrap()
        }
    };
    let mut p = path::PathBuf::from(parent);
    for c in path {
        p.push(c);
    }
    p.to_str().unwrap().to_string()
}
