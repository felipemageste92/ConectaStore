//! Tipos de dados que representam os elementos e as relações da loja.

use std::fmt;

/// Identifica um vértice do grafo e, ao mesmo tempo, informa o seu tipo.
/// As variantes evitam que um cliente, produto e categoria com o mesmo número
/// sejam confundidos entre si.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Client(u64),
    Product(u64),
    Category(u64),
}

impl fmt::Display for NodeId {
    /// Converte o identificador para um texto legível, usado nas mensagens de erro.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client(id) => write!(f, "cliente:{id}"),
            Self::Product(id) => write!(f, "produto:{id}"),
            Self::Category(id) => write!(f, "categoria:{id}"),
        }
    }
}

/// Dados armazenados para cada cliente.
#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    pub id: u64,
    pub name: String,
}

/// Dados armazenados para cada produto.
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price: f64,
    pub available: bool,
}

/// Dados armazenados para cada categoria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: u64,
    pub name: String,
}

/// Agrupa todos os tipos que podem ocupar um vértice do grafo.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Client(Client),
    Product(Product),
    Category(Category),
}

impl Node {
    /// Extrai o identificador tipado independentemente da variante armazenada.
    pub fn id(&self) -> NodeId {
        match self {
            Self::Client(value) => NodeId::Client(value.id),
            Self::Product(value) => NodeId::Product(value.id),
            Self::Category(value) => NodeId::Category(value.id),
        }
    }
}

/// Tipos de aresta aceitos pelo domínio da loja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationType {
    Purchase,
    Interest,
    Rating,
    BelongsToCategory,
    Similarity,
}

impl RelationType {
    /// Retorna quanto cada tipo de relação influencia o cálculo da recomendação.
    pub fn score_factor(self) -> f64 {
        match self {
            Self::Purchase => 1.00,
            Self::Interest => 0.80,
            Self::Rating => 0.90,
            Self::BelongsToCategory => 0.60,
            Self::Similarity => 1.00,
        }
    }

    /// Fornece o motivo textual exibido junto da recomendação.
    pub fn description(self) -> &'static str {
        match self {
            Self::Purchase => "compras em comum",
            Self::Interest => "interesse relacionado",
            Self::Rating => "avaliacao relacionada",
            Self::BelongsToCategory => "mesma categoria",
            Self::Similarity => "produto similar",
        }
    }
}

/// Aresta direcionada: aponta para outro vértice e carrega tipo e intensidade.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub destination: NodeId,
    pub relation: RelationType,
    pub weight: f64,
}
