//! Busca em largura (BFS) e classificação das recomendações de produtos.

use crate::error::{ConectaStoreError, Result};
use crate::graph::Graph;
use crate::models::{Node, NodeId, RelationType};
use std::collections::{HashMap, HashSet, VecDeque};

/// Parâmetros que controlam até onde pesquisar e quantos resultados devolver.
#[derive(Debug, Clone, Copy)]
pub struct RecommendationConfig {
    /// Número máximo de arestas percorridas desde a origem.
    pub max_depth: usize,
    /// Quantidade máxima de recomendações no resultado.
    pub limit: usize,
    /// Redução aplicada ao score a cada novo passo do caminho.
    pub distance_decay: f64,
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::models::RelationType;
    use crate::repository::StoreRepository;

    /// Cria os vértices comuns usados pelos diferentes cenários de teste.
    fn base_repository() -> Result<StoreRepository> {
        let mut repository = StoreRepository::new();
        repository.add_client(1, "Ana")?;
        repository.add_category(1, "Tecnologia")?;
        for (id, name) in [
            (10, "Origem"),
            (20, "Forte"),
            (30, "Fraco"),
            (40, "Distante"),
        ] {
            repository.add_product(id, name, 100.0, true)?;
        }
        Ok(repository)
    }

    #[test]
    // Demonstra que a BFS alcança candidatos que não são vizinhos diretos.
    fn bfs_encontra_produto_em_dois_passos() -> Result<()> {
        let mut repository = base_repository()?;
        repository.connect(
            NodeId::Client(1),
            NodeId::Product(10),
            RelationType::Interest,
            1.0,
        )?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(20),
            RelationType::Similarity,
            0.9,
        )?;
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_client(1, RecommendationConfig::default())?;
        assert!(results.iter().any(|item| item.product_id == 20));
        Ok(())
    }

    #[test]
    // Um produto além da profundidade configurada não pode entrar no resultado.
    fn respeita_limite_de_profundidade() -> Result<()> {
        let mut repository = base_repository()?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(20),
            RelationType::Similarity,
            1.0,
        )?;
        repository.connect(
            NodeId::Product(20),
            NodeId::Product(40),
            RelationType::Similarity,
            1.0,
        )?;
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_product(
            10,
            RecommendationConfig {
                max_depth: 1,
                limit: 10,
                distance_decay: 0.85,
            },
        )?;
        assert!(results.iter().any(|item| item.product_id == 20));
        assert!(!results.iter().any(|item| item.product_id == 40));
        Ok(())
    }

    #[test]
    // O conjunto de visitados impede laços infinitos e resultados duplicados.
    fn ciclo_termina_sem_repetir_resultados() -> Result<()> {
        let mut repository = base_repository()?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(20),
            RelationType::Similarity,
            1.0,
        )?;
        repository.connect(
            NodeId::Product(20),
            NodeId::Product(10),
            RelationType::Similarity,
            1.0,
        )?;
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_product(10, RecommendationConfig::default())?;
        assert_eq!(
            results.iter().filter(|item| item.product_id == 20).count(),
            1
        );
        assert!(!results.iter().any(|item| item.product_id == 10));
        Ok(())
    }

    #[test]
    // Produtos que o cliente já comprou são usados como lista de exclusão.
    fn exclui_produto_ja_comprado() -> Result<()> {
        let mut repository = base_repository()?;
        repository.connect(
            NodeId::Client(1),
            NodeId::Product(10),
            RelationType::Purchase,
            1.0,
        )?;
        repository.connect(
            NodeId::Client(1),
            NodeId::Product(20),
            RelationType::Interest,
            1.0,
        )?;
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_client(1, RecommendationConfig::default())?;
        assert!(!results.iter().any(|item| item.product_id == 10));
        assert!(results.iter().any(|item| item.product_id == 20));
        Ok(())
    }

    #[test]
    // A ordenação prioriza score e usa o ID como desempate determinístico.
    fn ordena_por_score_e_depois_por_id() -> Result<()> {
        let mut repository = base_repository()?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(30),
            RelationType::Similarity,
            0.5,
        )?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(20),
            RelationType::Similarity,
            0.9,
        )?;
        repository.connect(
            NodeId::Product(10),
            NodeId::Product(40),
            RelationType::Similarity,
            0.5,
        )?;
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_product(10, RecommendationConfig::default())?;
        let ids: Vec<u64> = results.iter().map(|item| item.product_id).collect();
        assert_eq!(ids, vec![20, 30, 40]);
        Ok(())
    }

    #[test]
    // O corte final respeita a quantidade máxima pedida pelo chamador.
    fn limita_quantidade_de_resultados() -> Result<()> {
        let mut repository = base_repository()?;
        for id in [20, 30, 40] {
            repository.connect(
                NodeId::Product(10),
                NodeId::Product(id),
                RelationType::Similarity,
                0.8,
            )?;
        }
        let engine = RecommendationEngine::new(repository.graph());
        let results = engine.for_product(
            10,
            RecommendationConfig {
                max_depth: 2,
                limit: 2,
                distance_decay: 0.85,
            },
        )?;
        assert_eq!(results.len(), 2);
        Ok(())
    }

    #[test]
    // Um grafo sem relações válidas produz uma lista vazia, não um erro.
    fn retorna_vazio_quando_nao_ha_recomendacoes() -> Result<()> {
        let repository = base_repository()?;
        let engine = RecommendationEngine::new(repository.graph());
        assert!(engine
            .for_client(1, RecommendationConfig::default())?
            .is_empty());
        Ok(())
    }
}

impl Default for RecommendationConfig {
    /// Oferece valores equilibrados para chamadas que não exigem personalização.
    fn default() -> Self {
        Self {
            max_depth: 3,
            limit: 5,
            distance_decay: 0.85,
        }
    }
}

/// Resultado pronto para apresentação ao usuário.
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub product_id: u64,
    pub name: String,
    pub category: String,
    pub score: f64,
    pub reason: String,
}

/// Estado parcial colocado na fila durante a busca em largura.
#[derive(Debug, Clone)]
struct SearchState {
    node: NodeId,
    depth: usize,
    score: f64,
}

/// Melhor caminho encontrado até um produto candidato.
#[derive(Debug, Clone)]
struct Candidate {
    score: f64,
    reason: &'static str,
}

/// Serviço de recomendação que consulta um grafo sem modificá-lo.
pub struct RecommendationEngine<'a> {
    graph: &'a Graph,
}

impl<'a> RecommendationEngine<'a> {
    /// Associa o motor ao grafo que será pesquisado.
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// Recomenda para um cliente, excluindo todos os produtos já comprados.
    pub fn for_client(
        &self,
        client_id: u64,
        config: RecommendationConfig,
    ) -> Result<Vec<Recommendation>> {
        // Valida a origem e prepara a lista de produtos que não podem reaparecer.
        let origin = NodeId::Client(client_id);
        self.ensure_origin(origin)?;
        let purchased = self.purchased_products(origin)?;
        self.search(origin, &purchased, config)
    }

    /// Recomenda itens relacionados a um produto, sem sugerir o próprio produto.
    pub fn for_product(
        &self,
        product_id: u64,
        config: RecommendationConfig,
    ) -> Result<Vec<Recommendation>> {
        // A lista de exclusão começa contendo apenas o produto de origem.
        let origin = NodeId::Product(product_id);
        self.ensure_origin(origin)?;
        let mut excluded = HashSet::new();
        excluded.insert(product_id);
        self.search(origin, &excluded, config)
    }

    /// Executa a BFS, reúne candidatos e produz o ranking final.
    fn search(
        &self,
        origin: NodeId,
        excluded: &HashSet<u64>,
        config: RecommendationConfig,
    ) -> Result<Vec<Recommendation>> {
        // Etapa 1: valida os parâmetros e inicializa a fila com a origem.
        Self::validate_config(config)?;
        let mut queue = VecDeque::from([SearchState {
            node: origin,
            depth: 0,
            score: 1.0,
        }]);
        // Os conjuntos evitam revisitar vértices e duplicar produtos candidatos.
        let mut visited = HashSet::from([origin]);
        let mut unique_products = HashSet::new();
        let mut candidates: HashMap<u64, Candidate> = HashMap::new();

        // Etapa 2: retira os estados na ordem de chegada, característica da BFS.
        while let Some(state) = queue.pop_front() {
            if state.depth >= config.max_depth {
                continue;
            }
            // Explora tanto saídas quanto entradas para navegar relações direcionadas.
            self.explore_edges(
                &state,
                self.graph.outgoing(state.node)?,
                config,
                excluded,
                &mut visited,
                &mut unique_products,
                &mut candidates,
                &mut queue,
            )?;
            self.explore_edges(
                &state,
                self.graph.incoming(state.node)?,
                config,
                excluded,
                &mut visited,
                &mut unique_products,
                &mut candidates,
                &mut queue,
            )?;
        }

        // Etapa 3: completa os dados, ordena pelo melhor score e aplica o limite.
        let mut results = self.materialize(candidates)?;
        results.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.product_id.cmp(&right.product_id))
        });
        results.truncate(config.limit);
        Ok(results)
    }

    #[allow(clippy::too_many_arguments)]
    /// Avalia uma lista de arestas e atualiza candidatos e próximos estados.
    fn explore_edges(
        &self,
        state: &SearchState,
        edges: &[crate::models::Edge],
        config: RecommendationConfig,
        excluded: &HashSet<u64>,
        visited: &mut HashSet<NodeId>,
        unique_products: &mut HashSet<u64>,
        candidates: &mut HashMap<u64, Candidate>,
        queue: &mut VecDeque<SearchState>,
    ) -> Result<()> {
        for edge in edges {
            // O score acumula força da aresta, relevância da relação e distância.
            let depth = state.depth + 1;
            let score =
                state.score * edge.weight * edge.relation.score_factor() * config.distance_decay;
            let reason = edge.relation.description();

            // Somente produtos disponíveis e não excluídos viram candidatos.
            if let NodeId::Product(id) = edge.destination {
                if !excluded.contains(&id) && self.product_is_available(id)? {
                    // Para caminhos repetidos, conserva apenas a maior pontuação.
                    if unique_products.insert(id) {
                        candidates.insert(id, Candidate { score, reason });
                    } else if let Some(candidate) = candidates.get_mut(&id) {
                        if score > candidate.score {
                            candidate.score = score;
                            candidate.reason = reason;
                        }
                    }
                }
            }

            // Enfileira um destino inédito somente se ainda puder ser expandido.
            if depth < config.max_depth && visited.insert(edge.destination) {
                queue.push_back(SearchState {
                    node: edge.destination,
                    depth,
                    score,
                });
            }
        }
        Ok(())
    }

    /// Coleta os IDs ligados ao cliente por uma relação de compra.
    fn purchased_products(&self, client: NodeId) -> Result<HashSet<u64>> {
        Ok(self
            .graph
            .outgoing(client)?
            .iter()
            .filter(|edge| edge.relation == RelationType::Purchase)
            .filter_map(|edge| match edge.destination {
                NodeId::Product(id) => Some(id),
                _ => None,
            })
            .collect())
    }

    /// Troca candidatos internos por objetos completos para apresentação.
    fn materialize(&self, candidates: HashMap<u64, Candidate>) -> Result<Vec<Recommendation>> {
        candidates
            .into_iter()
            .map(|(id, candidate)| {
                let product = match self.graph.node(NodeId::Product(id))? {
                    Node::Product(product) => product,
                    _ => return Err(ConectaStoreError::NodeNotFound(NodeId::Product(id))),
                };
                Ok(Recommendation {
                    product_id: id,
                    name: product.name.clone(),
                    category: self.category_name(id)?,
                    score: candidate.score,
                    reason: candidate.reason.to_string(),
                })
            })
            .collect()
    }

    /// Procura a categoria ligada ao produto ou devolve um texto padrão.
    fn category_name(&self, product_id: u64) -> Result<String> {
        for edge in self.graph.outgoing(NodeId::Product(product_id))? {
            if edge.relation == RelationType::BelongsToCategory {
                if let Node::Category(category) = self.graph.node(edge.destination)? {
                    return Ok(category.name.clone());
                }
            }
        }
        Ok("Sem categoria".to_string())
    }

    /// Consulta a flag que informa se um produto ainda pode ser recomendado.
    fn product_is_available(&self, id: u64) -> Result<bool> {
        match self.graph.node(NodeId::Product(id))? {
            Node::Product(product) => Ok(product.available),
            _ => Ok(false),
        }
    }

    /// Garante que o cliente ou produto escolhido como origem existe no grafo.
    fn ensure_origin(&self, id: NodeId) -> Result<()> {
        self.graph.node(id).map(|_| ())
    }

    /// Impede configurações que tornariam a busca inválida ou sem resultados.
    fn validate_config(config: RecommendationConfig) -> Result<()> {
        if config.limit == 0 {
            return Err(ConectaStoreError::InvalidLimit);
        }
        if config.max_depth == 0 {
            return Err(ConectaStoreError::InvalidDepth);
        }
        if !config.distance_decay.is_finite() || !(0.0..=1.0).contains(&config.distance_decay) {
            return Err(ConectaStoreError::InvalidWeight(config.distance_decay));
        }
        Ok(())
    }
}
