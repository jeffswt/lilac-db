/// An artifact ID is composed of string components like a URL.
pub type ArtifactID<'a> = &'a [&'a str];

/// A specific artifact node either consumed by, produced from tasks, or
/// requested by the user.
///
/// An artifact may either refer to one or multiple cache files that physically
/// exist, or refer to none and instead be a logical step that performs certain
/// tasks on a build pipeline.
///
/// For artifacts that refer to local cache files, a validation on whether the
/// cache still exists is suggested before actually producing this cache file
/// (again).
pub trait Artifact<'a> {
    /// An identifier of this artifact.
    const id: ArtifactID<'a>;
}
