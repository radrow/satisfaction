pub type BuildHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;

pub type IndexSet<V> = indexmap::IndexSet<V, BuildHasher>;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;

pub type HashSet<V> = rustc_hash::FxHashSet<V>;
pub type HashMap<K,V> = rustc_hash::FxHashMap<K,V>;

pub type PriorityQueue<P, V> = priority_queue::PriorityQueue<V, P, BuildHasher>;
