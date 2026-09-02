use megastore::benchmark::measure_scenario;
use megastore::Result;

fn main() {
    if let Err(error) = run() {
        eprintln!("Falha na medicao: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("produtos,vertices,arestas,candidatos,construcao_ms,consulta_us,recomendacao_ms");
    for products in [100, 1_000, 10_000, 100_000] {
        let report = measure_scenario(products)?;
        println!(
            "{},{},{},{},{:.3},{:.3},{:.3}",
            report.products,
            report.vertices,
            report.edges,
            report.candidates_found,
            report.build_time.as_secs_f64() * 1_000.0,
            report.query_time.as_secs_f64() * 1_000_000.0,
            report.recommendation_time.as_secs_f64() * 1_000.0,
        );
    }
    Ok(())
}
