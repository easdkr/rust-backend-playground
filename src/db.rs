use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
use crate::infrastructure::persistence::seaorm::model as post;

pub async fn connect(db_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(db_url).await?;
    
    // Set up tables if they don't exist
    setup_schema(&db).await?;
    
    Ok(db)
}

async fn setup_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    
    let mut stmt = schema.create_table_from_entity(post::Entity);
    stmt.if_not_exists();
    
    db.execute(builder.build(&stmt)).await?;
    
    Ok(())
}
