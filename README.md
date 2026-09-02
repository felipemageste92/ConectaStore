# ConectaStore — recomendação baseada em grafos

Projeto acadêmico em Rust que representa clientes, produtos e categorias como vértices. Compras, interesses, avaliações, categorias e similaridades são arestas direcionadas e ponderadas.

## Arquitetura

- `models.rs`: entidades, identificadores e tipos de relacionamento.
- `graph.rs`: grafo em memória, listas de adjacência e validações estruturais.
- `repository.rs`: fachada para cadastro e consulta.
- `recommendation.rs`: BFS limitado, pontuação, filtros e ordenação.
- `benchmark.rs`: gerador determinístico e medição simples com `Instant`.
- `error.rs`: erros de domínio retornados por `Result`.
- `main.rs`: demonstração automática reproduzível.

O grafo possui uma lista de saída principal (`HashMap<NodeId, Vec<Edge>>`) e um índice reverso. O índice reverso permite percorrer relações recebidas sem realizar uma varredura global, preservando o armazenamento direcionado.

## Pontuação

Cada passo multiplica a pontuação acumulada pelo peso da aresta, pelo fator do tipo de relação e pelo decaimento de distância. Produtos indisponíveis, já comprados ou iguais à origem são excluídos. O resultado usa pontuação decrescente e ID crescente como desempate.

## Executar

```bash
cargo build
cargo run
cargo test
cargo test --lib
cargo test --test graph_integration_test
cargo test --test recommendation_integration_test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

Execute os comandos dentro da pasta `megastore`.

## Benchmark simples

A função pública `benchmark::run_benchmark(produtos)` cria dados determinísticos e devolve os tempos de construção, consulta e recomendação. O exemplo executável mede 100, 1.000, 10.000 e 100.000 produtos:

```bash
cargo run --release --example performance
```

A saída está em CSV e contém volumes, vértices, arestas, candidatos e tempos. Trata-se de uma medição básica, não de um benchmark científico. Para comparar execuções, feche aplicações pesadas, use a mesma máquina e repita cada cenário.
