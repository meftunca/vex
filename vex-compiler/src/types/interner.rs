use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I32,
    Bool,
    String,
    Vec(Arc<Type>),
    Named(String),
    Generic { name: String, args: Vec<Arc<Type>> },
    // Add other types as needed
}

pub struct TypeInterner {
    cache: DashMap<Type, Arc<Type>>,
}

impl TypeInterner {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn intern(&self, ty: Type) -> Arc<Type> {
        self.cache
            .entry(ty.clone())
            .or_insert_with(|| Arc::new(ty))
            .clone()
    }
}
