use core::panic;
use directories::ProjectDirs;
use sled::{self};
use std::{collections::HashMap, fmt::{write, Display}, fs, path::{Path, PathBuf}};
use crate::{ error::{Error, Result}};
use serde::{Deserialize, Serialize};

pub struct UserData {
    pub token: String,
    pub name: String,
    pub sum_graphs: Option<SumGraphsStruct>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SumGraphStruct {
    pub sum_graph_name: String,
    pub graphs_to_sum: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SumGraphsStruct {
    pub sum_graphs: Vec<SumGraphStruct>
}
impl Display for SumGraphStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names = String::new();
        self.graphs_to_sum.iter().for_each(|name| names.push_str(&format!("{name}, ")));
        write!(f, "____________________________\n\nSum Graph: {}\nGraphs: {}",self.sum_graph_name, names)
    }
}
impl Display for SumGraphsStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();
        for sum_graph in &self.sum_graphs {
            msg.push_str(&format!("{sum_graph}"));
            msg.push('\n');
        }
        write!(f, "{msg}")
    }
}
    

impl SumGraphsStruct {

    pub fn build(sum_graphs: Vec<SumGraphStruct>) -> SumGraphsStruct {
        SumGraphsStruct { sum_graphs }
    }

    pub fn save(&self) -> Result<()>{
        let path = get_path();
        dbg!(&path);
        if !path.exists() { fs::create_dir(&path)? }
        let toml_string = toml::to_string(self).unwrap();
        fs::write(path.join("sum_graph.toml"), toml_string)?;
        Ok(())
    }
    pub fn load() -> Result<Self> {
        let path = get_path();
        let new = toml::from_str(&fs::read_to_string(path.join("sum_graph.toml"))?);
        if let Ok(new) = new {
            return Ok(new);
        } else { return Err(Error::MissingEntryInDatabase("Failed loading local Sum Graphs".to_string())); }
        
    }
}
impl SumGraphStruct {
    pub fn new (sum_graph_name: String, graphs: Vec<String>) -> SumGraphStruct {
        SumGraphStruct { sum_graph_name, graphs_to_sum: graphs }
    }
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

fn get_path() ->  PathBuf {
    let path = match directories::BaseDirs::new() {
        Some(path) => path,
        None => panic!("Unsupported OS"),
    };
    path.config_dir().join("habitCLI")
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

        Ok(UserData { token, name, sum_graphs: None})
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
mod tests {
    use super::*;
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
    #[test]
    fn loading_graphs() {
        let graph = SumGraphsStruct::load();
        dbg!(graph.unwrap());
    }
}
