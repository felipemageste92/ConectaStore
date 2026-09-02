//! Biblioteca pública do ConectaStore.
//! Cada `mod` declara um módulo e os `pub use` oferecem uma API mais simples
//! para quem utiliza o crate, sem exigir o caminho interno completo.

// Expõe os módulos que formam as camadas da aplicação.
pub mod benchmark;
pub mod error;
pub mod graph;
pub mod models;
pub mod recommendation;
pub mod repository;

// Reexporta os principais tipos diretamente no namespace `megastore`.
pub use error::{ConectaStoreError, Result};
pub use graph::Graph;
pub use models::{Category, Client, Edge, Node, NodeId, Product, RelationType};
pub use recommendation::{Recommendation, RecommendationConfig, RecommendationEngine};
pub use repository::StoreRepository;
