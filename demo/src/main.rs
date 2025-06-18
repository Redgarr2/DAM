//! DAM - Digital Asset Management System Demo
//! 
//! This demonstrates the core functionality of the DAM system.

use std::path::PathBuf;
use tokio;
use tracing::{error};
use tracing_subscriber;

use ingest::IngestService;
use index::IndexService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting DAM - Digital Asset Management System");
    println!("=================================================");
    
    // Initialize core services
    println!("\n📚 Initializing services...");
    
    let mut ingest_service = match IngestService::new() {
        Ok(service) => {
            println!("✅ Ingest service initialized");
            service
        }
        Err(e) => {
            println!("❌ Failed to initialize ingest service: {}", e);
            return Err(e.into());
        }
    };
    
    let mut index_service = match IndexService::new() {
        Ok(service) => {
            println!("✅ Search index initialized");
            service
        }
        Err(e) => {
            println!("❌ Failed to initialize search service: {}", e);
            return Err(e.into());
        }
    };
    
    // Create test directory and sample files
    println!("\n📁 Setting up test environment...");
    setup_test_environment().await?;
    
    // Demonstrate file ingestion
    println!("\n📥 Testing file ingestion...");
    demo_file_ingestion(&mut ingest_service, &mut index_service).await?;
    
    // Demonstrate search functionality
    println!("\n🔍 Testing search functionality...");
    demo_search_functionality(&index_service).await?;
    
    println!("\n🎉 DAM System Demo Complete!");
    println!("✅ All core systems are functional and ready to use");
    println!("\nNext steps:");
    println!("1. Add AI model files to 'models/' directory for AI features");
    println!("2. Use the library APIs in your own applications");
    println!("3. Add server functionality for LAN sharing");
    
    Ok(())
}

async fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
    // Create directories
    tokio::fs::create_dir_all("test_assets").await?;
    tokio::fs::create_dir_all("models").await?;
    
    // Create sample files for testing
    create_sample_files().await?;
    
    println!("✅ Test environment created in 'test_assets/' directory");
    Ok(())
}

async fn create_sample_files() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample text file
    tokio::fs::write(
        "test_assets/sample_text.txt",
        "This is a sample text file for testing the DAM system.\nIt contains multiple lines of text."
    ).await?;
    
    // Create a sample JSON file (simulating metadata)
    tokio::fs::write(
        "test_assets/sample_metadata.json",
        r#"{
    "title": "Sample Asset",
    "description": "A test asset for the DAM system",
    "tags": ["test", "sample", "demo"],
    "created_by": "DAM System",
    "version": "1.0"
}"#
    ).await?;
    
    // Create a sample CSV file
    tokio::fs::write(
        "test_assets/sample_data.csv",
        "id,name,type,size\n1,test_image.jpg,image,2048\n2,test_video.mp4,video,15360\n3,test_model.blend,3d,8192"
    ).await?;
    
    println!("✅ Sample files created for testing");
    Ok(())
}

async fn demo_file_ingestion(
    ingest_service: &mut IngestService, 
    index_service: &mut IndexService
) -> Result<(), Box<dyn std::error::Error>> {
    
    let test_files = vec![
        "test_assets/sample_text.txt",
        "test_assets/sample_metadata.json", 
        "test_assets/sample_data.csv"
    ];
    
    for file_path in test_files {
        match ingest_service.ingest_file(&PathBuf::from(file_path)).await {
            Ok(asset) => {
                println!("✅ Ingested: {} ({:?})", file_path, asset.asset_type);
                
                // Add to search index
                match index_service.index_asset(&asset).await {
                    Ok(_) => println!("   📇 Added to search index"),
                    Err(e) => println!("   ⚠️  Failed to index: {}", e),
                }
            }
            Err(e) => {
                println!("⚠️  Failed to ingest {}: {}", file_path, e);
            }
        }
    }
    
    Ok(())
}

async fn demo_search_functionality(index_service: &IndexService) -> Result<(), Box<dyn std::error::Error>> {
    let test_queries = vec![
        "sample",
        "test",
        "DAM system",
        "metadata"
    ];
    
    for query in test_queries {
        println!("🔍 Searching for: '{}'", query);
        
        match index_service.search_text(query, 5).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("   No results found");
                } else {
                    for (i, result) in results.iter().enumerate() {
                        println!("   {}. {} (score: {:.2})", 
                                i + 1, 
                                result.document.filename,
                                result.score);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Search failed: {}", e);
            }
        }
    }
    
    // Get search statistics
    let stats = index_service.get_stats();
    println!("\n📊 Search Index Statistics:");
    println!("   Total documents: {}", stats.total_documents);
    println!("   Visual embeddings: {}", stats.visual_embeddings);
    println!("   Text embeddings: {}", stats.text_embeddings);
    
    Ok(())
}
