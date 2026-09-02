use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Client(u64),
    Product(u64),
    Category(u64),
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client(id) => write!(f, "cliente:{id}"),
            Self::Product(id) => write!(f, "produto:{id}"),
            Self::Category(id) => write!(f, "categoria:{id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price: f64,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Client(Client),
    Product(Product),
    Category(Category),
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Self::Client(value) => NodeId::Client(value.id),
            Self::Product(value) => NodeId::Product(value.id),
            Self::Category(value) => NodeId::Category(value.id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationType {
    Purchase,
    Interest,
    Rating,
    BelongsToCategory,
    Similarity,
}

impl RelationType {
    pub fn score_factor(self) -> f64 {
        match self {
            Self::Purchase => 1.00,
            Self::Interest => 0.80,
            Self::Rating => 0.90,
            Self::BelongsToCategory => 0.60,
            Self::Similarity => 1.00,
        }
    }

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

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub destination: NodeId,
    pub relation: RelationType,
    pub weight: f64,
}
