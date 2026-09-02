//! Camada de acesso que oferece operações da loja sobre o grafo genérico.

use crate::error::{ConectaStoreError, Result};
use crate::graph::Graph;
use crate::models::{Category, Client, Node, NodeId, Product, RelationType};

/// Fachada que centraliza cadastros, consultas e conexões do domínio.
#[derive(Debug, Default)]
pub struct StoreRepository {
    graph: Graph,
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    // Percorre o fluxo completo de cadastro e leitura de um produto.
    fn cadastra_e_consulta_produto_valido() -> Result<()> {
        let mut repository = StoreRepository::new();
        repository.add_product(1, "Mouse", 99.90, true)?;
        let product = repository.product(1)?;
        assert_eq!(product.id, 1);
        assert_eq!(product.name, "Mouse");
        assert_eq!(product.price, 99.90);
        assert!(product.available);
        Ok(())
    }

    #[test]
    // Garante que o grafo preserve um cadastro já existente.
    fn rejeita_id_duplicado() -> Result<()> {
        let mut repository = StoreRepository::new();
        repository.add_product(1, "Mouse", 99.90, true)?;
        let result = repository.add_product(1, "Outro mouse", 120.0, true);
        assert_eq!(
            result,
            Err(ConectaStoreError::NodeAlreadyExists(NodeId::Product(1)))
        );
        Ok(())
    }

    #[test]
    // Confirma que uma consulta inválida devolve o erro esperado.
    fn consulta_produto_inexistente() {
        let repository = StoreRepository::new();
        let result = repository.product(404);
        assert_eq!(
            result,
            Err(ConectaStoreError::NodeNotFound(NodeId::Product(404)))
        );
    }

    #[test]
    // Verifica os outros dois tipos de vértice oferecidos pelo repositório.
    fn cadastra_cliente_e_categoria() -> Result<()> {
        let mut repository = StoreRepository::new();
        repository.add_client(1, "Ana")?;
        repository.add_category(2, "Tecnologia")?;
        assert_eq!(repository.client(1)?.name, "Ana");
        assert_eq!(repository.category(2)?.name, "Tecnologia");
        Ok(())
    }
}

impl StoreRepository {
    /// Cria um repositório contendo um grafo vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Empresta uma referência somente de leitura para consultas mais avançadas.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Converte o nome recebido e cadastra um novo cliente no grafo.
    pub fn add_client(&mut self, id: u64, name: impl Into<String>) -> Result<()> {
        self.graph.add_node(Node::Client(Client {
            id,
            name: name.into(),
        }))
    }

    /// Valida o preço e cadastra um novo produto.
    pub fn add_product(
        &mut self,
        id: u64,
        name: impl Into<String>,
        price: f64,
        available: bool,
    ) -> Result<()> {
        // Valores infinitos, NaN ou negativos não representam preços válidos.
        if !price.is_finite() || price < 0.0 {
            return Err(ConectaStoreError::InvalidPrice(price));
        }
        self.graph.add_node(Node::Product(Product {
            id,
            name: name.into(),
            price,
            available,
        }))
    }

    /// Cadastra uma categoria que poderá ser ligada aos produtos.
    pub fn add_category(&mut self, id: u64, name: impl Into<String>) -> Result<()> {
        self.graph.add_node(Node::Category(Category {
            id,
            name: name.into(),
        }))
    }

    /// Busca um produto e confirma que o vértice possui o tipo correto.
    pub fn product(&self, id: u64) -> Result<&Product> {
        match self.graph.node(NodeId::Product(id))? {
            Node::Product(product) => Ok(product),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Product(id))),
        }
    }

    /// Busca um cliente e confirma que o vértice possui o tipo correto.
    pub fn client(&self, id: u64) -> Result<&Client> {
        match self.graph.node(NodeId::Client(id))? {
            Node::Client(client) => Ok(client),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Client(id))),
        }
    }

    /// Busca uma categoria e confirma que o vértice possui o tipo correto.
    pub fn category(&self, id: u64) -> Result<&Category> {
        match self.graph.node(NodeId::Category(id))? {
            Node::Category(category) => Ok(category),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Category(id))),
        }
    }

    /// Delega ao grafo a validação e o armazenamento de uma relação.
    pub fn connect(
        &mut self,
        from: NodeId,
        to: NodeId,
        relation: RelationType,
        weight: f64,
    ) -> Result<()> {
        self.graph.connect(from, to, relation, weight)
    }
}
