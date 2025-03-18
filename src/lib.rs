pub mod args;
pub mod pixela;
pub mod user_data;
pub mod error;

use std::{fmt::Display, io::stdin, sync::Arc};

use error::{Error, Result, SumGraphError, SumGraphErrorKind};

use args::{CreateGraphArgs, LoginArgs, NewUserArgs, PixelArgs, RemoveArgs, StreakGetArgs, SumArgs, SumGraphArgs};
use pixela::*;
use tokio::{sync::Mutex, task::JoinHandle};
use user_data::{SumGraphStruct, SumGraphsStruct};
pub struct Worker {
    /*
    Worker struct that calls all the functions 
    to keep main clenaer.
     */
    session: Session,
    api_key: Option<String>,
    name: Option<String>,
    sum_graphs: Option<SumGraphsStruct>,
}
impl Display for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Username: {}\nSum Graphs Info:\n{}", self.name.as_ref().unwrap(), self.sum_graphs.as_ref().unwrap()) 
        }
    }
impl Worker {
    pub fn new(session: Session) -> Worker {
        Worker {
            session,
            api_key: None,
            name: None,
            sum_graphs: None
        }
    }
    pub fn login(&mut self) -> Result<()> {
        // gets data from local database and saves it in the struct
        let user = user_data::User::new();
        let data = user.get_user_data()?;
        let (name, token) = (data.name, data.token);
        self.api_key = Some(token);
        self.name = Some(name);
        self.sum_graphs =  SumGraphsStruct::load().ok();
        
        Ok(())
    }

    pub async fn call_send(&self, args: PixelArgs<'_>) {
        let graph = args.graph;
        let date = args.date;
        let quantity = &args.quantity;
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = &self.create_url_graph(graph, &name);
        match &self
            .session
            .send_pixel(url, quantity, date.as_deref(), &api_key).await
        {
            Ok(call_result) => {
                if let CallResult::ApiResponse(msg) = call_result {
                    msg.out_message();
                }
            }
            Err(e) => println!("There was an error. {:?}", e),
        };
    }
    pub async fn handle_sum_graph(&self, args: SumArgs<'_>) -> Result<()>{
        let name = self.name.clone().expect("Should be logged in");
        let api_key = self.api_key.clone().expect("Should be logged in");
        let graphs = if let Some(graphs) = self.sum_graphs.as_ref() {
            graphs
        } else { return Err(Error::MissingEntryInDatabase("Sum graphs are not properly set up".to_string())) };
        let commits = Arc::new(Mutex::new(0));
        let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();
        let date: String = match args.date {
            Some(date) => date.to_string(),
            None => chrono::Local::now().format("%Y%m%d").to_string(),
        };
        let mut done_anything = false;

        for graph in &graphs.sum_graphs {

            if let Some(specified_name) = args.name {
                if graph.sum_graph_name != specified_name {
                    continue;
                }
            };

            for graph_name in &graph.graphs_to_sum {
                let name = name.clone();
                let api_key = api_key.clone();
                let date = date.clone();
                let commits = Arc::clone(&commits);
                let graph_name = graph_name.clone();
                let handle = tokio::spawn(async move {
                    let url = format!("https://pixe.la/v1/users/{}/graphs/{}",name, graph_name);
                    Session::async_get_graph_val(&url, &date, &api_key.clone(), commits.clone()).await?;
                    Ok(())
                });
                tasks.push(handle);
            }   
            for _ in 0..tasks.len() {
                let popped = tasks.pop().unwrap();
                popped.await.unwrap()?;
            }
            let url = format!("https://pixe.la/v1/users/{}/graphs/{}",name, graph.sum_graph_name);
            let sendable_commits = commits.lock().await.to_string();
            self.session.send_pixel(&url, &sendable_commits, args.date, &api_key).await?;
            println!("Summed {}.", graph.sum_graph_name);
            *commits.lock().await = 0;
            done_anything = true;
        }

        if !done_anything {
            return Err(Error::SumGraphError(SumGraphError::new(SumGraphErrorKind::GraphNotFoundLocally)));
        }
        println!("Success! Your Sum Graph has been updated.");
        return Ok(());
    }
        

    pub async fn call_get(&self, args: PixelArgs<'_>) {
        let graph = args.graph;
        let date = args.date;
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = &self.create_url_graph(graph, &name);
        match &self
            .session
            .get_pixel_info(url, graph, date.as_deref(), &api_key).await
        {
            Ok(call_result) => {
                if let CallResult::Heatmap(heatmap) = call_result {
                    heatmap.out_heatmap_info();
                }
            }
            Err(e) => println!("There was an error. {:?}", e),
        };
    }
    fn create_url_graph(&self, graph: &str, name: &str) -> String {
        format!("https://pixe.la/v1/users/{name}/graphs/{graph}")
    }
    pub fn call_save_data(&self, args: LoginArgs) -> Result<()> {
        let user = user_data::User::new();
        Ok(user.set_user_data(args.name, args.api_key)?)
    }
    pub async fn setup_graphs(&self, args: SumGraphArgs) -> Result<()> {
        let mut sum_graphs: Vec<SumGraphStruct> = vec![];
        let mut sum_graph_names_duplicate_tracker: Vec<String> = vec![];

        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = format!("https://pixe.la/v1/users/{name}/graphs/");
        let correct_names = self.session.get_graph_list(api_key, &url).await;
        let correct_names = if let Ok(CallResult::List(list)) = correct_names {
            list
        } else { return Err(Error::MissingEntryInDatabase("Unable to verify graph names, possibly graphs are non-existent".to_string()))};
        input_graph_names(&mut sum_graphs, &mut sum_graph_names_duplicate_tracker, args.sum_graph_amount, &correct_names)?;
            
            
        SumGraphsStruct::build(sum_graphs)?.save()?;
        println!("Sum Graphs saved locally. You can now use 'sum'.");

        Ok(())
    }
    pub async fn call_list(&self) -> Result<()> {
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = format!("https://pixe.la/v1/users/{name}/graphs/");
        let graph_list = &self.session.get_graph_list(api_key, &url).await;
        if let CallResult::List(list) = graph_list.as_ref().map_err(|e| Error::MissingEntryInDatabase(e.to_string()))?{
            list.iter().for_each(|graph_id| println!("Graph Name: {}", graph_id.trim_matches('"')));
        };

        Ok(())
    }
    pub async fn call_create_user(&self, args: NewUserArgs<'_>) -> Result<()> {
        let NewUserArgs{token, username, minor, tos} = args;
        let _ = &self.session.create_user(token, username, minor, tos).await?;
        println!("Success: Account created, from now on you are logged in on this device");
        match &self.call_save_data(LoginArgs{name: username, api_key: token}) {
            Ok(_) => (),
            Err(err) => println!("Local database failure: {err}"),
        }
        return Ok(());

    }
    pub async fn call_create_graph(&self, args: CreateGraphArgs<'_>) -> Result<()> {
        let CreateGraphArgs{name, id, number_type, color, unit} = args;
        let username = &self.name.to_owned().expect("Data should be there");
        let token = &self.api_key.to_owned().expect("Data should be there");
        self.session.create_graph(username, token, id, name, number_type, unit, color).await?;
        println!("Success: New graph created, check it out at https://pixe.la/v1/users/{}/graphs/{}.html.", username, id);
        return Ok(());

    }
    pub async fn call_remove_graph(&self, args: RemoveArgs<'_> ) -> Result<()> {
        let username = &self.name.to_owned().expect("Data should be there");
        let token = &self.api_key.to_owned().expect("Data should be there");
        let graph_name = args.graph_name;
        self.session.remove_graph(username, token, graph_name).await?;
        println!("Success: A graph has been removed from your account.");
        return Ok(());
    }

    pub async fn call_streak(&self, args: StreakGetArgs<'_>) -> Result<()> {
        let username = &self.name.to_owned().expect("Data should be there");
        let token = &self.api_key.to_owned().expect("Data should be there");
        let streak = self.session.get_streak(username, token, &args.graph_id).await?;
        println!("{}", prepare_streak_string(streak, &args.graph_id));
        Ok(())
    }
    pub fn print_data(&self) -> Result<()> {
        println!("{}", &self);
        Ok(())
    }

}
fn input_graph_names(sum_graphs: &mut Vec<SumGraphStruct>, sum_graph_names_duplicate_tracker: &mut Vec<String>, sum_graphs_amount: usize, correct_names: &Vec<String>) -> Result<()> {
    let mut correct_names_string = String::new(); 
    correct_names.iter().for_each(|name| correct_names_string.push_str(&format!("{name}\n")));
    println!("Graphs available for your account:\n{correct_names_string}");
    while sum_graphs.len() < sum_graphs_amount {
        let mut input: String = String::new();
        println!("Enter data #{}: (sum_graph_id graph_id(s)...", sum_graphs.len()+1);
        stdin().read_line(&mut input).unwrap();
        let input: Vec<_> = input.trim().split(" ").collect();
        let sum_graph_name = input.get(0).unwrap().to_string();

        // check for duplicates
        match sum_graph_names_duplicate_tracker.contains(&sum_graph_name) { 
            true => return Err(Error::SumGraphError(SumGraphError::new(error::SumGraphErrorKind::RepeatingNames))),
            false => sum_graph_names_duplicate_tracker.push(sum_graph_name.clone()),
        }

        let mut graphs: Vec<String> = input.into_iter().map(|n| n.to_string()).collect();
        graphs.dedup();
        graphs.iter().try_for_each(|name| -> Result<()> {if !correct_names.contains(name) {return Err(Error::SumGraphError(SumGraphError::new(SumGraphErrorKind::IncorrectNames)))}; return Ok(())})?;
        graphs.remove(0);
        sum_graphs.push(SumGraphStruct::new(sum_graph_name, graphs));
        println!("Input accepted.");
    }
        Ok(())

}
#[cfg(test)]
mod test {
    use crate::{args::{SumArgs, SumGraphArgs}, pixela::Session, Worker};

    #[tokio::test]
    async fn saving_graphs() {
        let session = Session::new();
        let worker = Worker::new(session);
        let args = SumGraphArgs{sum_graph_amount: 2};
        worker.setup_graphs(args).await.unwrap();

    }


}

