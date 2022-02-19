use crate::pipeline::artifact;
use crate::pipeline::resource;

/// Tree of managers and states which task nodes would rely on.
pub struct TaskContext {
    pub artifacts: artifact::ArtifactManager,
    pub resources: resource::ResourceManager,
}

impl TaskContext {
    pub fn new() -> TaskContext {
        TaskContext {
            artifacts: artifact::ArtifactManager {},
            resources: resource::ResourceManager::new(),
        }
    }
}
