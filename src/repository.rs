use crate::error::{ConectaStoreError, Result};
use crate::graph::Graph;
use crate::models::{Category, Client, Node, NodeId, Product, RelationType};

#[derive(Debug, Default)]
pub struct StoreRepository {
    graph: Graph,
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
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
    fn consulta_produto_inexistente() {
        let repository = StoreRepository::new();
        let result = repository.product(404);
        assert_eq!(
            result,
            Err(ConectaStoreError::NodeNotFound(NodeId::Product(404)))
        );
    }

    #[test]
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn add_client(&mut self, id: u64, name: impl Into<String>) -> Result<()> {
        self.graph.add_node(Node::Client(Client {
            id,
            name: name.into(),
        }))
    }

    pub fn add_product(
        &mut self,
        id: u64,
        name: impl Into<String>,
        price: f64,
        available: bool,
    ) -> Result<()> {
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

    pub fn add_category(&mut self, id: u64, name: impl Into<String>) -> Result<()> {
        self.graph.add_node(Node::Category(Category {
            id,
            name: name.into(),
        }))
    }

    pub fn product(&self, id: u64) -> Result<&Product> {
        match self.graph.node(NodeId::Product(id))? {
            Node::Product(product) => Ok(product),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Product(id))),
        }
    }

    pub fn client(&self, id: u64) -> Result<&Client> {
        match self.graph.node(NodeId::Client(id))? {
            Node::Client(client) => Ok(client),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Client(id))),
        }
    }

    pub fn category(&self, id: u64) -> Result<&Category> {
        match self.graph.node(NodeId::Category(id))? {
            Node::Category(category) => Ok(category),
            _ => Err(ConectaStoreError::NodeNotFound(NodeId::Category(id))),
        }
    }

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
