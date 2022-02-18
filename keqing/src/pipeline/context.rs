use crate::pipeline::artifact;

/// Tree of managers and states which task nodes would rely on.
pub struct TaskContext {
    pub artifacts: artifact::ArtifactManager,
}

impl TaskContext {
    pub fn new() -> TaskContext {
        TaskContext {
            artifacts: artifact::ArtifactManager {},
        }
    }
}
