use crate::pipeline::artifact;

/// A (synchronous) task node consuming artifacts and producing artifacts.
///
/// Task nodes should implement a static class inheriting this trait and inject
/// it into the pipeline scheduler.
///
/// Since the underlying processes had already taken care of the parallelism
/// and consumed all disk resources, a task doesn't have to be asynchronous,
/// and it shouldn't.
pub trait Task<'a, Params, Artifact>
where
    Artifact: artifact::Artifact<'a>,
{
    /// A declarative definition on if the artifact ID may be produced by this
    /// task node. Not that two nodes may **NEVER** produce the same artifacts.
    fn produces(id: &artifact::ArtifactID<'a>) -> bool;

    /// Produce artifacts with parameters. Incoming artifacts are to be
    /// manually taken from the scheduler as it might be difficult to
    /// determine.
    fn execute(params: &Params) -> Vec<Artifact>;
}
