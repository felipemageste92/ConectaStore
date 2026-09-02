//! Erros de domínio compartilhados por todas as camadas do projeto.

use crate::models::NodeId;
use std::error::Error;
use std::fmt;

/// Enumera as falhas esperadas ao manipular o grafo e as recomendações.
#[derive(Debug, Clone, PartialEq)]
pub enum ConectaStoreError {
    NodeAlreadyExists(NodeId),
    NodeNotFound(NodeId),
    InvalidRelationship { from: NodeId, to: NodeId },
    InvalidWeight(f64),
    InvalidPrice(f64),
    InvalidLimit,
    InvalidDepth,
}

impl fmt::Display for ConectaStoreError {
    /// Traduz cada variante em uma mensagem compreensível para o usuário.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeAlreadyExists(id) => write!(f, "vertice ja cadastrado: {id}"),
            Self::NodeNotFound(id) => write!(f, "vertice nao encontrado: {id}"),
            Self::InvalidRelationship { from, to } => {
                write!(f, "relacionamento invalido entre {from} e {to}")
            }
            Self::InvalidWeight(value) => write!(f, "peso invalido: {value}"),
            Self::InvalidPrice(value) => write!(f, "preco invalido: {value}"),
            Self::InvalidLimit => write!(f, "o limite deve ser maior que zero"),
            Self::InvalidDepth => write!(f, "a profundidade deve ser maior que zero"),
        }
    }
}

// Permite que o erro seja usado pelas APIs padrão de tratamento de erros do Rust.
impl Error for ConectaStoreError {}

/// Atalho para que as funções do projeto não precisem repetir o tipo de erro.
pub type Result<T> = std::result::Result<T, ConectaStoreError>;
