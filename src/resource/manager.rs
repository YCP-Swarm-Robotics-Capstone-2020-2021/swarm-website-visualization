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
    pub fn insert_with_name<S: Into<String>>(&mut self, name: S, resource: Vec<u8>) -> ResourceHandle
    {
        let handle = self.insert(resource);
        if let Some(old) = self.handle_map.insert(name.into(), handle)
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
    pub fn get_by_name(&self, name: &str) -> Option<&Vec<u8>>
    {
        let handle = *self.handle_map.get(name)?;
        self.get(handle)
    }

    #[allow(dead_code)]
    pub fn get_named_handle(&self, name: &str) -> Option<ResourceHandle>
    {
        self.handle_map.get(name).copied()
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, handle: ResourceHandle) -> Option<Vec<u8>>
    {
        self.resources.remove(handle)
    }

    #[allow(dead_code)]
    pub fn remove_by_name(&mut self, name: &str) -> Option<Vec<u8>>
    {
        let handle = self.handle_map.remove(name)?;
        self.remove(handle)
    }
}

#[cfg(test)]
mod tests
{
    use crate::resource::manager::*;

    #[test]
    fn test_by_handle()
    {
        let mut manager = ResourceManager::new();
        let r1 = manager.insert(vec![0, 1]);
        let r2 = manager.insert(vec![2, 3]);

        assert_eq!(manager.get(r1), Some(&vec![0u8, 1u8]));
        assert_eq!(manager.get(r2), Some(&vec![2u8, 3u8]));
        assert_eq!(manager.remove(r1), Some(vec![0u8, 1u8]));
        assert_eq!(manager.remove(r2), Some(vec![2u8, 3u8]));
    }

    #[test]
    fn test_by_name()
    {
        let mut manager = ResourceManager::new();
        let r1 = manager.insert_with_name("name1".to_string(), vec![0, 1]);
        let r2 = manager.insert_with_name("name2".to_string(), vec![2, 3]);

        assert_eq!(manager.get(r1).unwrap(), &vec![0u8, 1u8]);
        assert_eq!(manager.get(r2).unwrap(), &vec![2u8, 3u8]);
        assert_eq!(manager.get_by_name("name1"), Some(&vec![0u8, 1u8]));
        assert_eq!(manager.get_by_name("name2"), Some(&vec![2u8, 3u8]));
        assert_eq!(manager.get_named_handle(&"name1".to_string()), Some(r1));
        assert_eq!(manager.get_named_handle(&"name2".to_string()), Some(r2));
        assert_eq!(manager.remove_by_name(&"name1".to_string()), Some(vec![0u8, 1u8]));
        assert_eq!(manager.remove_by_name(&"name2".to_string()), Some(vec![2u8, 3u8]));
    }
}