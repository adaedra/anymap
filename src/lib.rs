use std::{any::Any, collections::HashMap, hash::Hash};

pub struct AnyMap<K>
where
    K: Eq + Hash,
{
    h: HashMap<K, Box<dyn Any>>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    KeyNotFound,
    TypeMismatch,
}

impl<K> AnyMap<K>
where
    K: Eq + Hash,
{
    pub fn new() -> AnyMap<K> {
        AnyMap { h: HashMap::new() }
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.h.contains_key(k)
    }

    pub fn contains_key_typed<V>(&self, k: &K) -> Result<(), Error>
    where
        V: 'static,
    {
        match self.h.get(k) {
            None => Err(Error::KeyNotFound),
            Some(v) if v.is::<V>() => Ok(()),
            Some(_) => Err(Error::TypeMismatch),
        }
    }

    pub fn get<V>(&self, k: &K) -> Result<&V, Error>
    where
        V: 'static,
    {
        self.h
            .get(k)
            .ok_or(Error::KeyNotFound)
            .and_then(|b| b.downcast_ref::<V>().ok_or(Error::TypeMismatch))
    }

    pub fn get_clone<V>(&mut self, k: &K) -> Result<V, Error>
    where
        V: Clone + 'static,
    {
        self.get(k).map(|b: &V| (*b).clone())
    }

    pub fn insert<V>(&mut self, k: K, v: V) -> Result<Option<Box<V>>, (Error, V)>
    where
        V: 'static,
    {
        if let Some(prev) = self.h.get(&k) {
            if !prev.is::<V>() {
                return Err((Error::TypeMismatch, v));
            }
        }

        Ok(self
            .h
            .insert(k, Box::new(v))
            .map(|b| b.downcast::<V>().unwrap()))
    }

    pub fn remove<V>(&mut self, k: &K) -> Result<Box<V>, Error>
    where
        V: 'static,
    {
        let prev = self.h.get(k).ok_or(Error::KeyNotFound)?;
        if !prev.is::<V>() {
            return Err(Error::TypeMismatch);
        }
        drop(prev);

        Ok(self.h.remove(k).unwrap().downcast::<V>().unwrap())
    }
}

impl<K> Default for AnyMap<K>
where
    K: Eq + Hash,
{
    fn default() -> AnyMap<K> {
        AnyMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{AnyMap, Error};

    #[test]
    fn missing_key() {
        let mut m = AnyMap::<&'static str>::new();

        assert_eq!(false, m.contains_key(&"foo"));
        assert_eq!(Err(Error::KeyNotFound), m.contains_key_typed::<u32>(&"foo"));
        assert_eq!(Err(Error::KeyNotFound), m.get::<u32>(&"foo"));
        assert_eq!(Err(Error::KeyNotFound), m.remove::<u32>(&"foo"));
    }

    #[test]
    fn normal() {
        let mut m = AnyMap::<&'static str>::new();
        assert_eq!(Ok(None), m.insert::<u32>(&"foo", 1));
        assert_eq!(
            Ok(Some(1)),
            m.insert::<u32>(&"foo", 42).map(|r| r.map(|b| *b))
        );

        assert_eq!(true, m.contains_key(&"foo"));
        assert_eq!(Ok(()), m.contains_key_typed::<u32>(&"foo"));
        assert_eq!(Ok(42), m.get::<u32>(&"foo").map(|r| *r));
        assert_eq!(Ok(42), m.get_clone::<u32>(&"foo"));
        assert_eq!(Ok(42), m.remove::<u32>(&"foo").map(|b| *b));
        assert_eq!(Err(Error::KeyNotFound), m.remove::<u32>(&"foo"));
    }

    #[test]
    fn type_mismatch() {
        let mut m = AnyMap::<&'static str>::new();
        assert_eq!(Ok(None), m.insert::<u32>(&"foo", 42));
        assert_eq!(
            Err(Error::TypeMismatch),
            m.insert::<bool>(&"foo", true).map_err(|(err, _)| err)
        );

        assert_eq!(true, m.contains_key(&"foo"));
        assert_eq!(
            Err(Error::TypeMismatch),
            m.contains_key_typed::<bool>(&"foo")
        );
        assert_eq!(Err(Error::TypeMismatch), m.get::<bool>(&"foo"));
        assert_eq!(Err(Error::TypeMismatch), m.remove::<bool>(&"foo"));
    }
}
