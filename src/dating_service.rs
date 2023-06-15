use std::sync::atomic::{AtomicU64, Ordering};

use block_id::{Alphabet, BlockId};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DateContent {
    who: String,
    what: String,
    shortdesc: String,
    longdesc: String,
    contact: String,
    password: String,
    pub action_type: Option<String>,
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
            action_type: None,
        }
    }

    pub fn new_placeholder() -> Self {
        DateContent {
            who: String::from("Dating Plattform"),
            what: String::from("Date"),
            shortdesc: String::from(
                "There is currently no date on this dating plattform feel free to post one",
            ),
            longdesc: String::from(""),
            contact: String::from(""),
            password: String::from("public"),
            action_type: None,
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Date {
    id: String,
    created: chrono::DateTime<Utc>,
    due: chrono::DateTime<Utc>,
    content: DateContent,
}
impl Date {
    pub fn new(id: String, content: DateContent, alive_in_days: i64) -> Self {
        Date {
            id,
            created: Utc::now(),
            due: Utc::now() + chrono::Duration::days(alive_in_days),
            content,
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub password: String,
    pub action_type: Option<String>,
}

fn validate_time(input: &str) -> Result<usize, String> {
    let days = match input.parse::<usize>() {
        Ok(days) => days,
        Err(_) => return Err(String::from("Time is not a number")),
    };
    if days > 21 {
        return Err(String::from("Come on! Dont overdo it"));
    }
    Ok(days)
}

pub struct DatingService {
    dates: RwLock<Vec<Date>>,
    current_id: AtomicU64,
    id_generator: BlockId<char>,
}

impl DatingService {
    pub fn new() -> DatingService {
        DatingService {
            dates: RwLock::new(Vec::new()),
            current_id: AtomicU64::new(0),
            id_generator: BlockId::new(Alphabet::lowercase_alphanumeric(), 9876, 5),
        }
    }

    pub async fn get_date(&self, id: &str) -> Result<Date, ()> {
        //let mut myiter = self.dates.read().await.iter().find(|date| date.id == id);
        for date in self.dates.read().await.iter() {
            if date.id == id {
                return Ok(date.clone());
            }
        }
        Err(())
    }

    pub async fn get_next_date_of(&self, id: Option<&str>) -> Date {
        let dates = self.dates.read().await;
        if dates.is_empty() {
            return Date::new("empty".to_string(), DateContent::new_placeholder(), 0);
        }

        let id = match id {
            None => return dates[0].clone(),
            Some(id) => id,
        };

        let mut filtered_date = dates.iter().fuse().skip_while(|date| date.id != id);
        filtered_date.next();
        let maybe_next_date = filtered_date.next();

        match maybe_next_date {
            Some(date) => date.clone(),
            None => dates[0].clone(),
        }
    }

    pub async fn list(&self) -> Vec<Date> {
        self.dates.read().await.to_vec()
    }

    pub fn find_date(
        &self,
        id: &str,
        password: String,
        vector_of_dates: &[Date],
    ) -> Result<usize, String> {
        let pos = match vector_of_dates.iter().position(|date| date.id == id) {
            Some(pos) => pos,
            None => return Err(String::from("Date does not exist!")),
        };
        if vector_of_dates[pos].content.password != password {
            Err(String::from("Password incorrect!"))
        } else {
            Ok(pos)
        }
    }

    pub async fn reset_timeout(
        &self,
        id: &str,
        password: String,
        days: String,
    ) -> Result<(), String> {
        let days = match validate_time(&days) {
            Ok(days) => days,
            Err(e) => return Err(e),
        };

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

    pub async fn add_date(&self, content: DateContent) -> Result<String, InputError> {
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
        let maybe_days = validate_time(&content.action_type.clone().unwrap());
        let mut days: i64 = 0;
        if maybe_days.is_err() {
            errors.push(maybe_days.err().unwrap())
        } else {
            days = maybe_days.unwrap() as i64;
        }
        if !errors.is_empty() {
            let e = InputError { content, errors };
            return Err(e);
        }
        let id = self
            .id_generator
            .encode_string(self.current_id.fetch_add(1, Ordering::AcqRel))
            .unwrap();
        let new_date = Date::new(id.clone(), content, days);
        self.dates.write().await.push(new_date);

        Ok(id)
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
                action_type: None,
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
        let output_date = dating_service.get_date(&new_uuid).await.unwrap();

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
            .reset_timeout(&new_uuid, String::from("public"), String::from("0"))
            .await
            .ok();
        assert!(dating_service.get_date(&new_uuid).await.is_err());
    }
}
