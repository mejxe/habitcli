use clap::{Args, Parser, Subcommand};

#[derive(Debug)]
// enum storing all possible argument types for cleaner data passing
pub enum ParsedArguments<'a> {
    PixelArgs(PixelArgs<'a>),
    LoginArgs(LoginArgs<'a>),
    SumGraphArgs(SumGraphArgs<'a>),
    SumArgs(SumArgs<'a>),
    NewUserData(NewUserArgs<'a>),
    GraphCreateArgs(CreateGraphArgs<'a>),
    StreakGetArgs(StreakGetArgs<'a>)
}

// a func that turns data from each command into an enum standarized enum variant
pub trait IntoArguments {
    fn into_args(&self) -> ParsedArguments;
}
#[derive(Debug)]
pub struct StreakGetArgs<'a> {
    pub graph_id: &'a str
}

#[derive(Debug)]
pub struct CreateGraphArgs<'a> {
    pub id: &'a str,
    pub name: &'a str ,
    pub number_type: &'a str,
    pub unit: &'a str,
    pub color: &'a str,
}
#[derive(Debug)]
pub struct NewUserArgs<'a> {
    pub token: &'a str,
    pub username: &'a str,
    pub minor: bool,
    pub tos: bool,
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
    /// Use to create a pixela account.
    Signup(NewUser),
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
    /// Creates a new graph on Pixela.
    Create(CreateGraph),
    /// Calculates your current streak of consecutive pixels
    Streak(GetStreak)
}

#[derive(Debug, Args)]
pub struct NewUser {
    /// Your new password.
    password: String,
    /// Your new username.
    username: String,
    /// Do you agree to Pixela's TOS? [yes/no]
    tos: String,
    /// Are you a minor? [yes/no]
    minor: String
}

#[derive(Debug, Args)]
pub struct GetList {}

#[derive(Debug, Args)]
pub struct GetStreak {
    /// Graph id
    graph_id: String
}
#[derive(Debug, Args)]
pub struct SumGraphs {
    /// Optional date to sum from. By default today.
    #[clap(short,long)]
    date: Option<String> 
}

#[derive(Debug, Args)]
pub struct GetData;

#[derive(Parser,Debug)]
pub struct CreateGraph {
    /// ID of the new graph.
    pub id: String,
    
    /// Name of the new graph.
    pub name: String,

    /// Type of the value of the unit (only int and float are supported)
    pub number_type: String,

    /// Unit used to track data on your graph (ex. hours, commits, miles).
    pub unit: String,

    /// Color of graphs pixels. Valid ones are: shibafu (green), momiji (red), sora (blue), ichou (yellow), ajisai (purple) and kuro (black).
    pub color: String,
}
#[derive(Debug, Args)]
pub struct SumGraph {
    /// List of graphs you want to sum in a sum graph (just the graph names) [max 3].
    #[clap(short,long,value_delimiter = ' ', num_args=2)]
    graphs: Vec<String>,
    /// Sum graph (just the name).
    #[clap(short,long)]
    sum_graph: String
}
impl IntoArguments for GetStreak {
    fn into_args(&self) -> ParsedArguments {
        let graph_id = &self.graph_id;
        let args = StreakGetArgs{graph_id};
        return ParsedArguments::StreakGetArgs(args)
    }
}
impl IntoArguments for CreateGraph {
    fn into_args(&self) -> ParsedArguments {
        let CreateGraph{id, name, number_type, unit, color} = &self;
        let args = CreateGraphArgs{id, name, number_type, unit, color};
        return ParsedArguments::GraphCreateArgs(args)
    }
}
impl IntoArguments for NewUser {
    fn into_args(&self) -> ParsedArguments {
        let username = &self.username;
        let token = &self.username;
        let minor = if &self.minor == "yes" {true} else {false}; 
        let tos = if &self.tos == "yes" {true} else {false}; 
        let args = NewUserArgs{username, token, minor, tos};
        return ParsedArguments::NewUserData(args)
    }
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
