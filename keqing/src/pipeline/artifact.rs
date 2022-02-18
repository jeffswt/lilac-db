use std::fs;
use std::io;
use std::path;

/// Path of a produced artifact file (or directory) is composed of string
/// components.
pub type ArtifactID<'a> = &'a [&'a str];

/// A specific artifact node either consumed by, produced from tasks, or
/// requested by the user.
///
/// An artifact may either refer to one or multiple cache files that physically
/// exist, or refer to none and instead be a logical step that performs certain
/// tasks on a build pipeline.
///
/// One artifact ID **MUST** refer to at most 1 local file.
///
/// For artifacts that refer to local cache files, a validation on whether the
/// cache still exists is suggested before actually producing this cache file
/// (again).
pub trait Artifact<'a> {
    /// An identifier of this artifact.
    const id: ArtifactID<'a>;
}

/// Provides an interface to file-system-like operations upon artifacts or
/// their underlying data.
///
/// Uses the *facade* pattern.
pub struct ArtifactManager;

impl ArtifactManager {
    /// Validate if an `ArtifactID` has a corresponding file lying in the file
    /// system.
    pub fn exists(&self, id: ArtifactID) -> bool {
        path::PathBuf::from(self.expose_path(id)).exists()
    }

    /// Opens a handle to an (existing) artifact file.
    ///
    /// Attempting to open a non-existent ID would result in an error.
    pub fn read(&self, id: ArtifactID) -> io::Result<fs::File> {
        if !self.exists(id) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "attempting to open non-existent artifact file",
            ));
        }
        fs::File::open(self.expose_path(id))
    }

    /// Opens a handle to a newly created file with an artifact ID.
    ///
    /// Attempting to open an existing file would result in an error. Use
    /// `.overwrite` instead.
    pub fn write(&self, id: ArtifactID) -> io::Result<fs::File> {
        if self.exists(id) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "cannot overwrite existing file",
            ));
        }
        let path = self.expose_path(id);

        let mut dir_path = path::PathBuf::from(&path);
        dir_path.pop();
        fs::create_dir_all(dir_path.to_str().unwrap())?;

        fs::File::create(path)
    }

    /// Opens a handle to a newly created file (or overwritten file if the
    /// artifact with given ID is existent).
    ///
    /// Warning: will delete original file if not taken with care.
    pub fn overwrite(&self, id: ArtifactID) -> io::Result<fs::File> {
        if self.exists(id) {
            self.purge(id)?;
        }
        self.write(id)
    }

    /// Removes artifact file in the underlying FS.
    ///
    /// Removing a non-existent ID would result in an error.
    pub fn purge(&self, id: ArtifactID) -> io::Result<()> {
        fs::remove_file(self.expose_path(id))
    }

    /// Removes entire artifact directory in the underlying FS.
    ///
    /// Removing a non-existent directory would result in an error.
    pub fn purge_dir(&self, dir_id: ArtifactID) -> io::Result<()> {
        fs::remove_dir_all(self.expose_path(dir_id))
    }

    /// Retrieves the absolute path in FS by `ArtifactID`.
    fn expose_path(&self, id: ArtifactID) -> String {
        let parent_path = "./cache"; // TODO: config-ize this field.
        let parent = match fs::canonicalize(parent_path) {
            Ok(ok) => ok,
            Err(_) => {
                fs::create_dir_all(parent_path).unwrap();
                fs::canonicalize(parent_path).unwrap()
            }
        };
        let mut p = path::PathBuf::from(parent);
        for c in id {
            p.push(c);
        }
        p.to_str().unwrap().to_string()
    }
}
