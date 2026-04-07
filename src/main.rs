mod db;

use db::Database;
use std::path::Path;

fn main() {
    let db_path = Path::new("state/cernio.db");

    // Ensure the state directory exists.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create state/ directory");
    }

    let db = Database::open(db_path).expect("failed to open database");

    let count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))
        .expect("failed to query companies");

    println!("Cernio database ready. {count} companies in universe.");
}
