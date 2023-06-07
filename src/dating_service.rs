use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DateContent {
    who: String,
    what: String,
    shortdesc: String,
    longdesc: String,
    contact: String,
    password: String,
}
#[derive(Debug)]
pub struct InputError {
    pub content: DateContent,
    pub errors: Vec<String>,
}

impl DateContent {
    pub fn new() -> Self {
        DateContent {
            who: String::from(""),
            what: String::from(""),
            shortdesc: String::from(""),
            longdesc: String::from(""),
            contact: String::from(""),
            password: String::from("public"),
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Date {
    id: Uuid,
    created: chrono::DateTime<Utc>,
    due: chrono::DateTime<Utc>,
    content: DateContent,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub password: String,
    pub action_type: Option<String>,
}

pub struct DatingService {
    dates: RwLock<Vec<Date>>,
}

impl DatingService {
    pub fn new() -> DatingService {
        DatingService {
            dates: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_date(&self, id: uuid::Uuid) -> Result<Date, ()> {
        for date in self.dates.read().await.iter() {
            if date.id == id {
                return Ok(date.clone());
            }
        }
        Err(())
    }

    pub async fn list(&self) -> Vec<Date> {
        self.dates.read().await.to_vec()
    }

    pub fn find_date(
        &self,
        id: uuid::Uuid,
        password: String,
        vector_of_dates: &[Date],
    ) -> Result<usize, String> {
        let pos = match vector_of_dates.iter().position(|date| date.id == id) {
            Some(pos) => pos,
            None => return Err(String::from("Date does not exist!")),
        };
        if vector_of_dates[pos].content.password != password {
            return Err(String::from("Password incorrect!"));
        } else {
            Ok(pos)
        }
    }

    pub async fn delete(&self, id: uuid::Uuid, password: String) -> Result<(), String> {
        let mut vector_of_dates = self.dates.write().await;
        match self.find_date(id, password, &vector_of_dates) {
            Ok(pos) => {
                vector_of_dates.remove(pos);
                Ok(())
            }
            Err(msg) => Err(msg),
        }
    }

    pub async fn reset_timeout(
        &self,
        id: uuid::Uuid,
        password: String,
        days: String,
    ) -> Result<(), String> {
        let days = match days.parse::<usize>() {
            Ok(days) => days,
            Err(_) => return Err(String::from("Time is not a number")),
        };
        if days > 21 {
            return Err(String::from("Come on! Dont overdo it"));
        }

        let mut vector_of_dates = self.dates.write().await;
        match self.find_date(id, password, &vector_of_dates) {
            Ok(pos) => {
                if days == 0 {
                    vector_of_dates.remove(pos);
                } else {
                    vector_of_dates[pos].due = Utc::now() + chrono::Duration::days(days as i64);
                }
                Ok(())
            }
            Err(msg) => Err(msg),
        }
    }

    pub async fn add_date(&self, content: DateContent) -> Result<Uuid, InputError> {
        let mut errors: Vec<String> = Vec::new();
        if content.who.len() < 2 {
            errors.push("Who must be at least 2 characters long".to_string());
        }
        if content.who.len() > 15 {
            errors.push("Who must be 15 characters or shorter".to_string());
        }
        if content.what.len() < 2 {
            errors.push("What must be at least 2 characters long".to_string());
        }
        if content.what.len() > 15 {
            errors.push("what must be 15 characters or shorter".to_string());
        }
        if content.shortdesc.len() < 10 {
            errors.push("Short description has to be at lesat 10 characters long".to_string());
        }
        if content.shortdesc.len() > 200 {
            errors.push("Short description must be 200 characters or shorter".to_string());
        }
        if errors.len() > 0 {
            let x = InputError { content, errors };
            return Err(x);
        }
        let new_id = Uuid::new_v4();
        let new_date = Date {
            id: new_id.clone(),
            created: Utc::now(),
            due: Utc::now() + chrono::Duration::days(7),
            content: content,
        };

        self.dates.write().await.push(new_date);

        Ok(new_id)
    }

    pub async fn clean_old_dates(&self) {
        let now = Utc::now();
        self.dates.write().await.retain(|v: &Date| v.due >= now)
    }
}

#[cfg(test)]
mod tests {

    use crate::dating_service::DatingService;

    use super::DateContent;

    impl DateContent {
        pub fn test_data() -> Self {
            DateContent {
                who: String::from("hacker"),
                what: String::from("self medication"),
                shortdesc: String::from("I am so depressed"),
                longdesc: String::from("I am so depressed. Please gimme something to sleep"),
                contact: String::from("intern@lists.metalab.at"),
                password: String::from("public"),
            }
        }
    }

    #[tokio::test]
    async fn it_works() {
        let dating_service = DatingService::new();
        let input_content = DateContent::test_data();
        let new_uuid = dating_service
            .add_date(input_content.clone())
            .await
            .unwrap();
        let output_date = dating_service.get_date(new_uuid).await.unwrap();

        assert_eq!(input_content.who, output_date.content.who);
    }

    #[tokio::test]
    async fn create_and_delete() {
        let dating_service = DatingService::new();
        let input_content = DateContent::test_data();
        let new_uuid = dating_service
            .add_date(input_content.clone())
            .await
            .unwrap();
        dating_service
            .delete(new_uuid, String::from("public"))
            .await
            .ok();
        assert!(dating_service.get_date(new_uuid).await.is_err());
    }
}
