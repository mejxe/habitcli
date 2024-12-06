use clap::{Args, Parser, Subcommand};

#[derive(Debug)]
// enum storing all possible argument types for cleaner data passing
pub enum ParsedArguments<'a> {
    PixelArgs(PixelArgs<'a>),
    LoginArgs(LoginArgs<'a>),
    SumGraphArgs(SumGraphArgs<'a>),
    SumArgs(SumArgs<'a>)
}

// a func that turns data from each command into an enum standarized enum variant
pub trait IntoArguments {
    fn into_args(&self) -> ParsedArguments;
}
#[derive(Debug)]
pub struct PixelArgs<'a> {
    pub graph: &'a str,
    pub date: Option<&'a str>,
    pub quantity: &'a str,
}
#[derive(Debug)]
pub struct SumArgs<'a> {
    pub date: Option<&'a str>,
}
#[derive(Debug)]
pub struct SumGraphArgs<'a> {
    // graphs that are suppossed to get summed up and sum_graph that stores id of a graph the
    // result is uploaded to
    pub graph_to_sum1: &'a str,
    pub graph_to_sum2: &'a str,
    pub sum_graph: &'a str,
}

#[derive(Debug)]
pub struct LoginArgs<'a> {
    //username and user's api key
    pub name: &'a str,
    pub api_key: &'a str,
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct HabitCLIArgs {
    #[clap(subcommand)]
    pub command_type: CommandType,
}
// all possible commands
#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Use to send pixels to Pixela.
    Send(SendPixel),
    /// Use to get pixels data from Pixela.
    Get(GetPixel),
    /// Use to log in with your pixela api token and pixela name.
    Login(LoginUser),
    /// List all graphs.
    List(GetList),
    /// Setup sum graph functionality.
    SetupSum(SumGraph),
    /// Print your data.
    Data(GetData),
    /// Sums today progress of your graphs to sum and uploads it to sum graph.
    Sum(SumGraphs),
}
#[derive(Debug, Args)]
pub struct GetList {}

#[derive(Debug, Args)]
pub struct SumGraphs {
    /// Optional date to sum from. By default today.
    #[clap(short,long)]
    date: Option<String> 
}

#[derive(Debug, Args)]
pub struct GetData;

#[derive(Debug, Args)]
pub struct SumGraph {
    /// List of graphs you want to sum in a sum graph (just the graph names) [max 3].
    #[clap(short,long,value_delimiter = ' ', num_args=2)]
    graphs: Vec<String>,
    /// Sum graph (just the name).
    #[clap(short,long)]
    sum_graph: String
}
impl IntoArguments for SumGraph {
    fn into_args(&self) -> ParsedArguments {
        let graph1 = &self.graphs[0];
        let graph2 = &self.graphs[1];
        let args = SumGraphArgs {graph_to_sum1: graph1, graph_to_sum2: graph2, sum_graph: &self.sum_graph };
        ParsedArguments::SumGraphArgs(args)
    }
}
impl IntoArguments for SumGraphs {
    fn into_args(&self) -> ParsedArguments {
        let args = SumArgs{date: self.date.as_deref()};
        ParsedArguments::SumArgs(args)
    }
}
        
#[derive(Debug, Args)]
pub struct LoginUser {
    /// Pixela username.
    #[arg()]
    name: String,
    #[arg()]
    /// Pixela api key.
    api_key: String,
}
impl IntoArguments for LoginUser {
    fn into_args(&self) -> ParsedArguments {
        ParsedArguments::LoginArgs(LoginArgs {
            name: &self.name,
            api_key: &self.api_key,
        })
    }
}
#[derive(Debug, Args)]
pub struct SendPixel {
    /// Date of a pixel that you wish to modify. Format: "yyyymmdd". Leave blank to upload for today.
    #[arg(short, long)]
    pub date: Option<String>,
    /// Graph id to interact with. (the name in the url on pixela)
    pub graph_id: String,
    /// Number of commits that you wish to send.
    pub quantity: String,
}
impl IntoArguments for SendPixel {
    fn into_args(&self) -> ParsedArguments {
        let date = &self.date;
        let graph = &self.graph_id;
        let quantity = &self.quantity;
        ParsedArguments::PixelArgs(PixelArgs {
            graph,
            date: date.as_deref(),
            quantity,
        })
    }
}

#[derive(Debug, Args)]
pub struct GetPixel {
    /// Date of a pixel that you wish to modify. Format: "yyyymmdd". Leave blank to upload for today.
    #[arg(short, long)]
    date: Option<String>,
    /// Graph name to interact with. (the name in the url on pixela)
    graph_id: String,
}

impl IntoArguments for GetPixel {
    fn into_args(&self) -> ParsedArguments {
        let date = &self.date;
        let graph = &self.graph_id;
        ParsedArguments::PixelArgs(PixelArgs {
            date: date.as_deref(),
            graph,
            quantity: "0",
        })
    }
}
