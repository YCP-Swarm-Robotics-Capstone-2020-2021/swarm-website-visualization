use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
};
use twox_hash::XxHash32;
use gen_vec::{Index, closed::ClosedGenVec};

pub type ResourceHandle = Index;
pub struct ResourceManager
{
    resources: ClosedGenVec<Vec<u8>>,
    handle_map: HashMap<String, ResourceHandle, BuildHasherDefault<XxHash32>>,
}

impl ResourceManager
{
    pub fn new() -> ResourceManager
    {
        ResourceManager
        {
            resources: ClosedGenVec::new(),
            handle_map: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, resource: Vec<u8>) -> ResourceHandle
    {
        self.resources.insert(resource)
    }

    /// Insert the resource and associate it with a string key
    ///
    /// If `name` already exists, it overwrites the existing resource
    #[allow(dead_code)]
    pub fn insert_with_name(&mut self, name: String, resource: Vec<u8>) -> ResourceHandle
    {
        let handle = self.insert(resource);
        if let Some(old) = self.handle_map.insert(name, handle)
        {
            self.resources.remove(old);
        }
        handle
    }

    #[allow(dead_code)]
    pub fn get(&self, handle: ResourceHandle) -> Option<&Vec<u8>>
    {
        self.resources.get(handle)
    }

    #[allow(dead_code)]
    pub fn get_by_name(&self, name: &String) -> Option<&Vec<u8>>
    {
        let handle = *self.handle_map.get(name)?;
        self.get(handle)
    }

    #[allow(dead_code)]
    pub fn get_named_handle(&self, name: &String) -> Option<ResourceHandle>
    {
        self.handle_map.get(name).copied()
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, handle: ResourceHandle) -> Option<Vec<u8>>
    {
        self.resources.remove(handle)
    }

    #[allow(dead_code)]
    pub fn remove_by_name(&mut self, name: &String) -> Option<Vec<u8>>
    {
        let handle = self.handle_map.remove(name)?;
        self.remove(handle)
    }
}