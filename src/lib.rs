//pub use args::IntoArguments;
pub mod args;
pub mod pixela;
pub mod user_data;
pub mod error;

use std::fmt::Display;

use error::Error;

use args::{LoginArgs, PixelArgs, SumGraphArgs, SumArgs};
use pixela::*;
pub struct Worker {
    /*
    Worker struct that calls all the functions 
    to keep main clenaer.
     */
    session: Session,
    api_key: Option<String>,
    name: Option<String>,
    sum_graph:Option<String>,
    graph_to_sum1:Option<String>,
    graph_to_sum2:Option<String>
}
impl Display for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Username: {:?}\nSum graph: {:?}\nGraph to sum #1: {:?}\nGraph to sum#2: {:?}\n", self.name, self.sum_graph, self.graph_to_sum1, self.graph_to_sum2)
        }
    }
impl Worker {
    pub fn new(session: Session) -> Worker {
        Worker {
            session,
            api_key: None,
            name: None,
            sum_graph: None,
            graph_to_sum1: None,
            graph_to_sum2: None,
        }
    }
    pub fn login(&mut self) -> Result<(), Error> {
        // gets data from local database and saves it in the struct
        let user = user_data::User::new();
        let data = user.get_user_data()?;
        let (name, token, sum_graph) = (data.name, data.token, data.sum_graph);
        if let Some(graphs_to_sum) = data.graphs_to_sum {
            self.graph_to_sum1 = Some(graphs_to_sum[0].clone());
            self.graph_to_sum2 = Some(graphs_to_sum[1].clone());
        };
        if let Some(sum_graph) = sum_graph {
            self.sum_graph = Some(sum_graph);
        }
        self.api_key = Some(token);
        self.name = Some(name);
        
        Ok(())
    }

    pub fn call_send(&self, args: PixelArgs) {
        let graph = args.graph;
        let date = args.date;
        let quantity = &args.quantity;
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = &self.create_url(graph, &name);
        match &self
            .session
            .send_pixel(url, quantity, date.as_deref(), &api_key)
        {
            Ok(call_result) => {
                if let CallResult::ApiResponse(msg) = call_result {
                    msg.out_message();
                }
            }
            Err(e) => println!("There was an error. {:?}", e),
        };
    }
    pub fn handle_sum_graph(&self, args: SumArgs) -> Result<(), Error>{
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let graph_names = vec![self.graph_to_sum1.clone().unwrap(), self.graph_to_sum2.clone().unwrap()];
        let date = args.date;
        let commits = self.session.get_graphs_to_sum_commits(graph_names, api_key, name, date)?;

        let date = args.date;
        let url = &self.create_url(&self.sum_graph.clone().unwrap(), &name);
        self.session.send_pixel(url, &commits, date, api_key)?;
        return Ok(());
    }
        

    pub fn call_get(&self, args: PixelArgs) {
        let graph = args.graph;
        let date = args.date;
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = &self.create_url(graph, &name);
        match &self
            .session
            .get_pixel_info(url, graph, date.as_deref(), &api_key)
        {
            Ok(call_result) => {
                if let CallResult::Heatmap(heatmap) = call_result {
                    heatmap.out_heatmap_info();
                }
            }
            Err(e) => println!("There was an error. {:?}", e),
        };
    }
    fn create_url(&self, graph: &str, name: &str) -> String {
        format!("https://pixe.la/v1/users/{name}/graphs/{graph}")
    }
    pub fn call_save_data(&self, args: LoginArgs) -> Result<(), Error> {
        let user = user_data::User::new();
        Ok(user.set_user_data(args.name, args.api_key)?)
    }
    pub fn call_save_sum_graph(&self, args: SumGraphArgs) -> Result<(), Error> {
        let user = user_data::User::new();
        (user.setup_sum_graph(args.sum_graph, args.graph_to_sum1, args.graph_to_sum2))?;
        Ok(())
    }
    pub fn call_list(&self) -> Result<(), Error> {
        let name = &self.name.to_owned().expect("Data should be there");
        let api_key = &self.api_key.to_owned().expect("Data should be there");
        let url = format!("https://pixe.la/v1/users/{name}/graphs/");
        let graph_list = &self.session.get_graph_list(api_key, &url);
        if let CallResult::List(list) = graph_list.as_ref().map_err(|e| Error::MissingEntryInDatabase(e.to_string()))?{
            list.iter().for_each(|graph_id| println!("{}", graph_id));
        };

        Ok(())
    }
    pub fn print_data(&self) -> Result<(), Error> {
        println!("{}", &self);
        Ok(())
    }

}

