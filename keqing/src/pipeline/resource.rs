use std::collections::HashSet;

/// Identifier of a unique resource.
pub type ResourceID<'a> = [&'a str];

/// A handle to a unique resource that must not be accessed by two parties
/// at the same time. Resource is released automatically when it is no longer
/// being referenced.
///
/// Users are required to implement the deadlock prevention algorithms
/// manually.
pub struct Resource {
    /// (Exposed) resource identifier.
    id: String,
    /// An *unsafe* reference to the issuer when is dropped.
    manager: usize,
}

impl Drop for Resource {
    fn drop(&mut self) -> () {
        let manager_ref = unsafe { &mut *(self.manager as *mut ResourceManager) };
        manager_ref.release(&self);
    }
}

/// Manager responsible for unique resource acquisition and release. Exclusive
/// access is guaranteed to users when is requested. When such demands cannot
/// be fulfilled, an error would be raised.
pub struct ResourceManager {
    identifiers: HashSet<String>,
}

impl ResourceManager {
    /// Creates default resource manager.
    pub fn new() -> Self {
        Self {
            identifiers: HashSet::new(),
        }
    }
    /// Acquire resource from manager.
    pub fn acquire(&mut self, id: &ResourceID) -> Resource {
        let id = self.expose_identifier(id);
        if self.identifiers.contains(&id) {
            panic!("resource '{id}' cannot be shared");
        }

        self.identifiers.insert(id.clone());
        Resource {
            id: id,
            manager: &*self as *const Self as usize,
        }
    }

    /// Release resource from manager.
    pub fn release(&mut self, resource: &Resource) -> () {
        let id = &resource.id;
        if !self.identifiers.contains(id) {
            panic!("double free of resource '{id}'");
        }

        self.identifiers.remove(id);
    }

    /// Convert resource identifier to a unique string for ease of storage.
    fn expose_identifier(&self, id: &ResourceID) -> String {
        let components: Vec<String> = id
            .iter()
            .map(|component| component.replace("/", "//"))
            .collect();
        components.join("/")
    }
}
