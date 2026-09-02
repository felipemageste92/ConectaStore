//! Estrutura do grafo e operações básicas sobre vértices e arestas.

use crate::error::{ConectaStoreError, Result};
use crate::models::{Edge, Node, NodeId, RelationType};
use std::collections::HashMap;

/// Grafo direcionado armazenado por listas de adjacência.
///
/// Além dos vértices, são mantidos dois índices de arestas. Isso permite
/// consultar rapidamente tanto as conexões de saída quanto as de entrada.
#[derive(Debug, Default)]
pub struct Graph {
    // Tabela principal: associa cada identificador aos dados do vértice.
    nodes: HashMap<NodeId, Node>,
    // Para cada origem, guarda as arestas que saem dela.
    adjacency: HashMap<NodeId, Vec<Edge>>,
    // O indice reverso conserva a direcao original no campo destination.
    reverse_adjacency: HashMap<NodeId, Vec<Edge>>,
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::models::{Client, Product};

    /// Monta o menor grafo necessário para reutilização nos testes abaixo.
    fn graph_with_client_and_product() -> Result<Graph> {
        let mut graph = Graph::new();
        graph.add_node(Node::Client(Client {
            id: 1,
            name: "Ana".to_string(),
        }))?;
        graph.add_node(Node::Product(Product {
            id: 10,
            name: "Notebook".to_string(),
            price: 3000.0,
            available: true,
        }))?;
        Ok(graph)
    }

    #[test]
    // Confirma que uma conexão aparece nos dois índices do grafo.
    fn cria_aresta_valida_e_indice_reverso() -> Result<()> {
        let mut graph = graph_with_client_and_product()?;
        graph.connect(
            NodeId::Client(1),
            NodeId::Product(10),
            RelationType::Purchase,
            0.8,
        )?;
        assert_eq!(graph.outgoing(NodeId::Client(1))?.len(), 1);
        assert_eq!(graph.incoming(NodeId::Product(10))?.len(), 1);
        Ok(())
    }

    #[test]
    // Uma aresta não pode partir de um vértice que não foi cadastrado.
    fn rejeita_vertice_inexistente() -> Result<()> {
        let mut graph = graph_with_client_and_product()?;
        let result = graph.connect(
            NodeId::Client(99),
            NodeId::Product(10),
            RelationType::Purchase,
            1.0,
        );
        assert_eq!(
            result,
            Err(ConectaStoreError::NodeNotFound(NodeId::Client(99)))
        );
        Ok(())
    }

    #[test]
    // O peso deve ser finito e permanecer no intervalo de zero a um.
    fn rejeita_pesos_invalidos() -> Result<()> {
        for weight in [-0.1, 1.1, f64::INFINITY, f64::NAN] {
            let mut graph = graph_with_client_and_product()?;
            let result = graph.connect(
                NodeId::Client(1),
                NodeId::Product(10),
                RelationType::Purchase,
                weight,
            );
            assert!(matches!(result, Err(ConectaStoreError::InvalidWeight(_))));
        }
        Ok(())
    }

    #[test]
    // Repetir a mesma relação substitui o peso em vez de criar outra aresta.
    fn conexao_duplicada_atualiza_peso_sem_duplicar() -> Result<()> {
        let mut graph = graph_with_client_and_product()?;
        graph.connect(
            NodeId::Client(1),
            NodeId::Product(10),
            RelationType::Interest,
            0.4,
        )?;
        graph.connect(
            NodeId::Client(1),
            NodeId::Product(10),
            RelationType::Interest,
            0.9,
        )?;
        let outgoing = graph.outgoing(NodeId::Client(1))?;
        let incoming = graph.incoming(NodeId::Product(10))?;
        assert_eq!(outgoing.len(), 1);
        assert_eq!(incoming.len(), 1);
        assert_eq!(outgoing[0].weight, 0.9);
        assert_eq!(incoming[0].weight, 0.9);
        Ok(())
    }
}

impl Graph {
    /// Cria um grafo vazio usando os valores padrão dos mapas.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cadastra um vértice e prepara suas listas de entrada e saída.
    pub fn add_node(&mut self, node: Node) -> Result<()> {
        let id = node.id();
        // Impede que um cadastro existente seja sobrescrito silenciosamente.
        if self.nodes.contains_key(&id) {
            return Err(ConectaStoreError::NodeAlreadyExists(id));
        }
        self.nodes.insert(id, node);
        // Todo vértice começa sem conexões em ambos os índices.
        self.adjacency.insert(id, Vec::new());
        self.reverse_adjacency.insert(id, Vec::new());
        Ok(())
    }

    /// Consulta os dados de um vértice ou informa que ele não existe.
    pub fn node(&self, id: NodeId) -> Result<&Node> {
        self.nodes
            .get(&id)
            .ok_or(ConectaStoreError::NodeNotFound(id))
    }

    /// Verifica a existência de um identificador sem gerar erro.
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Retorna a quantidade total de vértices cadastrados.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Soma as listas de saída para contar cada aresta uma única vez.
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(Vec::len).sum()
    }

    /// Retorna somente as arestas que partem do vértice informado.
    pub fn outgoing(&self, id: NodeId) -> Result<&[Edge]> {
        self.ensure_node(id)?;
        Ok(self.adjacency.get(&id).map_or(&[], Vec::as_slice))
    }

    /// Retorna somente as arestas que chegam ao vértice informado.
    pub fn incoming(&self, id: NodeId) -> Result<&[Edge]> {
        self.ensure_node(id)?;
        Ok(self.reverse_adjacency.get(&id).map_or(&[], Vec::as_slice))
    }

    /// Valida e cria uma relação direcionada entre dois vértices.
    pub fn connect(
        &mut self,
        from: NodeId,
        to: NodeId,
        relation: RelationType,
        weight: f64,
    ) -> Result<()> {
        // As duas extremidades precisam existir antes da criação da aresta.
        self.ensure_node(from)?;
        self.ensure_node(to)?;
        Self::validate_weight(weight)?;
        Self::validate_relationship(from, to, relation)?;

        // Registra a mesma relação no índice normal e no índice reverso.
        self.upsert_edge(from, to, relation, weight, false);
        self.upsert_edge(to, from, relation, weight, true);
        Ok(())
    }

    /// Centraliza a validação de existência usada pelas consultas e conexões.
    fn ensure_node(&self, id: NodeId) -> Result<()> {
        if self.contains(id) {
            Ok(())
        } else {
            Err(ConectaStoreError::NodeNotFound(id))
        }
    }

    /// Aceita apenas pesos finitos entre 0,0 e 1,0, inclusive.
    fn validate_weight(weight: f64) -> Result<()> {
        if weight.is_finite() && (0.0..=1.0).contains(&weight) {
            Ok(())
        } else {
            Err(ConectaStoreError::InvalidWeight(weight))
        }
    }

    /// Aplica as regras de quais tipos de vértices podem usar cada relação.
    fn validate_relationship(from: NodeId, to: NodeId, relation: RelationType) -> Result<()> {
        let valid = match relation {
            RelationType::Purchase | RelationType::Interest | RelationType::Rating => {
                matches!((from, to), (NodeId::Client(_), NodeId::Product(_)))
            }
            RelationType::BelongsToCategory => {
                matches!((from, to), (NodeId::Product(_), NodeId::Category(_)))
            }
            RelationType::Similarity => {
                matches!((from, to), (NodeId::Product(_), NodeId::Product(_))) && from != to
            }
        };
        if valid {
            Ok(())
        } else {
            Err(ConectaStoreError::InvalidRelationship { from, to })
        }
    }

    /// Insere uma aresta nova ou atualiza o peso de uma relação já existente.
    fn upsert_edge(
        &mut self,
        key: NodeId,
        destination: NodeId,
        relation: RelationType,
        weight: f64,
        reverse: bool,
    ) {
        // Escolhe em qual dos dois índices a aresta será armazenada.
        let map = if reverse {
            &mut self.reverse_adjacency
        } else {
            &mut self.adjacency
        };
        let edges = map.entry(key).or_default();
        // A combinação destino + tipo da relação identifica uma aresta única.
        if let Some(edge) = edges
            .iter_mut()
            .find(|edge| edge.destination == destination && edge.relation == relation)
        {
            edge.weight = weight;
        } else {
            edges.push(Edge {
                destination,
                relation,
                weight,
            });
        }
    }
}
