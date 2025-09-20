use std::fs;
use std::path::Path;
use SSRRR::algorithm::process::process::calculate;

fn main() {
    let test_dir = Path::new("assets");
    
    if !test_dir.exists() {
        eprintln!("Test directory not found: {:?}", test_dir);
        return;
    }
    
    // Read all .osu files in the Test directory
    let entries = match fs::read_dir(test_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            return;
        }
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("osu") {
            let file_path = path.to_string_lossy();
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            // Calculate star rating for this file
            let star_rating = calculate(&file_path, "None");
            println!("{} | {:.4}", file_name, star_rating);
        }
    }
}
