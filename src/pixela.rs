/*
 Api to communicate with Pixe.la web api
 */
use chrono;
use reqwest::blocking;
use serde_json;
use crate::error;


type Result<T> = error::Result<T>;

pub struct Session {
    client: blocking::Client,
}
impl Session {
    pub fn new() -> Session {
        let client = blocking::Client::new();
        Session {client}
    }

    pub(crate) fn get_pixel_info(
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
        let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::ReqwestError(err))?;

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
    pub(crate) fn send_pixel(
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

        let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::ReqwestError(err))?;

        Ok(CallResult::ApiResponse(Message::new(response)))
    }
    pub(crate) fn get_graph_list(
        &self,
        token: &str,
        url: &str
    ) -> Result<CallResult> {
        let client = &self.client;
        let response = client.get(url).header("X-USER-TOKEN", token).send();
        let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::ReqwestError(err))?;
        let graphs = if let Some(graphs) = response.get("graphs") {graphs.to_owned()} else { return Err(error::Error::MissingEntryInDatabase("No graphs to display".to_string()))}; // dfq error handling
        let graphs: Vec<String> = graphs.as_array().unwrap()
            .iter()
            .map(|obj| obj.get("id").unwrap().to_string())
            .collect();
        Ok(CallResult::List(graphs))

       
    }
    pub fn get_graphs_to_sum_commits(&self, graph_names: Vec<String>, token: &str, username: &str, date: Option<&str> ) -> Result<String> {
        let date: &str = match date {
            Some(date) => date,
            None => &chrono::Local::now().format("%Y%m%d").to_string(),
        };
        let mut commits_sum: u32 = 0;
        let client = &self.client;
        for graph in graph_names {
            let url = format!("https://pixe.la/v1/users/{username}/graphs/{graph}/{date}");
            let response = client.get(url).header("X-USER-TOKEN", token).send();
            let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::ReqwestError(err))?;
            let value: u32 = match response.get("quantity") {
                Some(quantity) => quantity.as_str().unwrap().parse().unwrap(),
                None => 0,
            };
            commits_sum += value as u32
            
        }
        Ok(commits_sum.to_string())
    }
    pub fn create_user(&self, user_specified_token: &str, username: &str, not_minor:bool, tos:bool) -> Result<()> {
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
        let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::ReqwestError(err))?;
        match response.get("isSuccess") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        return Ok(())
    }
    pub fn create_graph(&self, username:&str, token:&str, id: &str, name: &str, number_type: &str, unit: &str, color: &str) -> Result<()> {
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
        let response: serde_json::Value = response.unwrap().json().map_err(|err| error::Error::PixelaError(err.to_string()))?;
        match response.get("isSuccess") {
            None => return Err(error::Error::PixelaError(response.get("message").unwrap().to_string())),
            Some(message) => {if message == false {return Err(error::Error::PixelaError(response.get("message").unwrap().to_string()))}}
        }
        return Ok(());
    }
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


    /* TODO in the future
    pub(crate) async fn async_send(
        &self,
        url: &str,
        quantity: &str,
        date: Option<&str>,
        token: &str
        ) -> Result<()>{
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
            .send()
            .await
            .map_err(|err| Error::ReqwestError(err))?;

        let response: serde_json::Value = response.json().await.map_err(|err| Error::ReqwestError(err))?;
        return Ok(())
    }
    */
        

}
pub enum CallResult {
    // variants for each possible output of api communication functions
    ApiResponse(Message),
    Heatmap(Heatmap),
    List(Vec<String>),
}
pub struct SumGraphSystem {
    pub sum_amount: u32,
}
impl SumGraphSystem {
    pub fn new() -> SumGraphSystem {
        SumGraphSystem { sum_amount: 0 }
    }
    pub fn sum_graph_data(&mut self, graph_to_sum1_data: String, graph_to_sum2_data: String) {
        let graph_to_sum1_data: u32 = graph_to_sum1_data.parse().unwrap();
        let graph_to_sum2_data: u32 = graph_to_sum2_data.parse().unwrap();
        self.sum_amount = graph_to_sum1_data+graph_to_sum2_data;

    }

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
