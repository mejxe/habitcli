use std::sync::Arc;

/*
 Api to communicate with Pixe.la web api
 */
use chrono::{self, NaiveDate};
use reqwest::Client;
use serde_json::{self, Value};
use tokio::sync::Mutex;
use crate::error::{self};


type Result<T> = error::Result<T>;

pub struct Session {
    client: Client,
}
impl Session {
    pub fn new() -> Session {
        let client = Client::new();
        Session {client}
    }

    pub(crate) async fn get_pixel_info(
        &self,
        url: &str,
        name: &str,
        date: Option<&str>,
        token: &str,
    ) -> Result<CallResult> {
        let client = &self.client;
        let date: &str = match date {
            Some(date) => date,
            None => &chrono::Local::now().format("%Y%m%d").to_string(),
        };
        let url = format!("{url}/{date}");

        let response = client.get(url).header("X-USER-TOKEN", token).send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;

        let quantity: u32 = match response.get("quantity") {
            Some(quantity) => quantity.as_str().unwrap().parse().unwrap(),
            None => 0,
        };
        Ok(CallResult::Heatmap(Heatmap::new(
            name.to_string(),
            date.to_string(),
            quantity,
        )))
    }
    pub(crate) async fn send_pixel(
        &self,
        url: &str,
        quantity: &str,
        date: Option<&str>,
        token: &str,
    ) -> Result<CallResult> {
        let client = &self.client;
        let date: &str = match date {
            Some(date) => date,
            None => &chrono::Local::now().format("%Y%m%d").to_string(),
        };
        let response = client
            .post(url)
            .header("X-USER-TOKEN", token)
            .json(&serde_json::json!({
                "date": date,
                "quantity": quantity
            }))
            .send();

        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;

        Ok(CallResult::ApiResponse(Message::new(response)))
    }
    pub(crate) async fn get_graph_list(
        &self,
        token: &str,
        url: &str
    ) -> Result<CallResult> {
        let client = &self.client;
        let response = client.get(url).header("X-USER-TOKEN", token).send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;
        let graphs = if let Some(graphs) = response.get("graphs") {graphs.to_owned()} else { return Err(error::Error::MissingEntryInDatabase("No graphs to display".to_string()))}; // dfq error handling
        let graphs: Vec<String> = graphs.as_array().unwrap()
            .iter()
            .map(|obj| obj.get("id").unwrap().to_string().replace("/", "").replace('"', "").to_string())
            .collect();
        Ok(CallResult::List(graphs))

       
    }
    pub async fn create_user(&self, user_specified_token: &str, username: &str, not_minor:bool, tos:bool) -> Result<()> {
        let client = &self.client;
            let url = format!("https://pixe.la/v1/users/");
            if !tos || !not_minor {
                return Err(error::Error::PixelaError(String::from("You didn't agree to TOS or you're a minor.")));
            }
        let response = client
            .post(url)
            .json(&serde_json::json!({
                "token": user_specified_token,
                "username": username,
                "agreeTermsOfService": "yes",
                "notMinor": "yes"
            }))
            .send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;
        match response.get("isSuccess") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        return Ok(())
    }
    pub async fn create_graph(&self, username: &str, token: &str, id: &str, name: &str, number_type: &str, unit: &str, color: &str) -> Result<()> {
        validate_args(color, number_type)?;
        let response = self.client
            .post(format!("https://pixe.la/v1/users/{}/graphs", username ))
            .header("X-USER-TOKEN", token)
            .json(&serde_json::json!(
                    {
                        "id": id,
                        "name": name,
                        "type": number_type,
                        "unit": unit,
                        "color": color,
                    }
                ))
            .send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::PixelaError(err.to_string()))?;
        match response.get("isSuccess") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        return Ok(());
    }
    pub async fn get_streak(&self, username: &str, token: &str, graph_name: &str) -> Result<u32>{
        let client = &self.client;
        let url = format!("https://pixe.la/v1/users/{}/graphs/{}/pixels", username, graph_name);

        let response = client.get(url).header("X-USER-TOKEN", token).query(&serde_json::json!(
                    {
                        "withBody": "true",
                    }
                )).send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;
        match response.get("pixels") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        Ok(calculate_streak(response))
    }
    pub async fn async_get_graph_val( 
        url: &str,
        date: &str,
        token: &str,
        incr_pointer: Arc<Mutex<u32>>,
    ) -> Result<()> {
        let client = Client::new();
        let url = format!("{url}/{date}");

        let response = client.get(url).header("X-USER-TOKEN", token).send().await;
        let response: serde_json::Value = response.unwrap().json().await.map_err(|err| error::Error::ReqwestError(err))?;

        let quantity: u32 = match response.get("quantity") {
            Some(quantity) => quantity.as_str().unwrap().parse::<u32>().map_err(|_| error::Error::PixelaError(response.to_string()))?,
            None => 0,
        };
        *incr_pointer.lock().await += quantity;
        Ok(())
    }
    pub async fn remove_graph(&self, username: &str, token: &str, graph_name: &str) -> Result<()> {
        let response = self.client
            .delete(format!("https://pixe.la/v1/users/{username}/graphs/{graph_name}"))
            .header("X-USER-TOKEN", token)
            .send();
        let response: serde_json::Value = response.await.unwrap().json().await.map_err(|err| error::Error::PixelaError(err.to_string()))?;
        match response.get("isSuccess") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        return Ok(());
    }

}

// Functions not tied to the Pixela web api below
pub fn calculate_streak(pixels: Value) -> u32 {
    // calculates streak and prints string visualizing the streak
    let pixels_array = pixels.get("pixels").unwrap().as_array().unwrap();
    if pixels_array.len() == 0 { return 0 }
    let mut streak = 0;

    let mut streak_going = false;
    let mut streak_valid = false;

    // loop to compare dates
    for i in 1..pixels_array.len() {
        let date1 = NaiveDate::parse_from_str(pixels_array[i-1].get("date").unwrap().as_str().unwrap(), "%Y%m%d").unwrap();
        let date2 = NaiveDate::parse_from_str(pixels_array[i].get("date").unwrap().as_str().unwrap(), "%Y%m%d").unwrap();
        let quantity1 = pixels_array[i-1].get("quantity").unwrap().as_str().unwrap();
        let quantity2 = pixels_array[i].get("quantity").unwrap().as_str().unwrap();
        let difference = (date2-date1).num_days();

        if difference > 1 || quantity1 == "0" || quantity2 == "0" {
            streak_going = false;
            streak = 0;
        }
        else if difference == 1 && streak_going {
            streak += 1
        }
        if !streak_going && quantity2 != "0" {
            streak += 1;
            streak_going = true;
        }
        
    }

    // Is the streak valid - if there wasn't any commits yesterday it is not
    let today: NaiveDate = (chrono::Local::now()).date_naive();
    let yesterday: NaiveDate = today-chrono::Days::new(1);
    let last_date = NaiveDate::parse_from_str(pixels_array.last().unwrap().get("date").unwrap().as_str().unwrap(), "%Y%m%d").unwrap();
    if last_date == today || last_date == yesterday {
        streak_valid = true;
    }

    if !streak_valid {
        streak = 0;
    };

    return streak;
}

pub fn prepare_streak_string(days: u32, graph_name: &str) -> String{
    let years = days/365;
    let months = (days - years*365) / 30; 
    let all_days = days - years*365 - months*30;
    format!("Your streak for the {graph_name} is {days} days long! (OR {years} year(s), {months} month(s) and {all_days} day(s).")
    
}

pub fn validate_args(color: &str, _type: &str) -> Result<()> {
    let valid_colors: [&str; 6] = ["shibafu", "momiji", "sora", "ichou", "ajisai", "kuro"];
    let valid_types: [&str;2] = ["int", "float"];
    if valid_colors.contains(&color) && valid_types.contains(&_type) {
        return Ok(());
    }
    else if !valid_colors.contains(&color) {
        return Err(error::Error::PixelaError("Wrong color name".to_string()))
    }
    else if !valid_types.contains(&_type) {
        return Err(error::Error::PixelaError("Wrong type".to_string()))
    }
    else {
        return Err(error::Error::PixelaError("Wrong color name and type".to_string()))
    }
}
// enums and structs

pub enum CallResult {
    // variants for each possible output of api communication functions
    ApiResponse(Message),
    Heatmap(Heatmap),
    List(Vec<String>),
}

#[derive(Debug)]
pub struct Message {
    json_message: serde_json::Value,
}
#[derive(Debug)]
pub struct Heatmap {
    name: String,
    quantity: u32,
    date: String,
}

impl Heatmap {
    pub fn new(name: String, date: String, quantity: u32) -> Heatmap {
        Heatmap {
            name,
            date,
            quantity,
        }
    }
    pub fn out_heatmap_info(&self) {
        println!(
            "Heatmap name: {}\nPixel date: {}\nCommits amount: {}",
            &self.name, &self.date, &self.quantity
        );
    }
}

impl Message {
    pub fn new(json_message: serde_json::Value) -> Message {
        Message { json_message }
    }

    pub fn out_message(&self) {
        let message = &self.json_message;
        if message.get("isSuccess").unwrap() == true {
            println!(
                "Success! API responded with: {}",
                message.get("message").unwrap()
            )
        } else {
            println!("API call failed: {}", message.get("message").unwrap())
        }
    }
}
