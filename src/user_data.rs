use core::panic;
use sled::{self};
use std::{fmt::Display, fs, path::PathBuf};
use crate::error::{Error, Result, SumGraphError, SumGraphErrorKind};
use serde::{Deserialize, Serialize};

pub struct UserData {
    pub token: String,
    pub name: String,
    pub sum_graphs: Option<SumGraphsStruct>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SumGraphStruct {
    pub sum_graph_name: String,
    pub graphs_to_sum: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SumGraphsStruct {
    pub sum_graphs: Vec<SumGraphStruct>
}
    

impl SumGraphsStruct {

    pub fn build(mut sum_graphs: Vec<SumGraphStruct>) -> Result<SumGraphsStruct> {
        for i in 0..sum_graphs.len() {
            let mut j = i+1;
            while j < sum_graphs.len() {
                let mut k = 0;
                while sum_graphs[i].graphs_to_sum.contains(&sum_graphs[j].sum_graph_name) {
                    k += 1;
                    sum_graphs.swap(i, j);
                    if k>1 {return Err(Error::SumGraphError(SumGraphError::new(SumGraphErrorKind::GraphsSumEachOther)))}
                }
                j += 1;
            }

        }
        Ok(SumGraphsStruct { sum_graphs })
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


impl User {
    pub fn set_user_data(&self, name: &str, token: &str) -> Result<()> {
        // puts user specific data in the local database
        let _ = self.database.insert("token", token)?;
        let _ = self.database.insert("name", name)?;
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
}
impl Display for SumGraphStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names = String::new();
        self.graphs_to_sum.iter().for_each(|name| names.push_str(&format!("{name}, ")));

        // delete trailing comma
        names.pop();
        names.pop();

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

#[cfg(test)]
mod tests {
    use super::*;
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
#[test]
    fn sum_build() {
        let sumgraph_A = SumGraphStruct::new("A".to_string(), vec!["B","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_B = SumGraphStruct::new("B".to_string(), vec!["g","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_C = SumGraphStruct::new("C".to_string(), vec!["A","g","z"].iter().map(|s| s.to_string()).collect());
        let sum_graphs = vec![sumgraph_A.clone(), sumgraph_B.clone(), sumgraph_C.clone()];
        assert_eq!(SumGraphsStruct::build(sum_graphs).unwrap(), SumGraphsStruct{sum_graphs: vec![sumgraph_B, sumgraph_A, sumgraph_C]});

        let sumgraph_A = SumGraphStruct::new("A".to_string(), vec!["B","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_B = SumGraphStruct::new("B".to_string(), vec!["C","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_C = SumGraphStruct::new("C".to_string(), vec!["A","g","z"].iter().map(|s| s.to_string()).collect());
        let sum_graphs = vec![sumgraph_A.clone(), sumgraph_B.clone(), sumgraph_C.clone()];
        assert_eq!(SumGraphsStruct::build(sum_graphs).unwrap(), SumGraphsStruct{sum_graphs: vec![sumgraph_C, sumgraph_B, sumgraph_A]});
    }

#[test]
#[should_panic]
    fn sum_build_panic() {
        let sumgraph_A = SumGraphStruct::new("A".to_string(), vec!["C","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_B = SumGraphStruct::new("B".to_string(), vec!["g","x","z"].iter().map(|s| s.to_string()).collect());
        let sumgraph_C = SumGraphStruct::new("C".to_string(), vec!["A","g","z"].iter().map(|s| s.to_string()).collect());
        let sum_graphs = vec![sumgraph_A.clone(), sumgraph_B.clone(), sumgraph_C.clone()];
        SumGraphsStruct::build(sum_graphs).unwrap();
    }
}
