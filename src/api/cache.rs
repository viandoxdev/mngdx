use std::{any::Any, collections::HashMap, time::Instant};

use uuid::Uuid;

use super::structs::json::data::RelationshipKind;

/// Simple cache with support for directed relationships
pub struct ApiCache {
    data: HashMap<Uuid, Box<dyn Any + Send + Sync>>,
    relationships: HashMap<Uuid, Vec<(Uuid, RelationshipKind)>>,
    expiration_dates: HashMap<Uuid, Instant>,
}

impl ApiCache {
    /// Make a new cache
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            relationships: HashMap::new(),
            expiration_dates: HashMap::new(),
        }
    }
    /// Add element in cache
    pub fn insert<T: Any + Send + Sync>(&mut self, uuid: Uuid, value: T, expire: Option<Instant>) {
        log::trace!("Add {uuid} to cache");

        self.data.insert(uuid, Box::new(value));

        self.relationships.insert(uuid, Vec::new());

        if let Some(instant) = expire {
            self.expiration_dates.insert(uuid, instant);
        }
    }
    /// Remove element from the cache.
    pub fn remove(&mut self, uuid: &Uuid) {
        log::trace!("Remove {uuid} from cache");
        self.data.remove(uuid);
        self.relationships.remove(uuid);
        self.expiration_dates.remove(uuid);
    }
    /// Add a relationship from a to b.
    pub fn link(&mut self, a: &Uuid, b: &Uuid, kind: RelationshipKind) {
        log::trace!("Link {a} -> {b} ({kind:?})");

        if let Some(rels) = self.relationships.get_mut(a) {
            rels.push((*b, kind));
        } else {
            self.relationships.insert(*a, vec![(*b, kind)]);
        }
    }
    /// Remove relationship from a to b
    pub fn unlink(&mut self, a: &Uuid, b: &Uuid) {
        log::trace!("Remove link between {a} and {b}");
        if let Some(rels) = self.relationships.get_mut(a) {
            for (i, e) in rels.iter().enumerate() {
                if e.0 == *b {
                    rels.remove(i);
                    break;
                }
            }
        }
    }
    /// Get (clone) data with specific uuid and type, returns None if either are wrong.
    pub fn get<T: Any + Clone>(&mut self, uuid: &Uuid) -> Option<T> {
        log::trace!("Access {uuid}");

        // delete if expired
        if let Some(exp) = self.expiration_dates.get(uuid) {
            if Instant::now().cmp(exp) == std::cmp::Ordering::Greater {
                log::trace!("Removing expired data");
                self.remove(uuid);
                return None;
            }
        }

        match self.data.get(uuid) {
            Some(boxed) => boxed.downcast_ref::<T>().cloned(),
            None => None,
        }
    }
    /// Get uuid of objects linked to another.
    pub fn get_linked(&self, uuid: &Uuid, kind: RelationshipKind) -> Option<Vec<Uuid>> {
        log::trace!("Access related to {uuid} ({kind:?})");
        self.relationships.get(uuid).map(|rels| {
            rels.iter()
                .filter_map(|x| if x.1 == kind { Some(x.0) } else { None })
                .collect()
        })
    }
    /// clear the cache
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
