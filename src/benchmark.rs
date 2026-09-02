//! Geração de dados sintéticos e medição de desempenho do projeto.

use crate::error::Result;
use crate::models::{NodeId, RelationType};
use crate::recommendation::{RecommendationConfig, RecommendationEngine};
use crate::repository::StoreRepository;
use std::time::{Duration, Instant};

/// Reúne os volumes e tempos observados em uma execução do benchmark.
#[derive(Debug)]
pub struct BenchmarkReport {
    pub clients: usize,
    pub products: usize,
    pub vertices: usize,
    pub edges: usize,
    pub build_time: Duration,
    pub query_time: Duration,
    pub recommendation_time: Duration,
    pub candidates_found: usize,
}

/// Constrói sempre o mesmo grafo para um determinado número de produtos.
pub fn generate_synthetic_graph(products: usize) -> Result<StoreRepository> {
    // Define volumes proporcionais, garantindo ao menos clientes e uma categoria.
    let clients = (products / 10).max(10);
    let categories = (products / 100).max(1);
    let mut repository = StoreRepository::new();

    // Etapa 1: cadastra as categorias disponíveis.
    for category in 1..=categories {
        repository.add_category(category as u64, format!("Categoria {category}"))?;
    }

    // Etapa 2: cadastra produtos, categorias e similaridade com o item anterior.
    for product in 1..=products {
        let product_id = product as u64;
        let category_id = ((product - 1) % categories + 1) as u64;
        repository.add_product(product_id, format!("Produto {product}"), 10.0, true)?;
        repository.connect(
            NodeId::Product(product_id),
            NodeId::Category(category_id),
            RelationType::BelongsToCategory,
            1.0,
        )?;
        if product > 1 {
            repository.connect(
                NodeId::Product((product - 1) as u64),
                NodeId::Product(product_id),
                RelationType::Similarity,
                0.8,
            )?;
        }
    }

    // Etapa 3: cadastra clientes e três comportamentos reproduzíveis por cliente.
    for client in 1..=clients {
        let client_id = client as u64;
        repository.add_client(client_id, format!("Cliente {client}"))?;
        if products > 0 {
            let first = ((client * 7) % products + 1) as u64;
            let second = ((client * 13) % products + 1) as u64;
            let third = ((client * 17) % products + 1) as u64;
            repository.connect(
                NodeId::Client(client_id),
                NodeId::Product(first),
                RelationType::Purchase,
                1.0,
            )?;
            repository.connect(
                NodeId::Client(client_id),
                NodeId::Product(second),
                RelationType::Interest,
                0.7,
            )?;
            repository.connect(
                NodeId::Client(client_id),
                NodeId::Product(third),
                RelationType::Rating,
                0.9,
            )?;
        }
    }
    Ok(repository)
}

/// Mede separadamente a construção, uma consulta direta e uma recomendação.
pub fn measure_scenario(products: usize) -> Result<BenchmarkReport> {
    // Cronometra a criação de todos os vértices e arestas.
    let build_start = Instant::now();
    let repository = generate_synthetic_graph(products)?;
    let build_time = build_start.elapsed();

    // Cronometra a busca direta pelo último produto criado.
    let query_id = products.max(1) as u64;
    let query_start = Instant::now();
    let product = repository.product(query_id)?;
    std::hint::black_box(product);
    let query_time = query_start.elapsed();

    // Cronometra uma busca de recomendações a partir do primeiro cliente.
    let recommendation_start = Instant::now();
    let engine = RecommendationEngine::new(repository.graph());
    let recommendations = engine.for_client(
        1,
        RecommendationConfig {
            max_depth: 3,
            limit: products.max(1),
            distance_decay: 0.85,
        },
    )?;
    std::hint::black_box(&recommendations);
    let recommendation_time = recommendation_start.elapsed();

    // Consolida os contadores e durações em um relatório único.
    Ok(BenchmarkReport {
        clients: (products / 10).max(10),
        products,
        vertices: repository.graph().node_count(),
        edges: repository.graph().edge_count(),
        build_time,
        query_time,
        recommendation_time,
        candidates_found: recommendations.len(),
    })
}

/// Nome alternativo mantido como ponto de entrada público do benchmark.
pub fn run_benchmark(products: usize) -> Result<BenchmarkReport> {
    measure_scenario(products)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // O gerador determinístico deve repetir volumes e conteúdo para a mesma entrada.
    fn gerador_produz_volume_e_relacoes_reproduziveis() -> Result<()> {
        let first = generate_synthetic_graph(100)?;
        let second = generate_synthetic_graph(100)?;
        assert_eq!(first.graph().node_count(), second.graph().node_count());
        assert_eq!(first.graph().edge_count(), second.graph().edge_count());
        assert_eq!(first.product(100)?.name, "Produto 100");
        Ok(())
    }
}
