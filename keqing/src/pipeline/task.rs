use crate::pipeline::artifact;

/// A (synchronous) task node consuming artifacts and produces exactly 1
/// artifact.
///
/// Task nodes should implement a factory class inheriting this trait. Every
/// time a task is called a new instance would be initialized.
///
/// Asynchronocy is not required and not recommended due to increased
/// complexity.
pub trait Task<'a, Params, Artifact>
where
    Artifact: artifact::Artifact<'a>,
{
    /// Produce artifacts with parameters. Incoming artifacts should be
    /// manually requested from the upstream tasks dynamically.
    ///
    /// We suggest producing exactly 1 artifact.
    fn execute(params: &Params) -> Vec<Artifact>;
}
