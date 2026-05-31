use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub fn new(title: String, content: String) -> Result<Self, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        Ok(Self {
            id: None,
            title,
            content,
            created_at: Utc::now(),
        })
    }

    pub fn update(&mut self, title: Option<String>, content: Option<String>) -> Result<(), String> {
        if let Some(t) = title {
            if t.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            self.title = t;
        }
        if let Some(c) = content {
            self.content = c;
        }
        Ok(())
    }
}
