use clap::Parser;
use habitcli::{
    args::{self, IntoArguments, ParsedArguments},
    pixela::Session, Worker, error::Error
};
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = args::HabitCLIArgs::parse();

    let session = Session::new();
    let mut worker = Worker::new(session);
// match statement for every possible user inputted command
    match args.command_type {
        args::CommandType::Signup(arguments) => {
            if let ParsedArguments::NewUserData(args) = arguments.into_args() {
                worker.call_create_user(args).await?;
            }
        }
        args::CommandType::Send(arguments) => {
            worker.login()?;
            if let ParsedArguments::PixelArgs(args) = arguments.into_args() {
                worker.call_send(args).await;
            }
        }

        args::CommandType::Get(arguments) => {
            worker.login()?;
            if let ParsedArguments::PixelArgs(args) = arguments.into_args() {
                worker.call_get(args).await;
            }
        }

        args::CommandType::Login(arguments) => {
            if let ParsedArguments::LoginArgs(args) = arguments.into_args() {
                match worker.call_save_data(args) {
                    Ok(_) => (),
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        args::CommandType::Create(arguments) => {
            worker.login()?;
            if let ParsedArguments::GraphCreateArgs(args) = arguments.into_args() {
                worker.call_create_graph(args).await?;
            }
        }
        args::CommandType::Remove(arguments) => {
            worker.login()?;
            if let ParsedArguments::RemoveArgs(args) = arguments.into_args() {
                worker.call_remove_graph(args).await?;
            }
        }
        args::CommandType::List(_) => {
            worker.login()?;
            worker.call_list().await?;
        }
        args::CommandType::Streak(arguments) => {
            worker.login()?;
            if let ParsedArguments::StreakGetArgs(args) = arguments.into_args() {
                worker.call_streak(args).await?
            }
        }
        args::CommandType::SetupSum(arguments) => {
            worker.login()?;
            if let ParsedArguments::SumGraphArgs(args) = arguments.into_args()  {
                match worker.setup_graphs(args).await{
                    Ok(_) => (),
                    Err(e) => println!("{e}")
                }
            }
        }
        args::CommandType::Data(_) => {
            worker.login()?;
            worker.print_data()?;
        }

        args::CommandType::Sum(arguments) => {
            worker.login()?;
            if let ParsedArguments::SumArgs(args) = arguments.into_args() {
                match worker.handle_sum_graph(args).await {
                    Ok(_) => (),
                    Err(e) => println!("{:?}",e)
                }
            worker.login()?;
    
        }
        }

    };
    Ok(())
}
