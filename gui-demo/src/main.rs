//! DAM GUI Demo - Web-based Interface
//! 
//! A simple web interface for the Digital Asset Management system

use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult, middleware::Logger};
use actix_files::Files;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use serde_json::json;

use ingest::IngestService;
use index::IndexService;

#[derive(Clone)]
struct AppState {
    ingest: Arc<Mutex<IngestService>>,
    index: Arc<Mutex<IndexService>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting DAM Web GUI");
    
    // Initialize services
    let ingest_service = match IngestService::new() {
        Ok(service) => {
            info!("✅ Ingest service initialized");
            Arc::new(Mutex::new(service))
        }
        Err(e) => {
            error!("❌ Failed to initialize ingest service: {}", e);
            std::process::exit(1);
        }
    };
    
    let index_service = match IndexService::new() {
        Ok(service) => {
            info!("✅ Search index initialized");
            Arc::new(Mutex::new(service))
        }
        Err(e) => {
            error!("❌ Failed to initialize search service: {}", e);
            std::process::exit(1);
        }
    };
    
    let app_state = AppState {
        ingest: ingest_service,
        index: index_service,
    };
    
    info!("🌐 Starting web server on http://localhost:8080");
    
    // Use compile-time absolute path to static files - works from any directory
    let static_files = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
    
    info!("📁 Serving static files from: {}", static_files);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .route("/status", web::get().to(api_status))
                    .route("/search", web::get().to(api_search))
                    .route("/stats", web::get().to(api_stats))
                    .route("/import", web::post().to(api_import))
            )
            .service(Files::new("/", static_files).index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn api_status() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "online",
        "service": "DAM Digital Asset Manager",
        "version": "0.1.0"
    })))
}

async fn api_search(
    query: web::Query<SearchQuery>,
    state: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let q = query.q.as_deref().unwrap_or("");
    let limit = query.limit.unwrap_or(10);
    
    if q.is_empty() {
        return Ok(HttpResponse::Ok().json(json!({
            "results": [],
            "total": 0,
            "query": q
        })));
    }
    
    let index = state.index.lock().await;
    match index.search_text(q, limit).await {
        Ok(results) => {
            let search_results: Vec<_> = results.iter().map(|r| {
                json!({
                    "id": r.document.id,
                    "filename": r.document.filename,
                    "path": r.document.file_path,
                    "content": r.document.metadata.get("content").unwrap_or(&"".to_string()).chars().take(200).collect::<String>(),
                    "score": r.score
                })
            }).collect();
            
            Ok(HttpResponse::Ok().json(json!({
                "results": search_results,
                "total": results.len(),
                "query": q
            })))
        }
        Err(e) => {
            error!("Search failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Search failed",
                "message": e.to_string()
            })))
        }
    }
}

async fn api_stats(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let index = state.index.lock().await;
    let stats = index.get_stats();
    
    Ok(HttpResponse::Ok().json(json!({
        "total_documents": stats.total_documents,
        "visual_embeddings": stats.visual_embeddings,
        "text_embeddings": stats.text_embeddings,
        "last_updated": chrono::Utc::now().to_rfc3339()
    })))
}

async fn api_import(
    body: web::Json<ImportRequest>,
    state: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let path = std::path::PathBuf::from(&body.path);
    
    if !path.exists() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Path not found",
            "path": body.path
        })));
    }
    
    let mut ingest = state.ingest.lock().await;
    let mut index = state.index.lock().await;
    
    if path.is_dir() {
        // Handle directory import
        info!("Importing directory: {}", body.path);
        match ingest.ingest_directory(&path).await {
            Ok(assets) => {
                let mut imported_count = 0;
                let mut failed_count = 0;
                
                // Index all successfully ingested assets
                for asset in assets {
                    match index.index_asset(&asset).await {
                        Ok(_) => {
                            imported_count += 1;
                            info!("Successfully indexed: {}", asset.current_path.display());
                        }
                        Err(e) => {
                            failed_count += 1;
                            error!("Failed to index {}: {}", asset.current_path.display(), e);
                        }
                    }
                }
                
                info!("Directory import complete: {} imported, {} failed", imported_count, failed_count);
                Ok(HttpResponse::Ok().json(json!({
                    "success": true,
                    "type": "directory",
                    "path": body.path,
                    "imported_count": imported_count,
                    "failed_count": failed_count,
                    "message": format!("Imported {} assets from directory", imported_count)
                })))
            }
            Err(e) => {
                error!("Failed to ingest directory: {}", e);
                Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Failed to import directory",
                    "message": e.to_string(),
                    "path": body.path
                })))
            }
        }
    } else {
        // Handle single file import
        info!("Importing file: {}", body.path);
        match ingest.ingest_file(&path).await {
            Ok(asset) => {
                match index.index_asset(&asset).await {
                    Ok(_) => {
                        info!("Successfully imported and indexed: {}", body.path);
                        Ok(HttpResponse::Ok().json(json!({
                            "success": true,
                            "type": "file",
                            "asset_id": asset.id,
                            "asset_type": format!("{:?}", asset.asset_type),
                            "path": body.path,
                            "message": format!("Imported {:?} file", asset.asset_type)
                        })))
                    }
                    Err(e) => {
                        error!("Failed to index asset: {}", e);
                        Ok(HttpResponse::InternalServerError().json(json!({
                            "error": "Failed to index asset",
                            "message": e.to_string()
                        })))
                    }
                }
            }
            Err(e) => {
                error!("Failed to ingest file: {}", e);
                Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Failed to import file",
                    "message": e.to_string(),
                    "path": body.path
                })))
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<usize>,
}

#[derive(serde::Deserialize)]
struct ImportRequest {
    path: String,
}
