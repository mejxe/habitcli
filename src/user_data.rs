use core::panic;
use sled::{self};
use std::{collections::HashMap, fs};
use crate::error::{Error, Result};

pub struct UserData {
    pub token: String,
    pub name: String,
    pub sum_graph: Option<String>,
    pub graphs_to_sum: Option<Vec<String>>,
}

pub struct User {
    database: sled::Db,
}
impl User {
    pub fn new() -> User {
        User {
            database: sled::open(get_path()).expect("Path should be correct"),
        }
    }
}

fn get_path() -> String {
    let path = match directories::BaseDirs::new() {
        Some(path) => path,
        None => panic!("Unsupported OS"),
    };
    let path = path.config_dir().to_str().unwrap();
    format!("{path}/habitCLI").to_string()
}

fn create_config_dir() -> std::io::Result<()> {
    // creates directory to store habitcli files
    let path = get_path();
    fs::create_dir(&path)?;
    Ok(())
}

impl User {
    pub fn set_user_data(&self, name: &str, token: &str) -> Result<()> {
        // puts user specific data in the local database
        let _ = self.database.insert("token", token)?;
        let _ = self.database.insert("name", name)?;
        Ok(())
    }
    pub fn setup_sum_graph(&self, sum_graph_id: &str, graph_to_sum1: &str, graph_to_sum2: &str) -> Result<()> {
        /*!
        setups the graph summing system provided the name of the sum_graph and graphs to sum.
        only one sum_graph is supported
        */
        let _ = self.database.insert("sum_graph", sum_graph_id);
        let _ = self.database.insert("graph_to_sum1", graph_to_sum1);
        let _ = self.database.insert("graph_to_sum2", graph_to_sum2);
        Ok(())
    }

    pub fn get_user_data(&self) -> Result<UserData> {
        // gets all user local data from database and pass it in a standarized way
        let none_message = "Log in first. (habitcli login -h)";
        let token = if let Some(token_vector) = self.database.get("token")? {
            std::str::from_utf8(&token_vector).unwrap().to_string()
        } else {
            return Err(Error::from(none_message));
        };

        let name = if let Some(name_vector) = self.database.get("name")? {
            std::str::from_utf8(&name_vector).unwrap().to_string()
        } else {
            return Err(Error::from(none_message));
        };
        let sum_graph = if let Some(sum_graph_id) = self.database.get("sum_graph")? {
            Some(std::str::from_utf8(&sum_graph_id).unwrap().to_string())
        } else {
            None
        };
        let mut graphs_to_sum: Vec<String> = Vec::new();
        if let Some(graph_to_sum) = self.database.get("graph_to_sum1")? {
            graphs_to_sum.push(std::str::from_utf8(&graph_to_sum).unwrap().to_string());
        };
        if let Some(graph_to_sum2) = self.database.get("graph_to_sum2")? {
            graphs_to_sum.push(std::str::from_utf8(&graph_to_sum2).unwrap().to_string())
        };
        let mut graphs_result = None;
        if graphs_to_sum.len() == 2 {
            graphs_result = Some(graphs_to_sum.clone())
        };


        Ok(UserData { token, name, sum_graph, graphs_to_sum:graphs_result})
    }
    pub fn create_sum_graph_database(&self, data_to_sum: HashMap<String, String>) -> Result<()> {
        /*!
        crates sled database consisting of pixela graph names
        and corresponding daily commits for every graph that is supposed
        to be added together in a sum graph.
        database format:
        "graph_name" : "graph_commits"
        */
        for graph in data_to_sum {
            let val: &str = &graph.1;
            let _ = self.database.insert(graph.0, val)?;
        }
        Ok(())
    }


}

#[cfg(test)]
#[test]
fn create_dir_works() {
    create_config_dir().unwrap();
}
#[test]
fn test_getdata() {
    let user = User::new();
    let name = user.database.get("name").unwrap().unwrap();
    let name = std::str::from_utf8(&name);
    let token = user.database.get("token").unwrap().unwrap();
    let token = std::str::from_utf8(&token);
    println!("{:?}", name);
    println!("{:?}", token);
}
